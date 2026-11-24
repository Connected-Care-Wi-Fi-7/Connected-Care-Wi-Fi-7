use std::{fs, path::Path};

use serde::Deserialize;
use thiserror::Error;

/// Default maximum memory for voxel cell values: 256 MB (decimal).
pub const DEFAULT_MAX_GRID_BYTES: usize = 256_000_000;

fn default_max_grid_bytes() -> usize {
    DEFAULT_MAX_GRID_BYTES
}

/// Configuration of the mapped space in metres.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpaceConfig {
    /// Lower corner of the space in metres, ordered x, y, z.
    pub origin: [f64; 3],
    /// Space dimension along the x-axis.
    pub width: f64,
    /// Space dimension along the y-axis.
    pub depth: f64,
    /// Space dimension along the z-axis.
    pub height: f64,
    /// Side length of each cubic voxel in metres.
    pub voxel_size: f64,
    /// Access-point positions in metres, ordered x, y, z.
    pub access_points: Vec<[f64; 3]>,
    /// Maximum bytes for voxel cells, not including other application memory.
    #[serde(default = "default_max_grid_bytes")]
    pub max_grid_bytes: usize,
}

/// A failure reading, parsing or validating a space configuration file.
#[derive(Debug, Error)]
pub enum SpaceConfigLoadError {
    /// The configuration file could not be read.
    #[error("failed to read space configuration: {0}")]
    Read(#[source] std::io::Error),
    /// The configuration is not valid TOML or has the wrong fields.
    #[error("invalid space configuration TOML: {0}")]
    Parse(#[from] toml::de::Error),
    /// The parsed configuration describes an invalid space.
    #[error("invalid space configuration: {0}")]
    Invalid(#[from] SpaceConfigError),
}

/// A problem with the configured space or an access-point position.
#[derive(Debug, Error, PartialEq)]
pub enum SpaceConfigError {
    /// An origin coordinate is not finite.
    #[error("origin coordinate on axis {axis} must be finite")]
    InvalidOrigin {
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
    /// A space dimension or voxel size is not finite and positive.
    #[error("{field} must be finite and greater than zero")]
    InvalidSize {
        /// Name of the invalid field.
        field: &'static str,
    },
    /// The grid memory budget is zero.
    #[error("max_grid_bytes must be greater than zero")]
    InvalidMemoryLimit,
    /// An axis has no representable finite upper boundary.
    #[error("upper boundary on axis {axis} must be finite and greater than the origin")]
    InvalidUpperBoundary {
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
    /// An access point is non-finite or outside the space.
    #[error("access point {index} is outside the space or non-finite on axis {axis}")]
    InvalidAccessPoint {
        /// Zero-based index of the access point in the configuration.
        index: usize,
        /// Axis index: 0 = x, 1 = y, 2 = z.
        axis: usize,
    },
}

impl SpaceConfig {
    /// Reads a local TOML file and validates its space configuration.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read, parsed or validated.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, SpaceConfigLoadError> {
        let contents = fs::read_to_string(path).map_err(SpaceConfigLoadError::Read)?;
        Self::from_toml(&contents)
    }

    /// Parses and validates a space configuration from TOML text.
    ///
    /// # Errors
    /// Returns an error if the input cannot be parsed or describes an invalid space.
    pub fn from_toml(contents: &str) -> Result<Self, SpaceConfigLoadError> {
        let config: Self = toml::from_str(contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Checks dimensions and access points against the configured space.
    ///
    /// Access-point positions may lie on either boundary of each axis. Voxel
    /// lookups remain upper-exclusive. Since the fields are public, callers
    /// must validate again after modifying them.
    ///
    /// # Errors
    ///
    /// Returns an error for non-finite origins, non-positive or non-finite
    /// sizes, unrepresentable upper bounds, or access points outside the space.
    pub fn validate(&self) -> Result<(), SpaceConfigError> {
        if self.max_grid_bytes == 0 {
            return Err(SpaceConfigError::InvalidMemoryLimit);
        }
        let dimensions = [self.width, self.depth, self.height];
        for (field, size) in [
            ("width", self.width),
            ("depth", self.depth),
            ("height", self.height),
            ("voxel_size", self.voxel_size),
        ] {
            if !size.is_finite() || size <= 0.0 {
                return Err(SpaceConfigError::InvalidSize { field });
            }
        }

        let mut upper = [0.0; 3];
        for axis in 0..3 {
            let origin = self.origin[axis];
            if !origin.is_finite() {
                return Err(SpaceConfigError::InvalidOrigin { axis });
            }
            upper[axis] = origin + dimensions[axis];
            if !upper[axis].is_finite() || upper[axis] <= origin {
                return Err(SpaceConfigError::InvalidUpperBoundary { axis });
            }
        }

        for (index, position) in self.access_points.iter().enumerate() {
            for axis in 0..3 {
                let coordinate = position[axis];
                if !coordinate.is_finite()
                    || coordinate < self.origin[axis]
                    || coordinate > upper[axis]
                {
                    return Err(SpaceConfigError::InvalidAccessPoint { index, axis });
                }
            }
        }
        Ok(())
    }
}
