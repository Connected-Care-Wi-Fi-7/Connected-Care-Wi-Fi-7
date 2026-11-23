//! Pipeline bridge: moves `cc-core` output from pipeline threads into Bevy.
//!
//! A producer (today the fake source, later `cc-pipeline`) sends
//! [`PipelineOutput`] values through a bounded channel. Once per Bevy frame,
//! [`drain_pipeline`] takes everything waiting without blocking and exposes it
//! as resources (latest state) and messages (every alert). Other plugins read
//! only those, so they never deal with threads or channels.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use cc_core::{Alert, OccupancyFrame, Track};

/// How many outputs the channel holds before the producer starts dropping.
pub const CHANNEL_CAPACITY: usize = 256;

/// Upper bound on outputs handled per Bevy frame, so a flood cannot stall rendering.
const MAX_OUTPUTS_PER_FRAME: usize = 1024;

/// One item of pipeline output, as delivered to the viewer.
#[derive(Debug, Clone)]
pub enum PipelineOutput {
    /// A new occupancy estimate.
    Occupancy(OccupancyFrame),
    /// The current set of tracks, replacing the previous set.
    Tracks(Vec<Track>),
    /// A new alert.
    Alert(Alert),
}

/// Why a send to the viewer did not deliver its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    /// The channel was full, so the output was dropped and counted.
    Full,
    /// The viewer has closed; the producer should stop.
    Disconnected,
}

/// Producer end of the bridge. Cheap to clone and safe to use from any thread.
#[derive(Debug, Clone)]
pub struct PipelineSender {
    tx: SyncSender<PipelineOutput>,
    dropped: Arc<AtomicU64>,
}

impl PipelineSender {
    /// Sends without blocking. A full channel drops the output and counts it.
    ///
    /// # Errors
    ///
    /// Returns [`SendError::Full`] when the output was dropped, or
    /// [`SendError::Disconnected`] when the viewer has gone away.
    pub fn send(&self, output: PipelineOutput) -> Result<(), SendError> {
        match self.tx.try_send(output) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                Err(SendError::Full)
            }
            Err(TrySendError::Disconnected(_)) => Err(SendError::Disconnected),
        }
    }
}

/// Consumer end of the bridge, held by the Bevy app as a resource.
#[derive(Resource)]
pub struct PipelineReceiver {
    // `Receiver` is not `Sync`; the mutex makes it usable as a resource.
    rx: Mutex<Receiver<PipelineOutput>>,
    dropped: Arc<AtomicU64>,
}

/// Creates a connected sender and receiver with [`CHANNEL_CAPACITY`] slots.
pub fn channel() -> (PipelineSender, PipelineReceiver) {
    let (tx, rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
    let dropped = Arc::new(AtomicU64::new(0));
    (
        PipelineSender {
            tx,
            dropped: Arc::clone(&dropped),
        },
        PipelineReceiver {
            rx: Mutex::new(rx),
            dropped,
        },
    )
}

/// The most recent occupancy estimate, if any has arrived.
#[derive(Resource, Debug, Default)]
pub struct LatestOccupancy(pub Option<OccupancyFrame>);

/// The most recent set of tracks.
#[derive(Resource, Debug, Default)]
pub struct LatestTracks(pub Vec<Track>);

/// Sent once for every alert that arrives from the pipeline.
#[derive(Message, Debug, Clone)]
pub struct AlertReceived(pub Alert);

/// Counters describing the health of the bridge.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct BridgeStats {
    /// Outputs received from the pipeline since start-up.
    pub received: u64,
    /// Outputs dropped because the channel was full.
    pub dropped: u64,
    /// Whether a producer is still attached.
    pub connected: bool,
}

/// Registers the bridge's resources, messages and drain system.
///
/// The app must also insert a [`PipelineReceiver`]; without one the bridge
/// stays idle and reports itself as disconnected.
pub struct BridgePlugin;

impl Plugin for BridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LatestOccupancy>()
            .init_resource::<LatestTracks>()
            .init_resource::<BridgeStats>()
            .add_message::<AlertReceived>()
            .add_systems(PreUpdate, drain_pipeline);
    }
}

