//! 2D user interface drawn with egui (see ADR 0004).
//!
//! For now this is a status panel with the live frame counter. The alert
//! panel (#30) and zone editor (#29) add their own panels here.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::bridge::{AlertReceived, BridgeStats, LatestOccupancy, LatestTracks};

/// Draws the viewer's panels.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<AlertCount>()
            .add_systems(Update, count_alerts)
            .add_systems(EguiPrimaryContextPass, status_panel);
    }
}

/// Alerts received since start-up: how many, and the most recent one.
#[derive(Resource, Debug, Default)]
struct AlertCount {
    total: u64,
    last: Option<String>,
}

fn count_alerts(mut alerts: MessageReader<AlertReceived>, mut count: ResMut<AlertCount>) {
    for AlertReceived(alert) in alerts.read() {
        count.total += 1;
        count.last = Some(format!("{:?} at {}", alert.kind, seconds(alert.timestamp)));
    }
}

/// Formats a session timestamp in nanoseconds as seconds.
fn seconds(timestamp_ns: u64) -> String {
    format!("{:.1} s", timestamp_ns as f64 / 1e9)
}

fn status_panel(
    mut contexts: EguiContexts,
    stats: Res<BridgeStats>,
    occupancy: Res<LatestOccupancy>,
    tracks: Res<LatestTracks>,
    alerts: Res<AlertCount>,
) -> Result {
    let session_time = occupancy
        .0
        .as_ref()
        .map_or("-".to_owned(), |f| seconds(f.timestamp));

    egui::Window::new("Pipeline").show(contexts.ctx_mut()?, |ui| {
        ui.label(if stats.connected {
            "Source: connected"
        } else {
            "Source: disconnected"
        });
        ui.label(format!("Frames received: {}", stats.received));
        ui.label(format!("Frames dropped: {}", stats.dropped));
        ui.label(format!("Session time: {session_time}"));
        ui.label(format!("Tracks: {}", tracks.0.len()));
        ui.label(format!("Alerts: {}", alerts.total));
        ui.label(format!(
            "Last alert: {}",
            alerts.last.as_deref().unwrap_or("-")
        ));
    });
    Ok(())
}
