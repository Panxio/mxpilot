use evdev::KeyCode;
use mx_uinput::Action;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identificador CID del botón superior del MX Vertical.
pub const TOP_BUTTON_CID: u16 = 0x00FD;

/// Lista de CIDs conocidos del Logitech MX Vertical y familia MX.
pub const VALID_CIDS: &[u16] = &[
    0x0050, // Clic izquierdo
    0x0051, // Clic derecho
    0x0052, // Clic central
    0x0053, // Botón lateral atrás (Back)
    0x0056, // Botón lateral adelante (Forward)
    0x00D0, // Gesto / Scroll click
    0x00FD, // Botón superior de DPI (Kill-Gate)
];

/// Representación serializable de una acción de entrada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionPayload {
    SingleKey {
        key: String,
    },
    KeyCombo {
        modifiers: Vec<String>,
        key: String,
    },
    MouseButton {
        button: String,
    },
    None,
    SystemDefault,
}

impl ActionPayload {
    /// Convierte el payload a una acción uinput tipada tras validar allowlist.
    pub fn to_action(&self) -> Result<Action, String> {
        match self {
            Self::SingleKey { key } => {
                let code = parse_key_code(key)?;
                Ok(Action::SingleKey(code))
            }
            Self::KeyCombo { modifiers, key } => {
                let mut mods = Vec::new();
                for m in modifiers {
                    let c = parse_key_code(m)?;
                    mods.push(c);
                }
                let code = parse_key_code(key)?;
                Ok(Action::KeyCombo {
                    modifiers: mods,
                    key: code,
                })
            }
            Self::MouseButton { button } => {
                let code = parse_mouse_button(button)?;
                Ok(Action::MouseButton(code))
            }
            Self::None => Ok(Action::None),
            Self::SystemDefault => Ok(Action::None),
        }
    }

    /// Crea un payload desde una acción uinput.
    pub fn from_action(action: &Action) -> Self {
        match action {
            Action::SingleKey(k) => Self::SingleKey {
                key: format!("{:?}", k),
            },
            Action::KeyCombo { modifiers, key } => Self::KeyCombo {
                modifiers: modifiers.iter().map(|m| format!("{:?}", m)).collect(),
                key: format!("{:?}", key),
            },
            Action::MouseButton(b) => Self::MouseButton {
                button: format!("{:?}", b),
            },
            Action::None => Self::None,
        }
    }
}

/// Mapeo seguro de nombres de teclas a KeyCode de evdev.
pub fn parse_key_code(s: &str) -> Result<KeyCode, String> {
    match s {
        "KEY_LEFTMETA" | "SUPER" | "META" => Ok(KeyCode::KEY_LEFTMETA),
        "KEY_RIGHTMETA" => Ok(KeyCode::KEY_RIGHTMETA),
        "KEY_LEFTCTRL" | "CTRL" => Ok(KeyCode::KEY_LEFTCTRL),
        "KEY_RIGHTCTRL" => Ok(KeyCode::KEY_RIGHTCTRL),
        "KEY_LEFTALT" | "ALT" => Ok(KeyCode::KEY_LEFTALT),
        "KEY_RIGHTALT" => Ok(KeyCode::KEY_RIGHTALT),
        "KEY_LEFTSHIFT" | "SHIFT" => Ok(KeyCode::KEY_LEFTSHIFT),
        "KEY_RIGHTSHIFT" => Ok(KeyCode::KEY_RIGHTSHIFT),
        "KEY_TAB" => Ok(KeyCode::KEY_TAB),
        "KEY_ENTER" => Ok(KeyCode::KEY_ENTER),
        "KEY_ESC" => Ok(KeyCode::KEY_ESC),
        "KEY_SPACE" => Ok(KeyCode::KEY_SPACE),
        "KEY_BACKSPACE" => Ok(KeyCode::KEY_BACKSPACE),
        "KEY_C" => Ok(KeyCode::KEY_C),
        "KEY_V" => Ok(KeyCode::KEY_V),
        "KEY_X" => Ok(KeyCode::KEY_X),
        "KEY_Z" => Ok(KeyCode::KEY_Z),
        "KEY_A" => Ok(KeyCode::KEY_A),
        "KEY_Q" => Ok(KeyCode::KEY_Q),
        "KEY_W" => Ok(KeyCode::KEY_W),
        "KEY_E" => Ok(KeyCode::KEY_E),
        "KEY_R" => Ok(KeyCode::KEY_R),
        "KEY_T" => Ok(KeyCode::KEY_T),
        "KEY_PLAYPAUSE" => Ok(KeyCode::KEY_PLAYPAUSE),
        "KEY_MUTE" => Ok(KeyCode::KEY_MUTE),
        "KEY_VOLUMEUP" => Ok(KeyCode::KEY_VOLUMEUP),
        "KEY_VOLUMEDOWN" => Ok(KeyCode::KEY_VOLUMEDOWN),
        _ => Err(format!("Tecla no permitida o desconocida: {}", s)),
    }
}

