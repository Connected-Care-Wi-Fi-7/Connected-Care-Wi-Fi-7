//! Spatial mapping and tracking for the Connected Care pipeline.

#![forbid(unsafe_code)]

mod config;

pub use config::{DEFAULT_MAX_GRID_BYTES, SpaceConfig, SpaceConfigError, SpaceConfigLoadError};
