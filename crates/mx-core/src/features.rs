pub const FEATURE_ROOT: u16 = 0x0000;
pub const FEATURE_FEATURE_SET: u16 = 0x0001;
pub const FEATURE_DEVICE_INFO: u16 = 0x0003;
pub const FEATURE_BATTERY_STATUS: u16 = 0x1000;
pub const FEATURE_UNIFIED_BATTERY: u16 = 0x1004;
pub const FEATURE_REPROGRAMMABLE_KEYS: u16 = 0x1B04;
pub const FEATURE_ADJUSTABLE_DPI: u16 = 0x2201;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Discharging,
    Charging,
    Full,
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub state: BatteryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    pub key_index: u8,
    pub control_id: u16,
    pub task_id: u16,
    pub flags: u8,
    pub can_divert: bool,
}

impl BatteryInfo {
    /// Parsea la respuesta de Feature 0x1000 (BatteryStatus)
    /// Parámetros:
    /// Byte 0: Nivel porcentual (0..100)
    /// Byte 1: Próximo nivel de descarga
    /// Byte 2: Estado de carga (0: discharging, 1: recharging, 2: almost full, 3: full)
    pub fn parse_feature_1000(params: &[u8]) -> Option<Self> {
        if params.len() < 3 {
            return None;
        }
        let percentage = params[0].min(100);
        let state = match params[2] {
            0 => BatteryState::Discharging,
            1 => BatteryState::Charging,
            2 => BatteryState::Charging,
            3 => BatteryState::Full,
            other => BatteryState::Unknown(other),
        };
        Some(Self { percentage, state })
    }

    /// Parsea la respuesta de Feature 0x1004 (UnifiedBattery)
    /// Parámetros:
    /// Byte 0: Porcentaje (0..100)
    /// Byte 1: Flags de estado (bit 0-2: charging status)
    pub fn parse_feature_1004(params: &[u8]) -> Option<Self> {
        if params.is_empty() {
            return None;
        }
        let percentage = params[0].min(100);
        let state = if params.len() > 1 {
            let status_code = params[1] & 0x07;
            match status_code {
                0 => BatteryState::Discharging,
                1 => BatteryState::Charging,
                2 => BatteryState::Full,
                other => BatteryState::Unknown(other),
            }
        } else {
            BatteryState::Discharging
        };
        Some(Self { percentage, state })
    }
}

impl KeyInfo {
    /// Parsea la información de una tecla reprogramable de Feature 0x1B04
    /// Params:
    /// Byte 0-1: Control ID (Big Endian)
    /// Byte 2-3: Task ID (Big Endian)
    /// Byte 4: Flags (bit 0: divertable / reprogrammable)
    pub fn parse(key_index: u8, params: &[u8]) -> Option<Self> {
        if params.len() < 5 {
            return None;
        }
        let control_id = u16::from_be_bytes([params[0], params[1]]);
        let task_id = u16::from_be_bytes([params[2], params[3]]);
        let flags = params[4];
        let can_divert = (flags & 0x01) != 0 || (flags & 0x10) != 0;

        Some(Self {
            key_index,
            control_id,
            task_id,
            flags,
            can_divert,
        })
    }
}

// Constantes de Control ID (CID) para Logitech MX Vertical
pub const CID_LEFT_CLICK: u16 = 0x0050;
pub const CID_RIGHT_CLICK: u16 = 0x0051;
pub const CID_MIDDLE_CLICK: u16 = 0x0052;
pub const CID_BACK_BUTTON: u16 = 0x0053;
pub const CID_FORWARD_BUTTON: u16 = 0x0056;
pub const CID_TOP_BUTTON: u16 = 0x00FD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivertedButtonsEvent {
    pub pressed_cids: Vec<u16>,
}

impl DivertedButtonsEvent {
    /// Parsea el evento divertedButtonsEvent (Event 0 de Feature 0x1B04)
    /// Params contiene hasta 4 CIDs activos de 2 bytes cada uno
    pub fn parse(params: &[u8]) -> Self {
        let mut pressed_cids = Vec::new();
        for chunk in params.chunks_exact(2) {
            let cid = u16::from_be_bytes([chunk[0], chunk[1]]);
            if cid != 0 {
                pressed_cids.push(cid);
            }
        }
        Self { pressed_cids }
    }

    /// Comprueba si un CID específico se encuentra en la lista de presionados
    pub fn is_cid_pressed(&self, cid: u16) -> bool {
        self.pressed_cids.contains(&cid)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_battery_1000() {
        let params = [85, 80, 0]; // 85%, discharging
        let info = BatteryInfo::parse_feature_1000(&params).expect("Debe parsear batería");
        assert_eq!(info.percentage, 85);
        assert_eq!(info.state, BatteryState::Discharging);

        let charging_params = [40, 50, 1];
        let info_c = BatteryInfo::parse_feature_1000(&charging_params).unwrap();
        assert_eq!(info_c.percentage, 40);
        assert_eq!(info_c.state, BatteryState::Charging);
    }

    #[test]
    fn test_parse_key_info() {
        // CID 0x00C3, Task ID 0x0001, Flags 0x11 (divert enabled)
        let params = [0x00, 0xC3, 0x00, 0x01, 0x11];
        let key = KeyInfo::parse(2, &params).expect("Debe parsear tecla");
        assert_eq!(key.key_index, 2);
        assert_eq!(key.control_id, 0x00C3);
        assert_eq!(key.task_id, 0x0001);
        assert!(key.can_divert);
    }

    #[test]
    fn test_diverted_buttons_event() {
        // Evento con botón superior (0x00FD) presionado
        let params = [0x00, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let event = DivertedButtonsEvent::parse(&params);
        assert_eq!(event.pressed_cids.len(), 1);
        assert!(event.is_cid_pressed(CID_TOP_BUTTON));
        assert!(!event.is_cid_pressed(CID_BACK_BUTTON));

        // Evento con botón liberado (todos ceros)
        let empty_params = [0x00, 0x00, 0x00, 0x00];
        let empty_event = DivertedButtonsEvent::parse(&empty_params);
        assert_eq!(empty_event.pressed_cids.len(), 0);
        assert!(!empty_event.is_cid_pressed(CID_TOP_BUTTON));
    }
}
