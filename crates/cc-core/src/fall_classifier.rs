use thiserror::Error;

use crate::{FallEvent, ProcessedFrame, Track};

/// An error while classifying a fall detection update.
#[derive(Debug, Error)]
pub enum FallClassifierError {
    /// The classifier could not process the update.
    #[error("failed to classify fall update: {source}")]
    ClassificationFailed {
        /// The underlying cause reported by the classifier implementation.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Detects possible falls from processed CSI and the current track estimates.
pub trait FallClassifier: Send {
    /// Processes one update from the active CSI session and returns the falls
    /// detected by this update. An empty vector means no fall was detected.
    ///
    /// Implementations may retain previous updates for a rolling window. Event
    /// times use the session timeline and a track association may be absent.
    ///
    /// # Errors
    ///
    /// Returns a [`FallClassifierError`] if the update could not be processed.
    fn classify(
        &mut self,
        frame: &ProcessedFrame,
        tracks: &[Track],
    ) -> Result<Vec<FallEvent>, FallClassifierError>;
}
