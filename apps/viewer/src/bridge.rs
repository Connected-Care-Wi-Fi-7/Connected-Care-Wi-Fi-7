//! Pipeline bridge: moves `cc-core` output from pipeline threads into Bevy.
//!
//! A producer (today the fake source, later `cc-pipeline`) sends
//! [`PipelineOutput`] values through two lanes:
//!
//! - **State** (occupancy and tracks) uses a bounded channel. Only the newest
//!   value matters, so when it is full the update is dropped and counted.
//! - **Alerts** use their own unbounded channel, so an alert is never dropped,
//!   however busy the state lane is. Alerts are rare, so it stays small.
//!
//! Once per Bevy frame, [`drain_pipeline`] takes everything waiting without
//! blocking and exposes it as resources (latest state) and messages (every
//! alert). Other plugins read only those, so they never deal with threads or
//! channels.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use cc_core::{Alert, OccupancyFrame, Track};

/// How many state updates the channel holds before the producer starts dropping.
pub const CHANNEL_CAPACITY: usize = 256;

/// Upper bound on items handled per lane per Bevy frame, so a flood cannot
/// stall rendering. Anything left over waits for the next frame.
const MAX_OUTPUTS_PER_FRAME: usize = 1024;

/// One item of pipeline output, as delivered to the viewer.
#[derive(Debug, Clone)]
pub enum PipelineOutput {
    /// A new occupancy estimate.
    Occupancy(OccupancyFrame),
    /// The current set of tracks, replacing the previous set.
    Tracks(Vec<Track>),
    /// A new alert. Never dropped by the bridge.
    Alert(Alert),
}

/// Why a send to the viewer did not deliver its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendError {
    /// The state channel was full, so the update was dropped and counted.
    /// Never returned for alerts.
    Full,
    /// The viewer has closed; the producer should stop.
    Disconnected,
}

/// Producer end of the bridge. Cheap to clone and safe to use from any thread.
#[derive(Debug, Clone)]
pub struct PipelineSender {
    state: SyncSender<PipelineOutput>,
    alerts: Sender<Alert>,
    dropped: Arc<AtomicU64>,
}

impl PipelineSender {
    /// Sends without blocking. Alerts always go through; a state update is
    /// dropped and counted if the state channel is full.
    ///
    /// # Errors
    ///
    /// Returns [`SendError::Full`] when a state update was dropped, or
    /// [`SendError::Disconnected`] when the viewer has gone away.
    pub fn send(&self, output: PipelineOutput) -> Result<(), SendError> {
        if let PipelineOutput::Alert(alert) = output {
            return self.alerts.send(alert).map_err(|_| SendError::Disconnected);
        }
        match self.state.try_send(output) {
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
    // `Receiver` is not `Sync`; the mutexes make it usable as a resource.
    state: Mutex<Receiver<PipelineOutput>>,
    alerts: Mutex<Receiver<Alert>>,
    dropped: Arc<AtomicU64>,
}

/// Creates a connected sender and receiver. The state lane has
/// [`CHANNEL_CAPACITY`] slots; the alert lane is unbounded.
pub fn channel() -> (PipelineSender, PipelineReceiver) {
    let (state_tx, state_rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
    let (alert_tx, alert_rx) = mpsc::channel();
    let dropped = Arc::new(AtomicU64::new(0));
    (
        PipelineSender {
            state: state_tx,
            alerts: alert_tx,
            dropped: Arc::clone(&dropped),
        },
        PipelineReceiver {
            state: Mutex::new(state_rx),
            alerts: Mutex::new(alert_rx),
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
    /// Outputs (state updates and alerts) received since start-up.
    pub received: u64,
    /// State updates dropped because the channel was full. Alerts are never dropped.
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
    let (Ok(state_rx), Ok(alert_rx)) = (receiver.state.lock(), receiver.alerts.lock()) else {
        // Only possible if a panic occurred while a lock was held.
        stats.connected = false;
        return;
    };
    let mut connected = true;

    for _ in 0..MAX_OUTPUTS_PER_FRAME {
        match state_rx.try_recv() {
            Ok(output) => {
                stats.received += 1;
                match output {
                    PipelineOutput::Occupancy(frame) => occupancy.0 = Some(frame),
                    PipelineOutput::Tracks(latest) => tracks.0 = latest,
                    // `send` routes alerts to their own lane; handle one here anyway.
                    PipelineOutput::Alert(alert) => {
                        alerts.write(AlertReceived(alert));
                    }
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                connected = false;
                break;
            }
        }
    }

    for _ in 0..MAX_OUTPUTS_PER_FRAME {
        match alert_rx.try_recv() {
            Ok(alert) => {
                stats.received += 1;
                alerts.write(AlertReceived(alert));
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                connected = false;
                break;
            }
        }
    }

    stats.connected = connected;
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
    fn alert_gets_through_when_state_channel_is_full() {
        let (mut app, tx) = test_app();
        for _ in 0..CHANNEL_CAPACITY {
            tx.send(PipelineOutput::Tracks(vec![track(1)])).unwrap();
        }
        assert_eq!(
            tx.send(PipelineOutput::Tracks(Vec::new())),
            Err(SendError::Full),
            "state channel should be full"
        );
        tx.send(PipelineOutput::Alert(alert(42))).unwrap();
        app.update();

        let messages = app.world().resource::<Messages<AlertReceived>>();
        let received: Vec<_> = messages.iter_current_update_messages().collect();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].0.id, 42);
        assert!(matches!(received[0].0.kind, AlertKind::PossibleFall));
        assert_eq!(app.world().resource::<BridgeStats>().dropped, 1);
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
