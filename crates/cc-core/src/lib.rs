//! Shared data contracts for the Connected Care pipeline.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A complex signal measurement. Its units and calibration need review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexMeasurement {
    /// Real component.
    pub real: f64,
    /// Imaginary component.
    pub imaginary: f64,
}

/// One measurement with provisional antenna and subcarrier indices.
///
/// The BE200 investigation must verify these indices and required metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiMeasurement {
    /// Transmitting antenna index.
    pub transmit_antenna: u32,
    /// Receiving antenna index.
    pub receive_antenna: u32,
    /// Subcarrier index; numbering convention is not yet agreed.
    pub subcarrier: i32,
    /// Complex CSI value.
    pub value: ComplexMeasurement,
}

/// A set of CSI measurements captured at one time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiFrame {
    /// Capture time; units and epoch are provisional.
    pub timestamp: u64,
    /// Source identifier; its allocation convention is provisional.
    pub source_id: u64,
    /// Captured measurements; format and metadata require hardware review.
    pub measurements: Vec<CsiMeasurement>,
}

/// Signal-processing output for mapping.
///
/// Preserving complex CSI values is provisional: DSP may need a different
/// representation or a time window rather than one capture timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedFrame {
    /// Capture time of the underlying data; units and epoch are provisional.
    pub timestamp: u64,
    /// Identifier of the source of the underlying data.
    pub source_id: u64,
    /// Processed measurements; shape and meaning require DSP review.
    pub measurements: Vec<CsiMeasurement>,
}

/// Provisional occupancy state; a continuous estimate may be needed instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OccupancyState {
    /// The cell has not been observed sufficiently to classify it.
    Unknown,
    /// The cell is estimated to be empty.
    Empty,
    /// The cell is estimated to be occupied.
    Occupied,
}

/// A grid of occupancy states in the project's coordinate frame.
///
/// Grid ordering, edge cells, and whether this covers multiple rooms need review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyGrid {
    /// Grid origin in metres, ordered x, y, z.
    pub origin: [f64; 3],
    /// Number of cells along x, y, and z respectively.
    pub dimensions: [u32; 3],
    /// Cell size in metres along x, y, and z respectively.
    pub voxel_size: [f64; 3],
    /// Cell values in x-fastest order, then y, then z.
    pub cells: Vec<OccupancyState>,
}

/// Occupancy estimates and detected centroids at one point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyFrame {
    /// Time when the estimate applies; units and epoch are provisional.
    pub timestamp: u64,
    /// Estimated occupancy grid.
    pub grid: OccupancyGrid,
    /// Estimated object centroids; these are not necessarily tracked IDs.
    pub centroids: Vec<Position>,
}

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
