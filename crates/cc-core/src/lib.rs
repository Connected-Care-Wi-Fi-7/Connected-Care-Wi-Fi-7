//! Shared data contracts for the Connected Care pipeline.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A position in the agreed coordinate frame, measured in metres.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Horizontal position along the x-axis in metres.
    pub x: f64,
    /// Horizontal position along the y-axis in metres.
    pub y: f64,
    /// Height in metres, or `None` when unknown.
    pub z: Option<f64>,
}

/// The latest estimated state of a tracked object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Temporary track identifier, not a person's identity.
    pub id: u64,
    /// Estimated centroid position.
    pub position: Position,
    /// Time when the position estimate was valid; units and epoch are provisional.
    pub timestamp: u64,
}

/// A possible fall detected by the classifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallEvent {
    /// Estimated event time; units and epoch are provisional.
    pub timestamp: u64,
    /// Associated track, if known.
    pub track_id: Option<u64>,
}

/// The reason an alert was raised.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertKind {
    /// A possible fall was detected.
    PossibleFall,
    /// An object entered a configured zone.
    ZoneEntry,
}

/// An event presented to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Identifier for this alert.
    pub id: u64,
    /// Alert creation time; units and epoch are provisional.
    pub timestamp: u64,
    /// Reason the alert was raised.
    pub kind: AlertKind,
    /// Associated track, if known.
    pub track_id: Option<u64>,
    /// Associated zone, if applicable.
    pub zone_id: Option<u64>,
}
