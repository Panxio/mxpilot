use std::time::Duration;
use crate::error::HidppError;
use crate::features::*;
use crate::frame::HidppFrame;
use crate::transport::Transport;

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(500);

pub struct HidppDevice<T: Transport> {
    pub transport: T,
    pub device_index: u8,
    pub software_id: u8,
    pub use_long_reports: bool,
}

impl<T: Transport> HidppDevice<T> {
    pub fn new(transport: T, device_index: u8) -> Self {
        Self {
            transport,
            device_index,
            software_id: 0x01,
            use_long_reports: true,
        }
    }

    pub fn device_index(&self) -> u8 {
        self.device_index
    }

    pub fn set_device_index(&mut self, index: u8) {
        self.device_index = index;
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Descubre y conecta con el dispositivo probando el índice 0xFF (Bluetooth/directo)
    /// y luego los slots de receptor Unifying/Bolt (0x01..=0x04)
    pub fn auto_connect(transport: T) -> Result<Self, HidppError> {
        let mut dev = Self::new(transport, 0xFF);
        if dev.ping().unwrap_or(false) {
            return Ok(dev);
        }

        for slot in 1..=4 {
            dev.set_device_index(slot);
            if dev.ping().unwrap_or(false) {
                return Ok(dev);
            }
        }

        Err(HidppError::HandshakeFailed)
    }

    /// Incrementa y retorna el siguiente software ID cíclico (0x01..0x0F)
    fn next_sw_id(&mut self) -> u8 {
        let current = self.software_id;
        self.software_id = if current >= 0x0F { 0x01 } else { current + 1 };
        current
    }

    /// Construye una trama de petición según la configuración de reportes largos/cortos
    fn make_request(&mut self, feature_index: u8, function_id: u8, params: &[u8]) -> HidppFrame {
        let sw_id = self.next_sw_id();
        if self.use_long_reports {
            HidppFrame::long(self.device_index, feature_index, function_id, sw_id, params)
        } else {
            HidppFrame::short(self.device_index, feature_index, function_id, sw_id, params)
        }
    }

    /// Realiza un ping consultando la versión de protocolo o la feature IFeatureSet (0x0001)
    pub fn ping(&mut self) -> Result<bool, HidppError> {
        // Feature 0x00 (Root), Function 0x01: getProtocolVersion(0x00, 0x00, pingData)
        let req = self.make_request(0x00, 0x01, &[0x00, 0x00, 0x5A]);
        match self.transport.request(&req, DEFAULT_TIMEOUT) {
            Ok(resp) => Ok(resp.feature_index == 0x00),
            Err(HidppError::Timeout) => {
                // Fallback: consultar si responde a getFeature(0x0001)
                match self.get_feature_index(FEATURE_FEATURE_SET) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => {
                // Si da error de protocolo en Function 1, probar getFeature(0x0001)
                match self.get_feature_index(FEATURE_FEATURE_SET) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// Obtiene el índice dinámico asignado por firmware a una característica (Feature ID)
    pub fn get_feature_index(&mut self, feature_id: u16) -> Result<u8, HidppError> {
        let f_high = (feature_id >> 8) as u8;
        let f_low = (feature_id & 0xFF) as u8;

        // Feature 0x0000 (IRoot), Function 0x00 (getFeature): params [f_high, f_low, 0x00]
        let req = self.make_request(0x00, 0x00, &[f_high, f_low, 0x00]);
        let resp = self.transport.request(&req, DEFAULT_TIMEOUT)?;

        if resp.params.is_empty() {
            return Err(HidppError::InvalidLength(resp.params.len()));
        }

        let feature_index = resp.params[0];
        if feature_index == 0 {
            Err(HidppError::FeatureNotFound(feature_id))
        } else {
            Ok(feature_index)
        }
    }

    /// Lee el estado de batería probando la feature 0x1000 y luego 0x1004
    pub fn read_battery(&mut self) -> Result<BatteryInfo, HidppError> {
        // Intentar Feature 0x1000 primero
        if let Ok(idx_1000) = self.get_feature_index(FEATURE_BATTERY_STATUS) {
            // Function 0x00: getBatteryLevelStatus
            let req = self.make_request(idx_1000, 0x00, &[0, 0, 0]);
            let resp = self.transport.request(&req, DEFAULT_TIMEOUT)?;
            if let Some(info) = BatteryInfo::parse_feature_1000(&resp.params) {
                return Ok(info);
            }
        }

        // Si no está 0x1000, intentar Feature 0x1004 (Unified Battery)
        if let Ok(idx_1004) = self.get_feature_index(FEATURE_UNIFIED_BATTERY) {
            // Function 0x00: getCapability / getStatus
            let req = self.make_request(idx_1004, 0x01, &[0, 0, 0]);
            let resp = self.transport.request(&req, DEFAULT_TIMEOUT)?;
            if let Some(info) = BatteryInfo::parse_feature_1004(&resp.params) {
                return Ok(info);
            }
        }

        Err(HidppError::FeatureNotFound(FEATURE_BATTERY_STATUS))
    }

    /// Comprueba si el dispositivo expone la feature 0x1B04 (Reprogrammable Keys / Diversion)
    pub fn check_reprogrammable_keys_feature(&mut self) -> Result<Option<u8>, HidppError> {
        match self.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS) {
            Ok(idx) => Ok(Some(idx)),
            Err(HidppError::FeatureNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Comprueba si el dispositivo expone la feature 0x2201 (Adjustable DPI)
    pub fn check_adjustable_dpi_feature(&mut self) -> Result<Option<u8>, HidppError> {
        match self.get_feature_index(FEATURE_ADJUSTABLE_DPI) {
            Ok(idx) => Ok(Some(idx)),
            Err(HidppError::FeatureNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Activa o desactiva el divert sobre un Control ID (CID) específico (Feature 0x1B04, Function 0x03)
    pub fn set_cid_divert(&mut self, cid: u16, divert: bool) -> Result<(), HidppError> {
        let f_idx = self.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS)?;
        // bfield: bit 0 = divert value, bit 1 = divert valid mask
        let bfield = if divert { 0x03 } else { 0x02 };
        let cid_hi = (cid >> 8) as u8;
        let cid_lo = (cid & 0xFF) as u8;
        // Function 0x03: setCidReporting(cid_hi, cid_lo, bfield, remap_hi, remap_lo)
        let req = self.make_request(f_idx, 0x03, &[cid_hi, cid_lo, bfield, 0x00, 0x00]);
        let resp = self.transport.request(&req, DEFAULT_TIMEOUT)?;
        if resp.params.len() < 3 {
            return Err(HidppError::InvalidLength(resp.params.len()));
        }
        Ok(())
    }

    /// Crea un guardián RAII para garantizar la liberación del divert al salir
    pub fn divert_guard(&mut self, cid: u16) -> Result<DivertGuard<'_, T>, HidppError> {
        DivertGuard::new(self, cid)
    }

    /// Restaura todos los CIDs reprogramables reportados por el dispositivo para asegurar un estado limpio
    pub fn reset_all_diverts(&mut self) -> Result<usize, HidppError> {
        let keys = self.list_reprogrammable_keys()?;
        let mut count = 0;
        for key in keys {
            if key.can_divert {
                let _ = self.set_cid_divert(key.control_id, false);
                count += 1;
            }
        }
        Ok(count)
    }

    /// Lista todas las teclas reprogramables consultando la Feature 0x1B04
    pub fn list_reprogrammable_keys(&mut self) -> Result<Vec<KeyInfo>, HidppError> {
        let f_idx = self.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS)?;

        // Function 0x00: getCount
        let req_count = self.make_request(f_idx, 0x00, &[0, 0, 0]);
        let resp_count = self.transport.request(&req_count, DEFAULT_TIMEOUT)?;
        let count = resp_count.params.first().copied().unwrap_or(0);

        let mut keys = Vec::with_capacity(count as usize);
        for key_idx in 0..count {
            // Function 0x01: getCtrlId(key_idx)
            let req_key = self.make_request(f_idx, 0x01, &[key_idx, 0, 0]);
            let resp_key = self.transport.request(&req_key, DEFAULT_TIMEOUT)?;
            if let Some(k) = KeyInfo::parse(key_idx, &resp_key.params) {
                keys.push(k);
            }
        }

        Ok(keys)
    }

    /// Lista todos los Feature IDs expuestos por el dispositivo consultando IFeatureSet (0x0001)
    pub fn list_all_features(&mut self) -> Result<Vec<u16>, HidppError> {
        let fset_idx = self.get_feature_index(FEATURE_FEATURE_SET)?;

        // Function 0x00: getCount
        let req_count = self.make_request(fset_idx, 0x00, &[0, 0, 0]);
        let resp_count = self.transport.request(&req_count, DEFAULT_TIMEOUT)?;
        let count = resp_count.params.first().copied().unwrap_or(0);

        let mut features = Vec::with_capacity(count as usize);
        for idx in 1..=count {
            // Function 0x01: getFeatureId(idx)
            let req_f = self.make_request(fset_idx, 0x01, &[idx, 0, 0]);
            let resp_f = self.transport.request(&req_f, DEFAULT_TIMEOUT)?;
            if resp_f.params.len() >= 2 {
                let feat_id = u16::from_be_bytes([resp_f.params[0], resp_f.params[1]]);
                features.push(feat_id);
            }
        }

        Ok(features)
    }
}

/// Guardián RAII para garantizar la liberación del desvío de un botón físico (Zero Dead Buttons)
pub struct DivertGuard<'a, T: Transport> {
    device: &'a mut HidppDevice<T>,
    cid: u16,
    active: bool,
}

impl<'a, T: Transport> DivertGuard<'a, T> {
    pub fn new(device: &'a mut HidppDevice<T>, cid: u16) -> Result<Self, HidppError> {
        device.set_cid_divert(cid, true)?;
        Ok(Self {
            device,
            cid,
            active: true,
        })
    }

    pub fn cid(&self) -> u16 {
        self.cid
    }

    /// Desactiva el divert cooperativamente y consume el guardián
    pub fn release(mut self) -> Result<(), HidppError> {
        self.active = false;
        self.device.set_cid_divert(self.cid, false)
    }

    /// Desactiva el guardián sin enviar paquete de restauración al dispositivo
    pub fn defuse(&mut self) {
        self.active = false;
    }

    pub fn device_mut(&mut self) -> &mut HidppDevice<T> {
        self.device
    }

    pub fn transport_mut(&mut self) -> &mut T {
        self.device.transport_mut()
    }
}

impl<'a, T: Transport> Drop for DivertGuard<'a, T> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.device.set_cid_divert(self.cid, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;

    #[test]
    fn test_device_get_feature_index() {
        let mut mock = MockTransport::new();
        // Respuesta a getFeature(0x1B04): asigna feature_index = 0x0A
        let resp = HidppFrame::long(0xFF, 0x00, 0x00, 0x01, &[0x0A, 0x00, 0x00]);
        mock.enqueue_response(resp);

        let mut dev = HidppDevice::new(mock, 0xFF);
        let idx = dev.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS).expect("Debe resolver feature index");
        assert_eq!(idx, 0x0A);
    }

    #[test]
    fn test_device_read_battery() {
        let mut mock = MockTransport::new();
        // 1. Resolver feature 0x1000 -> index 0x04
        mock.enqueue_response(HidppFrame::long(0xFF, 0x00, 0x00, 0x01, &[0x04, 0x00, 0x00]));
        // 2. Respuesta a lectura de batería: 75%, discharging
        mock.enqueue_response(HidppFrame::long(0xFF, 0x04, 0x00, 0x02, &[75, 70, 0x00]));

        let mut dev = HidppDevice::new(mock, 0xFF);
        let battery = dev.read_battery().expect("Debe leer batería");
        assert_eq!(battery.percentage, 75);
        assert_eq!(battery.state, BatteryState::Discharging);
    }

    #[test]
    fn test_kill_gate_check_feature_1b04() {
        let mut mock = MockTransport::new();
        // Feature 0x1B04 no soportada (devuelve 0x00)
        mock.enqueue_response(HidppFrame::long(0xFF, 0x00, 0x00, 0x01, &[0x00, 0x00, 0x00]));

        let mut dev = HidppDevice::new(mock, 0xFF);
        let res = dev.check_reprogrammable_keys_feature().expect("Debe ejecutar consulta");
        assert_eq!(res, None);
    }

    #[test]
    fn test_set_cid_divert() {
        let mut mock = MockTransport::new();
        // 1. Resolver feature 0x1B04 -> index 0x0A
        mock.enqueue_response(HidppFrame::long(0xFF, 0x00, 0x00, 0x01, &[0x0A, 0x00, 0x00]));
        // 2. Respuesta a setCidReporting: echo
        mock.enqueue_response(HidppFrame::long(0xFF, 0x0A, 0x03, 0x02, &[0x00, 0xFD, 0x03, 0x00, 0x00]));

        let mut dev = HidppDevice::new(mock, 0xFF);
        let res = dev.set_cid_divert(CID_TOP_BUTTON, true);
        assert!(res.is_ok());

        assert_eq!(dev.transport.sent_frames.len(), 2);
        let set_req = &dev.transport.sent_frames[1];
        assert_eq!(set_req.feature_index, 0x0A);
        assert_eq!(set_req.function_id, 0x03);
        assert_eq!(set_req.params[0..3], [0x00, 0xFD, 0x03]); // CID 0x00FD, divert ON (0x03)
    }

    #[test]
    fn test_divert_guard_raii_drop() {
        let mut mock = MockTransport::new();
        // 1. Resolver feature 0x1B04 -> index 0x0A (para divert ON)
        mock.enqueue_response(HidppFrame::long(0xFF, 0x00, 0x00, 0x01, &[0x0A, 0x00, 0x00]));
        // 2. Respuesta a setCidReporting ON
        mock.enqueue_response(HidppFrame::long(0xFF, 0x0A, 0x03, 0x02, &[0x00, 0xFD, 0x03, 0x00, 0x00]));
        // 3. Resolver feature 0x1B04 -> index 0x0A (para divert OFF en Drop)
        mock.enqueue_response(HidppFrame::long(0xFF, 0x00, 0x00, 0x03, &[0x0A, 0x00, 0x00]));
        // 4. Respuesta a setCidReporting OFF
        mock.enqueue_response(HidppFrame::long(0xFF, 0x0A, 0x03, 0x04, &[0x00, 0xFD, 0x02, 0x00, 0x00]));

        let mut dev = HidppDevice::new(mock, 0xFF);
        {
            let guard = dev.divert_guard(CID_TOP_BUTTON).expect("Debe crear guard");
            assert_eq!(guard.cid(), CID_TOP_BUTTON);
            // El guard sale de scope aquí -> drop ejecuta set_cid_divert(..., false)
        }

        // Verificar que se enviaron tramas para divert ON (0x03) y luego OFF (0x02)
        assert!(dev.transport.sent_frames.len() >= 4);
        let last_frame = dev.transport.sent_frames.last().unwrap();
        assert_eq!(last_frame.params[0..3], [0x00, 0xFD, 0x02]); // bfield 0x02 = divert OFF
    }
}
