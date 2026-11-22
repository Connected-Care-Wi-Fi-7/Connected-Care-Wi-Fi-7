use thiserror::Error;

use crate::{CsiFrame, CsiSession};

/// The result of polling a CSI source successfully.
#[derive(Debug)]
pub enum CsiPoll {
    /// The next frame in the session.
    Frame(CsiFrame),
    /// A frame is not available yet and the stream remains open.
    Pending,
    /// The stream has ended. Later polls also return this outcome.
    EndOfStream,
}

/// An error while obtaining a CSI frame from the source.
#[derive(Debug, Error)]
pub enum CsiSourceError {
    /// The source could not provide a valid next frame.
    #[error("failed to read CSI frame: {source}")]
    ReadFailed {
        /// The underlying cause reported by the source implementation.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Supplies frames for one fixed CSI session.
pub trait CsiSource: Send {
    /// Returns the session metadata, which is available from construction onwards.
    fn session(&self) -> &CsiSession;

    /// Returns the next outcome without waiting for a future frame to arrive.
    ///
    /// Frames must match the session layout and use its capture timestamps.
    /// The first capture defines time zero, and replay preserves recorded offsets.
    ///
    /// # Errors
    ///
    /// Returns a [`CsiSourceError`] if the source fails to read or decode a frame
    /// or if a frame does not match the session layout.
    fn poll_frame(&mut self) -> Result<CsiPoll, CsiSourceError>;
}
