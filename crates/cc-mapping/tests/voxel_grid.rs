use cc_core::OccupancyState;
use cc_mapping::{SpaceConfig, VoxelGrid, VoxelGridError};

// A one-voxel cross-section keeps the floating-point boundary tests focused on x.
fn single_axis_space(origin: f64, width: f64, voxel_size: f64) -> SpaceConfig {
    SpaceConfig {
        origin: [origin, 0.0, 0.0],
        width,
        depth: voxel_size,
        height: voxel_size,
        voxel_size,
        access_points: vec![],
        max_grid_bytes: cc_mapping::DEFAULT_MAX_GRID_BYTES,
    }
}

#[test]
fn floating_point_boundaries_preserve_valid_cells() {
    let cases = [
        ("rounded division", -2.0, 1.1, 1.1, 1),
        (
            "one-ulp partial cell",
            -2.0,
            (-1.8_f64).next_up() - (-2.0),
            0.1,
            3,
        ),
        ("shifted origin", -2.9, 7.600000000000001, 0.4, 19),
        ("decimal ratio", 0.0, 0.07, 0.01, 7),
    ];
    for (name, origin, width, size, count) in cases {
        let grid = VoxelGrid::new(&single_axis_space(origin, width, size)).unwrap();
        assert_eq!(grid.occupancy().dimensions[0], count, "{name}");
        let last = [count - 1, 0, 0];
        let position = grid.voxel_to_world(last).unwrap();
        assert!(position[0] < origin + width, "{name}");
        assert_eq!(grid.world_to_voxel(position).unwrap(), last, "{name}");
        assert!(
            grid.world_to_voxel([origin + width, 0.0, 0.0]).is_err(),
            "{name}"
        );
    }
    let grid = VoxelGrid::new(&single_axis_space(-2.0, 1.1, 1.1)).unwrap();
    assert_eq!(grid.world_to_voxel([-0.9, 0.0, 0.0]).unwrap(), [0, 0, 0]);
}

#[test]
fn grid_memory_budget_checks_limits_before_allocation() {
    let mut space = config();
    let bytes_per_cell = std::mem::size_of::<OccupancyState>();
    let required = 6 * bytes_per_cell;
    space.max_grid_bytes = required;
    assert!(VoxelGrid::new(&space).is_ok());
    space.max_grid_bytes = required - 1;
    assert!(
        matches!(VoxelGrid::new(&space), Err(VoxelGridError::MemoryLimitExceeded {
        required_bytes, limit_bytes
    }) if required_bytes == required && limit_bytes == required - 1)
    );
    space.max_grid_bytes = required + 1;
    assert!(VoxelGrid::new(&space).is_ok());
    space.width = 100_000.0;
    space.depth = 100_000.0;
    space.height = 0.5;
    space.max_grid_bytes = cc_mapping::DEFAULT_MAX_GRID_BYTES;
    space.access_points.clear();
    assert!(matches!(
        VoxelGrid::new(&space),
        Err(VoxelGridError::MemoryLimitExceeded { .. })
    ));
}

#[test]
fn overflowing_cell_count_is_rejected_without_allocating() {
    let mut space = config();
    space.width = 1_000_000_000.0;
    space.depth = 1_000_000_000.0;
    space.height = 100.0;
    space.voxel_size = 1.0;
    space.access_points.clear();
    assert!(matches!(
        VoxelGrid::new(&space),
        Err(VoxelGridError::GridTooLarge)
    ));
}

fn config() -> SpaceConfig {
    SpaceConfig {
        origin: [2.0, -1.0, 0.0],
        width: 1.25,
        depth: 1.0,
        height: 0.5,
        voxel_size: 0.5,
        access_points: vec![[2.1, -0.9, 0.25]],
        max_grid_bytes: cc_mapping::DEFAULT_MAX_GRID_BYTES,
    }
}

#[test]
fn builds_unknown_grid_including_partial_edge_cells() {
    let grid = VoxelGrid::new(&config()).unwrap();
    let data = grid.occupancy();
    assert_eq!(data.origin, [2.0, -1.0, 0.0]);
    assert_eq!(data.extent, [1.25, 1.0, 0.5]);
    assert_eq!(data.dimensions, [3, 2, 1]);
    assert_eq!(data.voxel_size, [0.5; 3]);
    assert_eq!(data.cells.len(), 6);
    assert!(
        data.cells
            .iter()
            .all(|value| matches!(value, OccupancyState::Unknown))
    );
}

#[test]
fn maps_shifted_origin_and_half_open_boundaries() {
    let grid = VoxelGrid::new(&config()).unwrap();
    assert_eq!(grid.world_to_voxel([2.0, -1.0, 0.0]).unwrap(), [0, 0, 0]);
    assert_eq!(grid.world_to_voxel([2.5, -0.5, 0.0]).unwrap(), [1, 1, 0]);
    assert_eq!(grid.world_to_voxel([3.24, -0.01, 0.49]).unwrap(), [2, 1, 0]);
    for point in [
        [1.99, -1.0, 0.0],
        [3.25, -1.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, -1.0, 0.5],
        [f64::NAN, -1.0, 0.0],
    ] {
        assert!(matches!(
            grid.world_to_voxel(point),
            Err(VoxelGridError::InvalidPosition { .. })
        ));
    }
}

#[test]
fn centres_partial_cells_and_round_trips_within_half_voxel_per_axis() {
    let grid = VoxelGrid::new(&config()).unwrap();
    assert_eq!(
        grid.voxel_to_world([2, 0, 0]).unwrap(),
        [3.125, -0.75, 0.25]
    );
    for point in [[2.0, -1.0, 0.0], [2.5, -0.5, 0.1], [3.24, -0.01, 0.49]] {
        let centre = grid
            .voxel_to_world(grid.world_to_voxel(point).unwrap())
            .unwrap();
        for axis in 0..3 {
            assert!((point[axis] - centre[axis]).abs() <= 0.25);
        }
    }
    assert!(matches!(
        grid.voxel_to_world([3, 0, 0]),
        Err(VoxelGridError::InvalidIndex { axis: 0 })
    ));
}

#[test]
fn rejects_unrepresentable_grid_dimensions() {
    let mut space = config();
    space.voxel_size = 1e-20;
    assert!(matches!(
        VoxelGrid::new(&space),
        Err(VoxelGridError::InvalidDimensions { .. })
    ));
}

#[test]
fn rejects_voxels_too_small_to_distinguish_at_the_grid_origin() {
    let space = single_axis_space(1e16, 8.0, 1.0);
    assert!(matches!(
        VoxelGrid::new(&space),
        Err(VoxelGridError::InvalidDimensions { axis: 0 })
    ));
}
