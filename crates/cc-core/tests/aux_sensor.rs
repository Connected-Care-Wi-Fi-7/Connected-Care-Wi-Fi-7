use std::{collections::VecDeque, io};

use cc_core::{AuxReading, AuxSensor, AuxSensorError};

struct FakeAuxSensor {
    outcomes: VecDeque<Result<Option<AuxReading>, AuxSensorError>>,
}

impl AuxSensor for FakeAuxSensor {
    fn poll_reading(&mut self) -> Result<Option<AuxReading>, AuxSensorError> {
        self.outcomes.pop_front().unwrap_or(Ok(None))
    }
}

#[test]
fn distinguishes_inactivity_from_no_reading_on_the_csi_session_timeline() {
    let active = AuxReading {
        captured_at_ns: 0,
        active: true,
    };
    let inactive = AuxReading {
        captured_at_ns: 20_000_000,
        active: false,
    };
    let mut sensor: Box<dyn AuxSensor> = Box::new(FakeAuxSensor {
        outcomes: VecDeque::from([Ok(Some(active)), Ok(None), Ok(Some(inactive))]),
    });

    assert_eq!(sensor.poll_reading().unwrap(), Some(active));
    assert_eq!(sensor.poll_reading().unwrap(), None);
    assert_eq!(sensor.poll_reading().unwrap(), Some(inactive));
}

#[test]
fn reports_read_failures_with_the_underlying_cause() {
    let mut sensor = FakeAuxSensor {
        outcomes: VecDeque::from([Err(AuxSensorError::ReadFailed {
            source: io::Error::other("sensor unavailable").into(),
        })]),
    };

    let error = sensor.poll_reading().unwrap_err();
    assert_eq!(
        error.to_string(),
        "failed to read auxiliary sensor: sensor unavailable"
    );
    assert!(std::error::Error::source(&error).is_some());
}
