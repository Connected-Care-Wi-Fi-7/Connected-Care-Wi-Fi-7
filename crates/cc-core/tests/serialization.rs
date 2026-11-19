use cc_core::{
    Alert, AlertKind, ComplexCsi, CsiFrame, CsiLayout, CsiSession, CsiTensor, FallEvent,
    OccupancyFrame, OccupancyGrid, OccupancyState, Position, ProcessedFrame, ProcessedSample,
    Track,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn assert_round_trip<T: Serialize + DeserializeOwned>(value: &T, expected: Value) {
    let encoded = serde_json::to_string(value).expect("serialize test value");
    let decoded: T = serde_json::from_str(&encoded).expect("deserialize test value");
    assert_eq!(serde_json::to_value(value).unwrap(), expected);
    assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
}

fn layout() -> CsiLayout {
    CsiLayout {
        tx_count: 1,
        rx_count: 1,
        subcarrier_indices: vec![-3, 2],
    }
}

#[test]
fn csi_session_round_trip() {
    let session = CsiSession {
        layout: layout(),
        center_frequency_hz: 5_000_000_000,
        channel_bandwidth_hz: 20_000_000,
        subcarrier_spacing_hz: 312_500,
        amplitude_convention: "source-scaled linear".into(),
    };
    assert_round_trip(
        &session,
        json!({
            "layout": {"tx_count": 1, "rx_count": 1, "subcarrier_indices": [-3, 2]},
            "center_frequency_hz": 5_000_000_000u64, "channel_bandwidth_hz": 20_000_000,
            "subcarrier_spacing_hz": 312_500, "amplitude_convention": "source-scaled linear"
        }),
    );
}

#[test]
fn csi_frame_round_trip() {
    let frame = CsiFrame {
        captured_at_ns: 12,
        csi: CsiTensor::new(
            &layout(),
            vec![
                Some(ComplexCsi {
                    real: 1.25,
                    imaginary: -0.5,
                }),
                None,
            ],
        )
        .unwrap(),
    };
    assert_round_trip(
        &frame,
        json!({"captured_at_ns": 12,
        "csi": {"values": [{"real": 1.25, "imaginary": -0.5}, null]}}),
    );
}

#[test]
fn processed_frame_round_trip() {
    let frame = ProcessedFrame {
        captured_at_ns: 13,
        signals: CsiTensor::new(
            &layout(),
            vec![
                Some(ProcessedSample {
                    amplitude: 1.25,
                    phase_rad: Some(7.0),
                    dynamic_amplitude: -0.5,
                    variance: None,
                }),
                None,
            ],
        )
        .unwrap(),
        motion_energy: Some(2.0),
    };
    assert_round_trip(
        &frame,
        json!({"captured_at_ns": 13,
        "signals": {"values": [{"amplitude": 1.25, "phase_rad": 7.0,
            "dynamic_amplitude": -0.5, "variance": null}, null]}, "motion_energy": 2.0}),
    );
}

#[test]
fn tensor_validates_length_and_indices() {
    let dimensions = CsiLayout {
        tx_count: 2,
        rx_count: 2,
        subcarrier_indices: vec![-1, 1],
    };
    assert!(CsiTensor::new(&dimensions, vec![Some(0); 7]).is_err());
    let tensor = CsiTensor::new(
        &dimensions,
        vec![
            Some(0),
            None,
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            Some(7),
        ],
    )
    .unwrap();
    assert_eq!(tensor.get(&dimensions, 0, 0, 1), Some(&None));
    assert_eq!(tensor.get(&dimensions, 1, 0, 0), Some(&Some(4)));
    assert_eq!(tensor.get(&dimensions, 2, 0, 0), None);
    assert_eq!(tensor.get(&dimensions, 0, 2, 0), None);
    assert_eq!(tensor.get(&dimensions, 0, 0, 2), None);
}

#[test]
fn deserialized_tensor_must_be_checked_against_layout() {
    let tensor: CsiTensor<i32> = serde_json::from_str(r#"{"values":[1]}"#).unwrap();
    assert!(tensor.validate_for(&layout()).is_err());
    assert_eq!(tensor.get(&layout(), 0, 0, 0), None);
    let overflow = CsiLayout {
        tx_count: usize::MAX,
        rx_count: 2,
        subcarrier_indices: vec![0],
    };
    assert!(tensor.validate_for(&overflow).is_err());
}

#[test]
fn occupancy_frame_round_trip() {
    let frame = OccupancyFrame {
        timestamp: 14,
        grid: OccupancyGrid {
            origin: [0.0, 1.0, 2.0],
            dimensions: [3, 1, 1],
            extent: [1.25, 0.5, 0.5],
            voxel_size: [0.5, 0.5, 0.5],
            cells: vec![
                OccupancyState::Unknown,
                OccupancyState::Empty,
                OccupancyState::Occupied,
            ],
        },
        centroids: vec![Position {
            x: 1.0,
            y: 2.0,
            z: None,
        }],
    };
    assert_round_trip(
        &frame,
        json!({
            "timestamp": 14,
            "grid": {"origin": [0.0, 1.0, 2.0], "dimensions": [3, 1, 1], "extent": [1.25, 0.5, 0.5],
                "voxel_size": [0.5, 0.5, 0.5], "cells": ["Unknown", "Empty", "Occupied"]},
            "centroids": [{"x": 1.0, "y": 2.0, "z": null}]
        }),
    );
}

#[test]
fn track_round_trip() {
    let track = Track {
        id: 7,
        position: Position {
            x: 1.0,
            y: 2.0,
            z: Some(1.5),
        },
        timestamp: 15,
    };
    assert_round_trip(
        &track,
        json!({
            "id": 7, "position": {"x": 1.0, "y": 2.0, "z": 1.5}, "timestamp": 15
        }),
    );
}

#[test]
fn fall_event_round_trip() {
    for track_id in [Some(7), None] {
        let event = FallEvent {
            timestamp: 16,
            track_id,
        };
        assert_round_trip(&event, json!({"timestamp": 16, "track_id": track_id}));
    }
}

#[test]
fn alert_round_trip() {
    let fall = Alert {
        id: 42,
        timestamp: 17,
        kind: AlertKind::PossibleFall,
        track_id: Some(7),
        zone_id: None,
    };
    assert_round_trip(
        &fall,
        json!({
            "id": 42, "timestamp": 17, "kind": "PossibleFall", "track_id": 7, "zone_id": null
        }),
    );
    let zone = Alert {
        id: 43,
        timestamp: 18,
        kind: AlertKind::ZoneEntry,
        track_id: None,
        zone_id: Some(3),
    };
    assert_round_trip(
        &zone,
        json!({
            "id": 43, "timestamp": 18, "kind": "ZoneEntry", "track_id": null, "zone_id": 3
        }),
    );
}
