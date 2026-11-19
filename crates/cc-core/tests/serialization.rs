use cc_core::{
    Alert, AlertKind, ComplexMeasurement, CsiFrame, CsiMeasurement, FallEvent, OccupancyFrame,
    OccupancyGrid, OccupancyState, Position, ProcessedFrame, Track,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn assert_round_trip<T: Serialize + DeserializeOwned>(value: &T, expected: Value) {
    let encoded = serde_json::to_string(value).expect("serialize test value");
    let decoded: T = serde_json::from_str(&encoded).expect("deserialize test value");
    assert_eq!(serde_json::to_value(value).unwrap(), expected);
    assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
}

fn measurement() -> CsiMeasurement {
    CsiMeasurement {
        transmit_antenna: 1,
        receive_antenna: 2,
        subcarrier: -3,
        value: ComplexMeasurement {
            real: 1.25,
            imaginary: -0.5,
        },
    }
}

#[test]
fn csi_frame_round_trip() {
    let frame = CsiFrame {
        timestamp: 12,
        source_id: 4,
        measurements: vec![measurement()],
    };
    assert_round_trip(&frame, json!({
        "timestamp": 12, "source_id": 4,
        "measurements": [{"transmit_antenna": 1, "receive_antenna": 2,
            "subcarrier": -3, "value": {"real": 1.25, "imaginary": -0.5}}]
    }));
}

#[test]
fn processed_frame_round_trip() {
    let frame = ProcessedFrame {
        timestamp: 13,
        source_id: 5,
        measurements: vec![measurement()],
    };
    assert_round_trip(&frame, json!({
        "timestamp": 13, "source_id": 5,
        "measurements": [{"transmit_antenna": 1, "receive_antenna": 2,
            "subcarrier": -3, "value": {"real": 1.25, "imaginary": -0.5}}]
    }));
}

#[test]
fn occupancy_frame_round_trip() {
    let frame = OccupancyFrame {
        timestamp: 14,
        grid: OccupancyGrid {
            origin: [0.0, 1.0, 2.0],
            dimensions: [3, 1, 1],
            voxel_size: [0.5, 0.5, 0.5],
            cells: vec![OccupancyState::Unknown, OccupancyState::Empty, OccupancyState::Occupied],
        },
        centroids: vec![Position { x: 1.0, y: 2.0, z: None }],
    };
    assert_round_trip(&frame, json!({
        "timestamp": 14,
        "grid": {"origin": [0.0, 1.0, 2.0], "dimensions": [3, 1, 1],
            "voxel_size": [0.5, 0.5, 0.5], "cells": ["Unknown", "Empty", "Occupied"]},
        "centroids": [{"x": 1.0, "y": 2.0, "z": null}]
    }));
}

#[test]
fn track_round_trip() {
    let track = Track {
        id: 7,
        position: Position { x: 1.0, y: 2.0, z: Some(1.5) },
        timestamp: 15,
    };
    assert_round_trip(&track, json!({
        "id": 7, "position": {"x": 1.0, "y": 2.0, "z": 1.5}, "timestamp": 15
    }));
}

#[test]
fn fall_event_round_trip() {
    for track_id in [Some(7), None] {
        let event = FallEvent { timestamp: 16, track_id };
        assert_round_trip(&event, json!({"timestamp": 16, "track_id": track_id}));
    }
}

#[test]
fn alert_round_trip() {
    let fall = Alert {
        id: 42, timestamp: 17, kind: AlertKind::PossibleFall,
        track_id: Some(7), zone_id: None,
    };
    assert_round_trip(&fall, json!({
        "id": 42, "timestamp": 17, "kind": "PossibleFall", "track_id": 7, "zone_id": null
    }));
    let zone = Alert {
        id: 43, timestamp: 18, kind: AlertKind::ZoneEntry,
        track_id: None, zone_id: Some(3),
    };
    assert_round_trip(&zone, json!({
        "id": 43, "timestamp": 18, "kind": "ZoneEntry", "track_id": null, "zone_id": 3
    }));
}
