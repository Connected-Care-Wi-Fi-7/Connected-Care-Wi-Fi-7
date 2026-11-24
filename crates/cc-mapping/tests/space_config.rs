use cc_mapping::{SpaceConfig, SpaceConfigError, SpaceConfigLoadError};

#[test]
fn loads_the_example_configuration() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/space.example.toml"
    );
    let space = SpaceConfig::load(path).unwrap();
    assert_eq!(space.width, 5.0);
    assert_eq!(space.voxel_size, 0.5);
    assert_eq!(space.access_points, vec![[1.0, 1.0, 2.0]]);
}

#[test]
fn rejects_invalid_toml_and_invalid_configuration() {
    assert!(matches!(
        SpaceConfig::from_toml("width = ["),
        Err(SpaceConfigLoadError::Parse(_))
    ));
    let invalid = "origin = [0.0, 0.0, 0.0]\nwidth = 0.0\ndepth = 1.0\nheight = 1.0\nvoxel_size = 0.5\naccess_points = []";
    assert!(matches!(
        SpaceConfig::from_toml(invalid),
        Err(SpaceConfigLoadError::Invalid(_))
    ));
}

#[test]
fn configuration_defaults_memory_budget_and_rejects_zero() {
    let text = "origin = [0.0, 0.0, 0.0]\nwidth = 1.0\ndepth = 1.0\nheight = 1.0\nvoxel_size = 0.5\naccess_points = []";
    let mut space = SpaceConfig::from_toml(text).unwrap();
    assert_eq!(space.max_grid_bytes, cc_mapping::DEFAULT_MAX_GRID_BYTES);
    space.max_grid_bytes = 0;
    assert!(matches!(
        space.validate(),
        Err(SpaceConfigError::InvalidMemoryLimit)
    ));
}

fn valid_space() -> SpaceConfig {
    SpaceConfig {
        origin: [2.0, -1.0, 0.0],
        width: 5.0,
        depth: 4.0,
        height: 2.5,
        voxel_size: 0.3,
        access_points: vec![[2.0, -1.0, 0.0], [6.9, 2.9, 2.4]],
        max_grid_bytes: cc_mapping::DEFAULT_MAX_GRID_BYTES,
    }
}

#[test]
fn accepts_valid_space_with_shifted_origin_and_partial_edge_cells() {
    assert_eq!(valid_space().validate(), Ok(()));
}

#[test]
fn accepts_no_access_points() {
    let mut space = valid_space();
    space.access_points.clear();
    assert_eq!(space.validate(), Ok(()));
}

#[test]
fn rejects_invalid_dimensions_and_voxel_size() {
    for field in ["width", "depth", "height", "voxel_size"] {
        for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut space = valid_space();
            match field {
                "width" => space.width = value,
                "depth" => space.depth = value,
                "height" => space.height = value,
                "voxel_size" => space.voxel_size = value,
                _ => unreachable!(),
            }
            assert_eq!(
                space.validate(),
                Err(SpaceConfigError::InvalidSize { field })
            );
        }
    }
}

#[test]
fn rejects_non_finite_origins() {
    for axis in 0..3 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut space = valid_space();
            space.origin[axis] = value;
            assert_eq!(
                space.validate(),
                Err(SpaceConfigError::InvalidOrigin { axis })
            );
        }
    }
}

#[test]
fn rejects_unrepresentable_upper_boundaries() {
    let mut overflow = valid_space();
    overflow.origin[0] = f64::MAX;
    overflow.width = f64::MAX;
    assert_eq!(
        overflow.validate(),
        Err(SpaceConfigError::InvalidUpperBoundary { axis: 0 })
    );

    let mut rounded = valid_space();
    rounded.origin[0] = f64::MAX;
    rounded.width = 1.0;
    assert_eq!(
        rounded.validate(),
        Err(SpaceConfigError::InvalidUpperBoundary { axis: 0 })
    );
}

#[test]
fn rejects_outside_and_non_finite_access_points_on_each_axis() {
    for axis in 0..3 {
        let space = valid_space();
        let dimensions = [space.width, space.depth, space.height];
        let lower = space.origin[axis];
        for value in [
            lower - 0.1,
            lower + dimensions[axis],
            f64::NAN,
            f64::INFINITY,
        ] {
            let mut space = valid_space();
            space.access_points[1][axis] = value;
            assert_eq!(
                space.validate(),
                Err(SpaceConfigError::InvalidAccessPoint { index: 1, axis })
            );
        }
    }
}
