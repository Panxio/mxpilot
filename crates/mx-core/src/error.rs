use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum HidppError {
    #[error("Trama HID++ con longitud inválida: {0}")]
    InvalidLength(usize),

    #[error("Report ID desconocido: 0x{0:02X}")]
    UnknownReportId(u8),

    #[error("Error de protocolo HID++: feature 0x{feature_index:02X}, func {function_id}, código de error 0x{error_code:02X}")]
    ProtocolError {
        feature_index: u8,
        function_id: u8,
        error_code: u8,
    },

    #[error("Tiempo de espera agotado al comunicarse con el dispositivo")]
    Timeout,

    #[error("Dispositivo desconectado o enlace de hardware perdido")]
    Disconnected,

    #[error("Error de E/S: {0}")]
    Io(String),

    #[error("Dispositivo no encontrado")]
    DeviceNotFound,

    #[error("Característica HID++ no soportada: 0x{0:04X}")]
    FeatureNotFound(u16),

    #[error("Dispositivo no respondió al handshake inicial")]
    HandshakeFailed,
}
