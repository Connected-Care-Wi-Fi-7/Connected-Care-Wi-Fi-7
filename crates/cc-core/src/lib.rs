//! Shared data contracts for the Connected Care pipeline.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

mod csi_source;

pub use csi_source::{CsiPoll, CsiSource, CsiSourceError};

/// Fixed CSI configuration for one program run. Sensing supplies this at startup.
/// The first captured frame defines session time zero; replay preserves offsets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiSession {
    /// Tensor dimensions and subcarrier indices for this session.
    pub layout: CsiLayout,
    /// RF centre frequency in hertz.
    pub center_frequency_hz: u64,
    /// Channel bandwidth in hertz.
    pub channel_bandwidth_hz: u32,
    /// Spacing between adjacent OFDM tone indices in hertz.
    pub subcarrier_spacing_hz: u32,
    /// Linear amplitude units and source scaling.
    pub amplitude_convention: String,
}

/// Tensor axes ordered transmit stream, receive chain, then subcarrier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiLayout {
    /// Number of transmit streams with stable indices during the session.
    pub tx_count: usize,
    /// Number of receive chains with stable indices during the session.
    pub rx_count: usize,
    /// Actual OFDM tone indices in increasing frequency order; gaps are allowed.
    pub subcarrier_indices: Vec<i32>,
}

/// One complex CSI coefficient in the session's amplitude convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexCsi {
    /// Real component.
    pub real: f64,
    /// Imaginary component.
    pub imaginary: f64,
}

/// Contiguous tensor values ordered [tx, rx, subcarrier], subcarrier fastest.
///
/// `None` marks an unavailable value. Deserialized tensors must be checked
/// against their session layout with `validate_for` before use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiTensor<T> {
    values: Vec<Option<T>>,
}

impl<T> CsiTensor<T> {
    /// Constructs a tensor if its length matches the layout.
    pub fn new(layout: &CsiLayout, values: Vec<Option<T>>) -> Result<Self, &'static str> {
        let tensor = Self { values };
        tensor.validate_for(layout)?;
        Ok(tensor)
    }

    /// Checks tensor length against the supplied layout, including overflow.
    pub fn validate_for(&self, layout: &CsiLayout) -> Result<(), &'static str> {
        let expected = layout
            .tx_count
            .checked_mul(layout.rx_count)
            .and_then(|count| count.checked_mul(layout.subcarrier_indices.len()))
            .ok_or("CSI layout dimensions overflow")?;
        if self.values.len() != expected {
            return Err("CSI tensor length does not match layout");
        }
        Ok(())
    }

    /// Returns a value at checked tensor indices, or `None` for invalid indices.
    /// An inner `None` represents an unavailable measurement.
    pub fn get(
        &self,
        layout: &CsiLayout,
        tx: usize,
        rx: usize,
        subcarrier: usize,
    ) -> Option<&Option<T>> {
        self.validate_for(layout).ok()?;
        if tx >= layout.tx_count
            || rx >= layout.rx_count
            || subcarrier >= layout.subcarrier_indices.len()
        {
            return None;
        }
        let index = (tx * layout.rx_count + rx) * layout.subcarrier_indices.len() + subcarrier;
        self.values.get(index)
    }
}

/// CSI measurements from one capture instant in the active session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiFrame {
    /// Nanoseconds since the session time origin, preserved during replay.
    pub captured_at_ns: u64,
    /// Complex measurements interpreted using the active session layout.
    pub csi: CsiTensor<ComplexCsi>,
}

/// Processed signal and feature values at one tensor position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSample {
    /// Denoised linear amplitude before background subtraction.
    pub amplitude: f64,
    /// Corrected, unwrapped phase in radians, if available.
    pub phase_rad: Option<f64>,
    /// Signed amplitude residual after background subtraction.
    pub dynamic_amplitude: f64,
    /// Temporal variance; definition, units and window require agreement.
    pub variance: Option<f64>,
}

/// DSP output for one capture, using the active session layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedFrame {
    /// Capture time in nanoseconds since the session origin, not processing time.
    pub captured_at_ns: u64,
    /// Processed values; unavailable positions contain `None`.
    pub signals: CsiTensor<ProcessedSample>,
    /// Aggregate motion feature, if available; formula and units need agreement.
    pub motion_energy: Option<f64>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyGrid {
    /// Grid origin in metres, ordered x, y, z.
    pub origin: [f64; 3],
    /// Number of cells along x, y, and z respectively.
    pub dimensions: [u32; 3],
    /// Exact physical extent in metres along x, y, and z, before cell padding.
    pub extent: [f64; 3],
    /// Cell size in metres along x, y, and z respectively.
    pub voxel_size: [f64; 3],
    /// Cell values in x-fastest order, then y, then z.
    pub cells: Vec<OccupancyState>,
}

/// Occupancy estimates and detected centroids at one point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyFrame {
    /// Estimate-valid time in nanoseconds since the CSI session's first capture.
    pub timestamp: u64,
    /// Estimated occupancy grid.
    pub grid: OccupancyGrid,
    /// Estimated object centroids; these are not necessarily tracked IDs.
    pub centroids: Vec<Position>,
}

/// A position in the proposed coordinate frame, measured in metres.
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
    /// Estimate-valid time in nanoseconds since the CSI session's first capture.
    pub timestamp: u64,
}

/// A possible fall detected by the classifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallEvent {
    /// Estimated fall time in nanoseconds since the CSI session's first capture.
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
    /// Alert creation time in nanoseconds since the CSI session's first capture.
    pub timestamp: u64,
    /// Reason the alert was raised.
    pub kind: AlertKind,
    /// Associated track, if known.
    pub track_id: Option<u64>,
    /// Associated zone, if applicable.
    pub zone_id: Option<u64>,
}
