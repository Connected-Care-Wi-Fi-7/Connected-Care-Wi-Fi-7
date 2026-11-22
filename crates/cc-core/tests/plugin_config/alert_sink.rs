use std::io;

use cc_core::{Alert, AlertKind, AlertSink, AlertSinkError};

#[derive(Default)]
pub(super) struct FakeAlertSink {
    pub(super) received: Vec<Alert>,
    failure: Option<AlertSinkError>,
}

impl AlertSink for FakeAlertSink {
    fn publish(&mut self, alert: &Alert) -> Result<(), AlertSinkError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        self.received.push(alert.clone());
        Ok(())
    }
}

fn alert() -> Alert {
    Alert {
        id: 42,
        timestamp: 20_000_000,
        kind: AlertKind::PossibleFall,
        track_id: Some(7),
        zone_id: None,
    }
}

#[test]
fn publishes_an_alert_through_a_trait_object() {
    let mut sink = FakeAlertSink::default();
    let publisher: &mut dyn AlertSink = &mut sink;
    publisher.publish(&alert()).unwrap();

    let [received] = sink.received.as_slice() else {
        panic!("expected one published alert");
    };
    assert_eq!(received.id, 42);
    assert_eq!(received.timestamp, 20_000_000);
    assert!(matches!(received.kind, AlertKind::PossibleFall));
    assert_eq!(received.track_id, Some(7));
    assert_eq!(received.zone_id, None);
}

#[test]
fn reports_publish_failures_with_the_underlying_cause() {
    let mut sink = FakeAlertSink {
        failure: Some(AlertSinkError::PublishFailed {
            source: io::Error::other("destination unavailable").into(),
        }),
        ..FakeAlertSink::default()
    };

    let error = sink.publish(&alert()).unwrap_err();
    assert_eq!(
        error.to_string(),
        "failed to publish alert: destination unavailable"
    );
    assert!(std::error::Error::source(&error).is_some());
    assert!(sink.received.is_empty());
}