/// Mapeo seguro de nombres de botón de ratón a KeyCode (BTN_*).
pub fn parse_mouse_button(s: &str) -> Result<KeyCode, String> {
    match s {
        "BTN_LEFT" => Ok(KeyCode::BTN_LEFT),
        "BTN_RIGHT" => Ok(KeyCode::BTN_RIGHT),
        "BTN_MIDDLE" => Ok(KeyCode::BTN_MIDDLE),
        "BTN_SIDE" => Ok(KeyCode::BTN_SIDE),
        "BTN_EXTRA" => Ok(KeyCode::BTN_EXTRA),
        _ => Err(format!("Botón de ratón no permitido o desconocido: {}", s)),
    }
}

/// Comando solicitado a través de IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcCommand {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "get_status")]
    GetStatus,
    #[serde(rename = "get_health")]
    GetHealth,
    #[serde(rename = "get_battery")]
    GetBattery,
    #[serde(rename = "set_action")]
    SetAction {
        cid: u16,
        action: ActionPayload,
    },
    #[serde(rename = "reload_profile")]
    ReloadProfile,
}

/// Petición IPC entrante (con `id` opcional para correlación asíncrona).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcRequest {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(flatten)]
    pub command: IpcCommand,
}

/// Datos funcionales retornados por `get_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusData {
    pub connected: bool,
    pub device_name: Option<String>,
    pub device_path: Option<String>,
    pub battery_level: Option<u8>,
    pub battery_status: Option<String>,
    pub diverted_cids: Vec<u16>,
    pub mapping: HashMap<u16, ActionPayload>,
}

/// Datos de salud y telemetría retornados por `get_health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonHealth {
    pub state: String,
    pub uptime_seconds: u64,
    pub device_name: Option<String>,
    pub device_path: Option<String>,
    pub device_index: Option<u8>,
    pub divert_epoch: u64,
    pub reconnect_count: u64,
    pub consecutive_failures: u32,
    pub queue_depth: usize,
    pub dropped_events: u64,
    pub last_error: Option<String>,
    pub rss_kb: u64,
}

/// Datos de batería retornados por `get_battery`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryData {
    pub percentage: u8,
    pub status: String,
}

/// Respuesta tipada emitida por el daemon hacia el cliente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum IpcResponse {
    #[serde(rename = "ok")]
    Ok {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        message: String,
    },
}

impl IpcResponse {
    /// Crea una respuesta de éxito con datos.
    pub fn ok<T: Serialize>(id: Option<u64>, data: T) -> Self {
        let value = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
        Self::Ok {
            id,
            data: Some(value),
        }
    }

    /// Crea una respuesta de éxito vacía (data = null).
    pub fn ok_empty(id: Option<u64>) -> Self {
        Self::Ok { id, data: None }
    }

    /// Crea una respuesta de error.
    pub fn error(id: Option<u64>, message: impl Into<String>) -> Self {
        Self::Error {
            id,
            message: message.into(),
        }
    }
}

