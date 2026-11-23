//! Fake pipeline producer used until `cc-pipeline` exists.
//!
//! A background thread sends a scripted scene through the bridge: one person
//! walking in a circle in a 4 m x 3 m x 2.5 m room, with a zone-entry alert
//! every few seconds. All data is synthetic.

use std::f64::consts::TAU;
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use cc_core::{Alert, AlertKind, OccupancyFrame, OccupancyGrid, OccupancyState, Position, Track};

use crate::bridge::{self, PipelineOutput, PipelineSender, SendError};

/// Output rate of the fake source in frames per second.
pub const FRAME_RATE_HZ: u64 = 10;
/// Room size in metres along x, y and z.
const ROOM_EXTENT: [f64; 3] = [4.0, 3.0, 2.5];
/// Voxel edge length in metres.
const VOXEL_SIZE: f64 = 0.5;
/// Frames per full lap of the walking circle.
const FRAMES_PER_LAP: u64 = 100;
/// A zone-entry alert is raised every this many frames.
const ALERT_EVERY: u64 = 50;
/// The single fake person's track ID.
const TRACK_ID: u64 = 1;

/// Starts the fake producer thread and connects it to the bridge.
pub struct FakeSourcePlugin;

impl Plugin for FakeSourcePlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = bridge::channel();
        let spawned = thread::Builder::new()
            .name("fake-pipeline".into())
            .spawn(move || run(&tx));
        match spawned {
            Ok(_) => {
                app.insert_resource(rx);
            }
            // Without the receiver the bridge shows "disconnected" instead of crashing.
            Err(err) => error!("could not start fake pipeline thread: {err}"),
        }
    }
}

/// Sends frames until the viewer closes its end of the channel.
fn run(tx: &PipelineSender) {
    let period = Duration::from_nanos(1_000_000_000 / FRAME_RATE_HZ);
    for tick in 0.. {
        for output in frame_outputs(tick) {
            // A full channel just drops this output; the bridge counts it.
            if tx.send(output) == Err(SendError::Disconnected) {
                return;
            }
        }
        thread::sleep(period);
    }
}

/// Builds the outputs for one tick. Timestamps start at zero, as in a CSI session.
pub fn frame_outputs(tick: u64) -> Vec<PipelineOutput> {
    let timestamp = tick * (1_000_000_000 / FRAME_RATE_HZ);
    let position = walking_position(tick);

    let mut outputs = vec![
        PipelineOutput::Occupancy(OccupancyFrame {
            timestamp,
            grid: grid_with_person(&position),
            centroids: vec![position.clone()],
        }),
        PipelineOutput::Tracks(vec![Track {
            id: TRACK_ID,
            position,
            timestamp,
        }]),
    ];
    if tick > 0 && tick.is_multiple_of(ALERT_EVERY) {
        outputs.push(PipelineOutput::Alert(Alert {
            id: tick / ALERT_EVERY,
            timestamp,
            kind: AlertKind::ZoneEntry,
            track_id: Some(TRACK_ID),
            zone_id: Some(1),
        }));
    }
    outputs
}

/// Position on a circle centred in the room, at standing centroid height.
fn walking_position(tick: u64) -> Position {
    let angle = TAU * (tick % FRAMES_PER_LAP) as f64 / FRAMES_PER_LAP as f64;
    Position {
        x: ROOM_EXTENT[0] / 2.0 + 1.0 * angle.cos(),
        y: ROOM_EXTENT[1] / 2.0 + 1.0 * angle.sin(),
        z: Some(1.0),
    }
}

/// An empty grid covering the room, with the person's column marked occupied.
fn grid_with_person(person: &Position) -> OccupancyGrid {
    let dims = ROOM_EXTENT.map(|e| (e / VOXEL_SIZE).ceil() as u32);
    let mut cells = vec![OccupancyState::Empty; (dims[0] * dims[1] * dims[2]) as usize];
    let cx = (person.x / VOXEL_SIZE) as u32;
    let cy = (person.y / VOXEL_SIZE) as u32;
    if cx < dims[0] && cy < dims[1] {
        // A standing person fills the lower 2 m of their column.
        let height_cells = (2.0 / VOXEL_SIZE) as u32;
        for z in 0..height_cells.min(dims[2]) {
            let index = cx + dims[0] * (cy + dims[1] * z);
            cells[index as usize] = OccupancyState::Occupied;
        }
    }
    OccupancyGrid {
        origin: [0.0; 3],
        dimensions: dims,
        extent: ROOM_EXTENT,
        voxel_size: [VOXEL_SIZE; 3],
        cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_matches_its_dimensions_and_marks_the_person() {
        let Some(PipelineOutput::Occupancy(frame)) = frame_outputs(0).into_iter().next() else {
            panic!("first output should be occupancy");
        };
        let [x, y, z] = frame.grid.dimensions;
        assert_eq!([x, y, z], [8, 6, 5]);
        assert_eq!(frame.grid.cells.len(), (x * y * z) as usize);
        let occupied = frame
            .grid
            .cells
            .iter()
            .filter(|c| matches!(c, OccupancyState::Occupied))
            .count();
        assert_eq!(occupied, 4, "one column of 2 m at 0.5 m voxels");
    }

    #[test]
    fn person_stays_inside_the_room() {
        for tick in 0..FRAMES_PER_LAP {
            let p = walking_position(tick);
            assert!((0.0..ROOM_EXTENT[0]).contains(&p.x));
            assert!((0.0..ROOM_EXTENT[1]).contains(&p.y));
        }
    }

    #[test]
    fn alerts_arrive_on_schedule_with_session_timestamps() {
        let has_alert = |tick| {
            frame_outputs(tick)
                .iter()
                .any(|o| matches!(o, PipelineOutput::Alert(_)))
        };
        assert!(!has_alert(0));
        assert!(has_alert(ALERT_EVERY));
        assert!(!has_alert(ALERT_EVERY + 1));

        let Some(PipelineOutput::Tracks(tracks)) = frame_outputs(FRAME_RATE_HZ).into_iter().nth(1)
        else {
            panic!("second output should be tracks");
        };
        assert_eq!(tracks[0].timestamp, 1_000_000_000, "one second in");
    }
}
