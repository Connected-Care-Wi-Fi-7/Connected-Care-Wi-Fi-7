use std::{fs, io, path::Path};

use serde::Deserialize;
use thiserror::Error;

/// Named implementation of the CSI source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsiSourceSelection {
    /// Selects the in-memory source for testing.
    Fake,
}

/// Named implementation of the fall classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallClassifierSelection {
    /// Selects the in-memory classifier for testing.
    Fake,
}

/// Named implementation of the auxiliary sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxSensorSelection {
    /// Selects the in-memory sensor for testing.
    Fake,
}

/// Named implementation of the alert sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSinkSelection {
    /// Selects the in-memory sink for testing.
    Fake,
}

/// Validated plug-in selections for the caller to resolve into implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginConfig {
    /// CSI source selection.
    pub csi_source: CsiSourceSelection,
    /// Fall classifier selection.
    pub fall_classifier: FallClassifierSelection,
    /// Auxiliary sensor selection.
    pub aux_sensor: AuxSensorSelection,
    /// Alert sink selection.
    pub alert_sink: AlertSinkSelection,
}

/// An error loading or validating a local plug-in configuration.
#[derive(Debug, Error)]
pub enum PluginConfigError {
    /// The local file could not be read as UTF-8.
    #[error("failed to read plug-in config: {0}")]
    Read(#[source] io::Error),
    /// The TOML syntax or structure is invalid.
    #[error("invalid plug-in config TOML: {0}")]
    Parse(#[from] toml::de::Error),
    /// A field names an unsupported implementation.
    #[error("unknown implementation {name:?} for plug-in config field {field:?}")]
    UnknownImplementation {
        /// Name of the field holding the rejected value.
        field: &'static str,
        /// Implementation name supplied in the file.
        name: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PluginNames {
    csi_source: String,
    fall_classifier: String,
    aux_sensor: String,
    alert_sink: String,
}

impl PluginConfig {
    /// Loads plug-in selections from a local TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, its TOML is invalid, or an
    /// implementation name is unsupported.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PluginConfigError> {
        let contents = fs::read_to_string(path).map_err(PluginConfigError::Read)?;
        Self::from_toml(&contents)
    }

    /// Parses TOML with four required top-level plug-in names.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed TOML, missing or unexpected fields, or
    /// unsupported implementation names.
    pub fn from_toml(contents: &str) -> Result<Self, PluginConfigError> {
        let names: PluginNames = toml::from_str(contents)?;

        Ok(Self {
            csi_source: select("csi_source", names.csi_source, CsiSourceSelection::Fake)?,
            fall_classifier: select(
                "fall_classifier",
                names.fall_classifier,
                FallClassifierSelection::Fake,
            )?,
            aux_sensor: select("aux_sensor", names.aux_sensor, AuxSensorSelection::Fake)?,
            alert_sink: select("alert_sink", names.alert_sink, AlertSinkSelection::Fake)?,
        })
    }
}

fn select<T>(field: &'static str, name: String, fake: T) -> Result<T, PluginConfigError> {
    match name.as_str() {
        "fake" => Ok(fake),
        _ => Err(PluginConfigError::UnknownImplementation { field, name }),
    }
}