/// Moves all waiting pipeline output into Bevy resources and messages.
pub fn drain_pipeline(
    receiver: Option<Res<PipelineReceiver>>,
    mut stats: ResMut<BridgeStats>,
    mut occupancy: ResMut<LatestOccupancy>,
    mut tracks: ResMut<LatestTracks>,
    mut alerts: MessageWriter<AlertReceived>,
) {
    let Some(receiver) = receiver else {
        stats.connected = false;
        return;
    };
    stats.dropped = receiver.dropped.load(Ordering::Relaxed);
    let Ok(rx) = receiver.rx.lock() else {
        // Only possible if a panic occurred while the lock was held.
        stats.connected = false;
        return;
    };
    stats.connected = true;
    for _ in 0..MAX_OUTPUTS_PER_FRAME {
        match rx.try_recv() {
            Ok(output) => {
                stats.received += 1;
                match output {
                    PipelineOutput::Occupancy(frame) => occupancy.0 = Some(frame),
                    PipelineOutput::Tracks(latest) => tracks.0 = latest,
                    PipelineOutput::Alert(alert) => {
                        alerts.write(AlertReceived(alert));
                    }
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                stats.connected = false;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;
    use cc_core::{AlertKind, Position};

    fn test_app() -> (App, PipelineSender) {
        let mut app = App::new();
        app.add_plugins(BridgePlugin);
        let (tx, rx) = channel();
        app.insert_resource(rx);
        (app, tx)
    }

    fn track(id: u64) -> Track {
        Track {
            id,
            position: Position {
                x: 1.0,
                y: 2.0,
                z: None,
            },
            timestamp: 5,
        }
    }

    fn alert(id: u64) -> Alert {
        Alert {
            id,
            timestamp: 9,
            kind: AlertKind::PossibleFall,
            track_id: Some(1),
            zone_id: None,
        }
    }

    #[test]
    fn outputs_become_resources_and_messages() {
        let (mut app, tx) = test_app();
        tx.send(PipelineOutput::Tracks(vec![track(1)])).unwrap();
        tx.send(PipelineOutput::Tracks(vec![track(2), track(3)]))
            .unwrap();
        tx.send(PipelineOutput::Alert(alert(7))).unwrap();
        app.update();

        let stats = app.world().resource::<BridgeStats>();
        assert_eq!(stats.received, 3);
        assert!(stats.connected);

        let ids: Vec<u64> = app
            .world()
            .resource::<LatestTracks>()
            .0
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec![2, 3], "the newest track set replaces the old one");

        let messages = app.world().resource::<Messages<AlertReceived>>();
        let alerts: Vec<u64> = messages
            .iter_current_update_messages()
            .map(|m| m.0.id)
            .collect();
        assert_eq!(alerts, vec![7]);
    }

    #[test]
    fn full_channel_drops_and_counts() {
        let (mut app, tx) = test_app();
        for _ in 0..CHANNEL_CAPACITY {
            tx.send(PipelineOutput::Tracks(Vec::new())).unwrap();
        }
        assert_eq!(
            tx.send(PipelineOutput::Tracks(Vec::new())),
            Err(SendError::Full)
        );
        app.update();

        let stats = app.world().resource::<BridgeStats>();
        assert_eq!(stats.received, CHANNEL_CAPACITY as u64);
        assert_eq!(stats.dropped, 1);
    }

    #[test]
    fn dropped_producer_is_reported_as_disconnected() {
        let (mut app, tx) = test_app();
        drop(tx);
        app.update();
        assert!(!app.world().resource::<BridgeStats>().connected);
    }

    #[test]
    fn missing_receiver_leaves_bridge_idle() {
        let mut app = App::new();
        app.add_plugins(BridgePlugin);
        app.update();
        assert_eq!(
            *app.world().resource::<BridgeStats>(),
            BridgeStats::default()
        );
    }

    #[test]
    fn sending_after_viewer_closes_reports_disconnected() {
        let (tx, rx) = channel();
        drop(rx);
        assert_eq!(
            tx.send(PipelineOutput::Tracks(Vec::new())),
            Err(SendError::Disconnected)
        );
    }
}
