use cc_core::{OccupancyGrid, OccupancyState};
use thiserror::Error;

use crate::{SpaceConfig, SpaceConfigError};

/// A validated voxel grid covering one configured space.
pub struct VoxelGrid {
    grid: OccupancyGrid,
}

/// An error constructing the grid or converting a position or index.
#[derive(Debug, Error)]
pub enum VoxelGridError {
    /// The input space is invalid.
    #[error("invalid space: {0}")]
    InvalidSpace(#[from] SpaceConfigError),
    /// An axis requires more cells than the shared grid can represent.
    #[error("voxel count cannot be represented on axis {axis}")]
    InvalidDimensions {
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
    /// The total number of cells cannot be represented or allocated.
    #[error("voxel grid is too large to allocate")]
    GridTooLarge,
    /// The grid cell array exceeds the configured memory budget.
    #[error(
        "grid requires {required_bytes} bytes for cells, exceeding the {limit_bytes}-byte limit; increase voxel size, reduce the space, or raise the limit"
    )]
    MemoryLimitExceeded {
        /// Bytes required for the cell values.
        required_bytes: usize,
        /// Configured maximum bytes for the cell values.
        limit_bytes: usize,
    },
    /// A coordinate is non-finite or outside the physical extent.
    #[error("position is outside the space or non-finite on axis {axis}")]
    InvalidPosition {
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
    /// An index is outside the grid.
    #[error("voxel index is outside the grid on axis {axis}")]
    InvalidIndex {
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
}

impl VoxelGrid {
    /// Creates an initially unknown grid covering the configured space.
    ///
    /// # Errors
    /// Returns an error for invalid configuration, unrepresentable dimensions
    /// or a grid that cannot be allocated.
    pub fn new(config: &SpaceConfig) -> Result<Self, VoxelGridError> {
        config.validate()?;
        let extent = [config.width, config.depth, config.height];
        let mut dimensions = [0_u32; 3];
        for axis in 0..3 {
            let ratio = extent[axis] / config.voxel_size;
            if !ratio.is_finite() || ratio > f64::from(u32::MAX) {
                return Err(VoxelGridError::InvalidDimensions { axis });
            }
            let mut count = ratio.ceil().max(1.0) as u32;
            // Division can round just above an integer even when its cell
            // boundary already covers the exact configured extent.
            if count > 1 && f64::from(count - 1) * config.voxel_size >= extent[axis] {
                count -= 1;
            }
            let origin = config.origin[axis];
            let upper = origin + extent[axis];
            if count > 1
                && (origin + config.voxel_size <= origin || upper - config.voxel_size >= upper)
            {
                return Err(VoxelGridError::InvalidDimensions { axis });
            }
            // Rounding after adding the origin can also create an empty cell
            // at the upper boundary even when the relative extent is larger.
            while count > 1 && origin + f64::from(count - 1) * config.voxel_size >= upper {
                count -= 1;
            }
            dimensions[axis] = count;
        }
        let count = dimensions
            .iter()
            .try_fold(1_usize, |count, &dimension| {
                usize::try_from(dimension)
                    .ok()
                    .and_then(|value| count.checked_mul(value))
            })
            .ok_or(VoxelGridError::GridTooLarge)?;
        let required_bytes = count
            .checked_mul(std::mem::size_of::<OccupancyState>())
            .ok_or(VoxelGridError::GridTooLarge)?;
        if required_bytes > config.max_grid_bytes {
            return Err(VoxelGridError::MemoryLimitExceeded {
                required_bytes,
                limit_bytes: config.max_grid_bytes,
            });
        }
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(count)
            .map_err(|_| VoxelGridError::GridTooLarge)?;
        cells.resize(count, OccupancyState::Unknown);
        Ok(Self {
            grid: OccupancyGrid {
                origin: config.origin,
                dimensions,
                extent,
                voxel_size: [config.voxel_size; 3],
                cells,
            },
        })
    }

    /// Borrows the shared occupancy representation of this grid.
    pub fn occupancy(&self) -> &OccupancyGrid {
        &self.grid
    }

    /// Converts a world position in metres to zero-based voxel indices.
    ///
    /// # Errors
    /// Returns an error if any coordinate is non-finite or outside the space.
    pub fn world_to_voxel(&self, position: [f64; 3]) -> Result<[u32; 3], VoxelGridError> {
        let mut index = [0_u32; 3];
        for axis in 0..3 {
            let origin = self.grid.origin[axis];
            let upper = origin + self.grid.extent[axis];
            if !position[axis].is_finite() || position[axis] < origin || position[axis] >= upper {
                return Err(VoxelGridError::InvalidPosition { axis });
            }
            let cell = ((position[axis] - origin) / self.grid.voxel_size[axis]).floor();
            // The physical bounds check above is authoritative. Subtraction or
            // division may round an in-bounds point up to the cell count.
            let mut candidate = (cell as u32).min(self.grid.dimensions[axis] - 1);
            // Compare against boundaries in world coordinates to account for
            // rounding when a shifted origin is subtracted.
            if candidate > 0
                && position[axis] < origin + f64::from(candidate) * self.grid.voxel_size[axis]
            {
                candidate -= 1;
            } else if candidate + 1 < self.grid.dimensions[axis]
                && position[axis] >= origin + f64::from(candidate + 1) * self.grid.voxel_size[axis]
            {
                candidate += 1;
            }
            index[axis] = candidate;
        }
        Ok(index)
    }

    /// Returns the centre of a voxel's intersection with the physical space.
    ///
    /// Edge cells can be partial. The round-trip error is at most half a
    /// nominal voxel size per axis, subject to floating-point rounding.
    ///
    /// # Errors
    /// Returns an error if any index is outside the grid.
    pub fn voxel_to_world(&self, index: [u32; 3]) -> Result<[f64; 3], VoxelGridError> {
        let mut position = [0.0; 3];
        for axis in 0..3 {
            if index[axis] >= self.grid.dimensions[axis] {
                return Err(VoxelGridError::InvalidIndex { axis });
            }
            let lower =
                self.grid.origin[axis] + f64::from(index[axis]) * self.grid.voxel_size[axis];
            let upper = (self.grid.origin[axis]
                + (f64::from(index[axis]) + 1.0) * self.grid.voxel_size[axis])
                .min(self.grid.origin[axis] + self.grid.extent[axis]);
            let centre = lower + (upper - lower) / 2.0;
            // For a one-ULP partial cell the midpoint can round to the
            // excluded upper boundary; use the representable lower edge.
            position[axis] = if centre >= upper { lower } else { centre };
        }
        Ok(position)
    }
}
