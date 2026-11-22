use std::{collections::VecDeque, io};

use cc_core::{
    ComplexCsi, CsiFrame, CsiLayout, CsiPoll, CsiSession, CsiSource, CsiSourceError, CsiTensor,
};

pub(super) struct FakeCsiSource {
    pub(super) session: CsiSession,
    pub(super) outcomes: VecDeque<Result<CsiPoll, CsiSourceError>>,
}

impl CsiSource for FakeCsiSource {
    fn session(&self) -> &CsiSession {
        &self.session
    }

    fn poll_frame(&mut self) -> Result<CsiPoll, CsiSourceError> {
        self.outcomes
            .pop_front()
            .unwrap_or(Ok(CsiPoll::EndOfStream))
    }
}

pub(super) fn session() -> CsiSession {
    CsiSession {
        layout: CsiLayout {
            tx_count: 1,
            rx_count: 1,
            subcarrier_indices: vec![-1],
        },
        center_frequency_hz: 5_000_000_000,
        channel_bandwidth_hz: 20_000_000,
        subcarrier_spacing_hz: 78_125,
        amplitude_convention: "synthetic linear".into(),
    }
}

#[test]
fn polls_a_session_through_a_trait_object() {
    let session = session();
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
    let mut source: Box<dyn CsiSource> = Box::new(FakeCsiSource {
        session,
        outcomes: VecDeque::from([Ok(CsiPoll::Pending), Ok(CsiPoll::Frame(frame))]),
    });

    assert_eq!(source.session().layout.subcarrier_indices, [-1]);
    assert!(matches!(source.poll_frame(), Ok(CsiPoll::Pending)));
    let CsiPoll::Frame(frame) = source.poll_frame().unwrap() else {
        panic!("expected a frame after Pending");
    };
    assert_eq!(frame.captured_at_ns, 0);
    assert!(frame.csi.validate_for(&source.session().layout).is_ok());
    assert!(matches!(source.poll_frame(), Ok(CsiPoll::EndOfStream)));
    assert!(matches!(source.poll_frame(), Ok(CsiPoll::EndOfStream)));
}

#[test]
fn reports_read_failures_with_the_underlying_cause() {
    let mut source = FakeCsiSource {
        session: session(),
        outcomes: VecDeque::from([Err(CsiSourceError::ReadFailed {
            source: io::Error::new(io::ErrorKind::InvalidData, "invalid frame").into(),
        })]),
    };

    let error = source.poll_frame().unwrap_err();
    assert_eq!(error.to_string(), "failed to read CSI frame: invalid frame");
    assert!(std::error::Error::source(&error).is_some());
}
