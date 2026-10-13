use std::collections::VecDeque;
use std::time::Duration;
use crate::error::HidppError;
use crate::frame::HidppFrame;

pub trait Transport {
    fn send(&mut self, frame: &HidppFrame) -> Result<(), HidppError>;
    fn receive(&mut self, timeout: Duration) -> Result<HidppFrame, HidppError>;

    /// Envía una petición y espera su respuesta correspondiente
    fn request(&mut self, frame: &HidppFrame, timeout: Duration) -> Result<HidppFrame, HidppError> {
        self.send(frame)?;
        self.receive(timeout)
    }
}

/// Transporte simulado en memoria para TDD aislado sin hardware
#[derive(Debug, Default)]
pub struct MockTransport {
    pub sent_frames: Vec<HidppFrame>,
    pub incoming_queue: VecDeque<HidppFrame>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_response(&mut self, frame: HidppFrame) {
        self.incoming_queue.push_back(frame);
    }
}

impl Transport for MockTransport {
    fn send(&mut self, frame: &HidppFrame) -> Result<(), HidppError> {
        self.sent_frames.push(frame.clone());
        Ok(())
    }

    fn receive(&mut self, _timeout: Duration) -> Result<HidppFrame, HidppError> {
        self.incoming_queue.pop_front().ok_or(HidppError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_transport_request() {
        let mut mock = MockTransport::new();
        let req = HidppFrame::short(0xFF, 0x00, 0x00, 0x01, &[0x00, 0x01]);
        let resp = HidppFrame::short(0xFF, 0x00, 0x00, 0x01, &[0x02, 0x00]);

        mock.enqueue_response(resp.clone());
        let res = mock.request(&req, Duration::from_millis(100)).expect("Debe recibir respuesta");

        assert_eq!(mock.sent_frames.len(), 1);
        assert_eq!(mock.sent_frames[0], req);
        assert_eq!(res, resp);
    }
}
