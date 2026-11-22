use std::{collections::VecDeque, path::Path};

use cc_core::{
    Alert, AlertKind, AlertSink, AlertSinkSelection, AuxReading, AuxSensor, AuxSensorSelection,
    ComplexCsi, CsiFrame, CsiPoll, CsiSource, CsiSourceSelection, CsiTensor, FallClassifier,
    FallClassifierSelection, FallEvent, PluginConfig, PluginConfigError, ProcessedFrame,
};

#[path = "plugin_config/alert_sink.rs"]
mod alert_sink;
#[path = "plugin_config/aux_sensor.rs"]
mod aux_sensor;
#[path = "plugin_config/csi_source.rs"]
mod csi_source;
#[path = "plugin_config/fall_classifier.rs"]
mod fall_classifier;

const FAKE_CONFIG: &str = include_str!("../../../config/plugins.example.toml");

#[test]
fn parses_all_four_selections() {
    assert_eq!(
        PluginConfig::from_toml(FAKE_CONFIG).unwrap(),
        PluginConfig {
            csi_source: CsiSourceSelection::Fake,
            fall_classifier: FallClassifierSelection::Fake,
            aux_sensor: AuxSensorSelection::Fake,
            alert_sink: AlertSinkSelection::Fake,
        }
    );
}

#[test]
fn loads_and_uses_all_four_fake_plugins() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/plugins.example.toml");
    let config = PluginConfig::load(path).unwrap();

    let mut source: Box<dyn CsiSource> = match config.csi_source {
        CsiSourceSelection::Fake => {
            let session = csi_source::session();
            let frame = CsiFrame {
                captured_at_ns: 0,
                csi: CsiTensor::new(
                    &session.layout,
                    vec![Some(ComplexCsi {
                        real: 1.0,
                        imaginary: 0.0,
                    })],
                )
                .unwrap(),
            };
            Box::new(csi_source::FakeCsiSource {
                session,
                outcomes: VecDeque::from([Ok(CsiPoll::Frame(frame))]),
            })
        }
    };
    let mut classifier: Box<dyn FallClassifier> = match config.fall_classifier {
        FallClassifierSelection::Fake => Box::new(fall_classifier::FakeFallClassifier {
            outcomes: VecDeque::from([Ok(vec![FallEvent {
                timestamp: 0,
                track_id: None,
            }])]),
        }),
    };
    let mut sensor: Box<dyn AuxSensor> = match config.aux_sensor {
        AuxSensorSelection::Fake => Box::new(aux_sensor::FakeAuxSensor {
            outcomes: VecDeque::from([Ok(Some(AuxReading {
                captured_at_ns: 0,
                active: true,
            }))]),
        }),
    };
    let mut sink = match config.alert_sink {
        AlertSinkSelection::Fake => alert_sink::FakeAlertSink::default(),
    };

    let CsiPoll::Frame(frame) = source.poll_frame().unwrap() else {
        panic!("expected a frame from the configured CSI source");
    };
    assert_eq!(frame.captured_at_ns, 0);
    let processed = ProcessedFrame {
        captured_at_ns: frame.captured_at_ns,
        signals: CsiTensor::new(&source.session().layout, vec![None]).unwrap(),
        motion_energy: None,
    };
    let events = classifier.classify(&processed, &[]).unwrap();
    let [event] = events.as_slice() else {
        panic!("expected one scripted fall from the configured classifier");
    };
    assert_eq!(event.timestamp, frame.captured_at_ns);
    assert_eq!(
        sensor.poll_reading().unwrap(),
        Some(AuxReading {
            captured_at_ns: frame.captured_at_ns,
            active: true,
        })
    );
    let publisher: &mut dyn AlertSink = &mut sink;
    publisher
        .publish(&Alert {
            id: 1,
            timestamp: event.timestamp,
            kind: AlertKind::PossibleFall,
            track_id: event.track_id,
            zone_id: None,
        })
        .unwrap();
    let [received] = sink.received.as_slice() else {
        panic!("expected the configured sink to accept one alert");
    };
    assert_eq!(received.timestamp, event.timestamp);
    assert!(matches!(received.kind, AlertKind::PossibleFall));
}

#[test]
fn identifies_the_field_and_name_for_an_unknown_implementation() {
    for field in ["csi_source", "fall_classifier", "aux_sensor", "alert_sink"] {
        let contents = FAKE_CONFIG.replace(
            &format!("{field} = \"fake\""),
            &format!("{field} = \"unknown\""),
        );
        let error = PluginConfig::from_toml(&contents).unwrap_err();

        assert!(matches!(
            &error,
            PluginConfigError::UnknownImplementation { field: actual, name }
                if *actual == field && name == "unknown"
        ));
        assert!(error.to_string().contains(field));
        assert!(error.to_string().contains("unknown"));
    }
}

#[test]
fn rejects_malformed_incomplete_and_unexpected_fields() {
    let with_extra_field = format!("{FAKE_CONFIG}extra = \"fake\"\n");
    for contents in ["csi_source =", "csi_source = 'fake'", &with_extra_field] {
        assert!(matches!(
            PluginConfig::from_toml(contents),
            Err(PluginConfigError::Parse(_))
        ));
    }
}
