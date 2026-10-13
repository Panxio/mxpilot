use crate::error::HidppError;

pub const SHORT_REPORT_ID: u8 = 0x10;
pub const LONG_REPORT_ID: u8 = 0x11;
pub const ERROR_REPORT_ID: u8 = 0x8F;

pub const SHORT_REPORT_LEN: usize = 7;
pub const LONG_REPORT_LEN: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidppFrame {
    pub report_id: u8,
    pub device_index: u8,
    pub feature_index: u8,
    pub function_id: u8,
    pub software_id: u8,
    pub params: Vec<u8>,
}

impl HidppFrame {
    /// Crea una nueva trama HID++ corta (7 bytes)
    pub fn short(
        device_index: u8,
        feature_index: u8,
        function_id: u8,
        software_id: u8,
        params: &[u8],
    ) -> Self {
        let mut p = Vec::with_capacity(3);
        p.extend_from_slice(params);
        while p.len() < 3 {
            p.push(0);
        }
        p.truncate(3);

        Self {
            report_id: SHORT_REPORT_ID,
            device_index,
            feature_index,
            function_id,
            software_id,
            params: p,
        }
    }

    /// Crea una nueva trama HID++ larga (20 bytes)
    pub fn long(
        device_index: u8,
        feature_index: u8,
        function_id: u8,
        software_id: u8,
        params: &[u8],
    ) -> Self {
        let mut p = Vec::with_capacity(16);
        p.extend_from_slice(params);
        while p.len() < 16 {
            p.push(0);
        }
        p.truncate(16);

        Self {
            report_id: LONG_REPORT_ID,
            device_index,
            feature_index,
            function_id,
            software_id,
            params: p,
        }
    }

    /// Serializa la trama en bytes listos para escribir en /dev/hidraw
    pub fn to_bytes(&self) -> Vec<u8> {
        let expected_len = match self.report_id {
            SHORT_REPORT_ID => SHORT_REPORT_LEN,
            LONG_REPORT_ID => LONG_REPORT_LEN,
            _ => 1 + 1 + 1 + 1 + self.params.len(),
        };

        let mut out = Vec::with_capacity(expected_len);
        out.push(self.report_id);
        out.push(self.device_index);
        out.push(self.feature_index);
        out.push((self.function_id << 4) | (self.software_id & 0x0F));
        out.extend_from_slice(&self.params);

        while out.len() < expected_len {
            out.push(0);
        }
        out
    }

    /// Deserializa una trama a partir de un slice crudo leído de /dev/hidraw
    pub fn from_bytes(raw: &[u8]) -> Result<Self, HidppError> {
        if raw.len() < 4 {
            return Err(HidppError::InvalidLength(raw.len()));
        }

        let report_id = raw[0];
        let device_index = raw[1];

        // Manejo de reporte de error HID++ 1.0 (0x8F)
        if report_id == ERROR_REPORT_ID {
            if raw.len() < 4 {
                return Err(HidppError::InvalidLength(raw.len()));
            }
            return Err(HidppError::ProtocolError {
                feature_index: raw[2],
                function_id: raw[1],
                error_code: raw[3],
            });
        }

        if report_id != SHORT_REPORT_ID && report_id != LONG_REPORT_ID {
            return Err(HidppError::UnknownReportId(report_id));
        }

        let min_len = match report_id {
            SHORT_REPORT_ID => SHORT_REPORT_LEN,
            LONG_REPORT_ID => LONG_REPORT_LEN,
            _ => 4,
        };

        if raw.len() < min_len {
            return Err(HidppError::InvalidLength(raw.len()));
        }

        let feature_index = raw[2];
        let func_sw = raw[3];
        let function_id = func_sw >> 4;
        let software_id = func_sw & 0x0F;

        // Comprobación de respuesta de error HID++ 2.0 (típicamente feature_index es el solicitado, pero puede tener código de error)
        // En HID++ 2.0, si el dispositivo devuelve error, la feature_index suele ser 0x8F o function_id 0x0F
        if feature_index == 0xFF && raw.len() >= 5 {
            return Err(HidppError::ProtocolError {
                feature_index: raw[2],
                function_id,
                error_code: raw[4],
            });
        }

        let payload_len = match report_id {
            SHORT_REPORT_ID => 3,
            LONG_REPORT_ID => 16,
            _ => raw.len() - 4,
        };

        let params = raw[4..4 + payload_len].to_vec();

        Ok(Self {
            report_id,
            device_index,
            feature_index,
            function_id,
            software_id,
            params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_frame_serialization() {
        let frame = HidppFrame::short(0xFF, 0x00, 0x01, 0x02, &[0x11, 0x22]);
        let bytes = frame.to_bytes();
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[0], 0x10);
        assert_eq!(bytes[1], 0xFF);
        assert_eq!(bytes[2], 0x00);
        assert_eq!(bytes[3], 0x12); // (1 << 4) | 2
        assert_eq!(bytes[4], 0x11);
        assert_eq!(bytes[5], 0x22);
        assert_eq!(bytes[6], 0x00); // padding
    }

    #[test]
    fn test_long_frame_serialization() {
        let frame = HidppFrame::long(0xFF, 0x1B, 0x02, 0x03, &[0xAA, 0xBB, 0xCC]);
        let bytes = frame.to_bytes();
        assert_eq!(bytes.len(), 20);
        assert_eq!(bytes[0], 0x11);
        assert_eq!(bytes[1], 0xFF);
        assert_eq!(bytes[2], 0x1B);
        assert_eq!(bytes[3], 0x23);
        assert_eq!(bytes[4], 0xAA);
        assert_eq!(bytes[5], 0xBB);
        assert_eq!(bytes[6], 0xCC);
        assert_eq!(bytes[7], 0x00); // padding
    }

    #[test]
    fn test_frame_deserialization_roundtrip() {
        let original = HidppFrame::short(0xFF, 0x05, 0x03, 0x01, &[0x01, 0x02, 0x03]);
        let bytes = original.to_bytes();
        let parsed = HidppFrame::from_bytes(&bytes).expect("Debe deserializar correctamente");
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_error_report_deserialization() {
        let err_bytes = [0x8F, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00];
        let res = HidppFrame::from_bytes(&err_bytes);
        assert!(matches!(
            res,
            Err(HidppError::ProtocolError {
                feature_index: 0x00,
                function_id: 0x01,
                error_code: 0x02
            })
        ));
    }
}
