use std::{collections::VecDeque, io};

use cc_core::{
    CsiLayout, CsiTensor, FallClassifier, FallClassifierError, FallEvent, Position, ProcessedFrame,
    Track,
};

pub(super) struct FakeFallClassifier {
    pub(super) outcomes: VecDeque<Result<Vec<FallEvent>, FallClassifierError>>,
}

impl FallClassifier for FakeFallClassifier {
    fn classify(
        &mut self,
        _frame: &ProcessedFrame,
        _tracks: &[Track],
    ) -> Result<Vec<FallEvent>, FallClassifierError> {
        self.outcomes.pop_front().expect("scripted outcome")
    }
}

fn frame(captured_at_ns: u64) -> ProcessedFrame {
    let layout = CsiLayout {
        tx_count: 1,
        rx_count: 1,
        subcarrier_indices: vec![-1],
    };
    ProcessedFrame {
        captured_at_ns,
        signals: CsiTensor::new(&layout, vec![None]).unwrap(),
        motion_energy: None,
    }
}

#[test]
fn returns_zero_or_multiple_falls_through_a_trait_object() {
    let tracks = [Track {
        id: 7,
        position: Position {
            x: 1.0,
            y: 2.0,
            z: None,
        },
        timestamp: 20,
    }];
    let mut classifier: Box<dyn FallClassifier> = Box::new(FakeFallClassifier {
        outcomes: VecDeque::from([
            Ok(Vec::new()),
            Ok(vec![
                FallEvent {
                    timestamp: 18,
                    track_id: Some(tracks[0].id),
                },
                FallEvent {
                    timestamp: 20,
                    track_id: None,
                },
            ]),
        ]),
    });

    assert!(classifier.classify(&frame(10), &[]).unwrap().is_empty());
    let events = classifier.classify(&frame(20), &tracks).unwrap();
    let [tracked, untracked] = events.as_slice() else {
        panic!("expected two fall events");
    };
    assert_eq!((tracked.timestamp, tracked.track_id), (18, Some(7)));
    assert_eq!((untracked.timestamp, untracked.track_id), (20, None));
}

#[test]
fn reports_classification_failure_with_the_underlying_cause() {
    let mut classifier = FakeFallClassifier {
        outcomes: VecDeque::from([Err(FallClassifierError::ClassificationFailed {
            source: io::Error::other("classifier unavailable").into(),
        })]),
    };

    let error = classifier.classify(&frame(0), &[]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "failed to classify fall update: classifier unavailable"
    );
    assert!(std::error::Error::source(&error).is_some());
}
