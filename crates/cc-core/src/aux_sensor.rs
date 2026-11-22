use thiserror::Error;

/// A provisional binary activation reading from a complementary sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuxReading {
    /// Observation time in nanoseconds since the first CSI capture of the session.
    pub captured_at_ns: u64,
    /// Whether the sensor observed activation at that time.
    pub active: bool,
}

/// An error while obtaining an auxiliary sensor reading.
#[derive(Debug, Error)]
pub enum AuxSensorError {
    /// The sensor could not provide a valid reading.
    #[error("failed to read auxiliary sensor: {source}")]
    ReadFailed {
        /// The underlying cause reported by the sensor implementation.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Supplies complementary sensor readings on the active CSI session timeline.
pub trait AuxSensor: Send {
    /// Polls for a new reading without waiting for a future reading to arrive.
    ///
    /// `Ok(None)` means there is no new reading. A reading with `active: false`
    /// represents observed inactivity. Implementations must convert timestamps
    /// from any separate sensor clock to the CSI session timeline.
    ///
    /// # Errors
    ///
    /// Returns an [`AuxSensorError`] if a reading cannot be obtained or its
    /// observation time cannot be represented on the session timeline.
    fn poll_reading(&mut self) -> Result<Option<AuxReading>, AuxSensorError>;
}
