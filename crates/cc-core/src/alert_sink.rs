use thiserror::Error;

use crate::Alert;

/// An error while publishing an alert to a sink.
#[derive(Debug, Error)]
pub enum AlertSinkError {
    /// The sink could not confirm acceptance of the alert.
    #[error("failed to publish alert: {source}")]
    PublishFailed {
        /// The underlying cause reported by the sink implementation.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Accepts alerts for delivery to a destination.
pub trait AlertSink: Send {
    /// Publishes an alert to this sink.
    ///
    /// `Ok(())` means the sink accepted the alert according to its documented
    /// contract, such as appending it to a log or queuing it for the dashboard.
    /// User acknowledgement is handled separately.
    ///
    /// # Errors
    ///
    /// Returns an [`AlertSinkError`] if the sink cannot confirm acceptance.
    fn publish(&mut self, alert: &Alert) -> Result<(), AlertSinkError>;
}