/// Parsea una línea JSON entrante validando framing y sintaxis.
pub fn parse_request(line: &str) -> Result<IpcRequest, String> {
    serde_json::from_str::<IpcRequest>(line)
        .map_err(|e| format!("Error deserializando JSON IPC: {}", e))
}

/// Serializa una respuesta IPC a formato JSON delimitado por salto de línea (`\n`).
pub fn format_response(response: &IpcResponse) -> String {
    let mut s = serde_json::to_string(response).unwrap_or_else(|_| {
        r#"{"status":"error","message":"Internal serialization error"}"#.to_string()
    });
    s.push('\n');
    s
}

/// Valida si un CID es reprogramable o soportado.
pub fn is_valid_cid(cid: u16) -> bool {
    VALID_CIDS.contains(&cid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_request() {
        let json = r#"{"cmd":"ping"}"#;
        let req = parse_request(json).expect("Debe parsear ping");
        assert_eq!(req.id, None);
        assert_eq!(req.command, IpcCommand::Ping);
    }

    #[test]
    fn test_parse_ping_with_id() {
        let json = r#"{"id":42,"cmd":"ping"}"#;
        let req = parse_request(json).expect("Debe parsear ping con id");
        assert_eq!(req.id, Some(42));
        assert_eq!(req.command, IpcCommand::Ping);
    }

    #[test]
    fn test_parse_set_action_super() {
        let json = r#"{"id":1,"cmd":"set_action","cid":253,"action":{"type":"SingleKey","key":"KEY_LEFTMETA"}}"#;
        let req = parse_request(json).expect("Debe parsear set_action");
        assert_eq!(req.id, Some(1));
        if let IpcCommand::SetAction { cid, action } = req.command {
            assert_eq!(cid, 253);
            let act = action.to_action().expect("Debe convertir a Action");
            assert_eq!(act, Action::SingleKey(KeyCode::KEY_LEFTMETA));
        } else {
            panic!("Expected SetAction");
        }
    }

    #[test]
    fn test_invalid_key_rejected() {
        let payload = ActionPayload::SingleKey {
            key: "KEY_HACKER_ATTEMPT".to_string(),
        };
        assert!(payload.to_action().is_err());
    }

    #[test]
    fn test_format_ok_response() {
        let resp = IpcResponse::ok(Some(10), "pong");
        let formatted = format_response(&resp);
        assert!(formatted.ends_with('\n'));
        assert!(formatted.contains(r#""status":"ok""#));
        assert!(formatted.contains(r#""id":10"#));
        assert!(formatted.contains(r#""data":"pong""#));
    }

    #[test]
    fn test_format_error_response() {
        let resp = IpcResponse::error(Some(5), "Device not found");
        let formatted = format_response(&resp);
        assert!(formatted.ends_with('\n'));
        assert!(formatted.contains(r#""status":"error""#));
        assert!(formatted.contains(r#""message":"Device not found""#));
    }

    #[test]
    fn test_health_serialization() {
        let health = DaemonHealth {
            state: "Active".to_string(),
            uptime_seconds: 120,
            device_name: Some("Logitech MX Vertical".to_string()),
            device_path: Some("/dev/hidraw3".to_string()),
            device_index: Some(255),
            divert_epoch: 1,
            reconnect_count: 0,
            consecutive_failures: 0,
            queue_depth: 0,
            dropped_events: 0,
            last_error: None,
            rss_kb: 3200,
        };
        let resp = IpcResponse::ok(Some(1), health);
        let s = format_response(&resp);
        assert!(s.contains(r#""state":"Active""#));
        assert!(s.contains(r#""rss_kb":3200"#));
        assert!(s.contains(r#""device_index":255"#));
        assert!(s.contains(r#""consecutive_failures":0"#));
    }

    #[test]
    fn test_cid_allowlist() {
        assert!(is_valid_cid(TOP_BUTTON_CID));
        assert!(is_valid_cid(0x0053)); // Back
        assert!(!is_valid_cid(0x9999));
    }
}
