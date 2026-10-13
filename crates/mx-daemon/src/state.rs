use crate::ipc::protocol::{ActionPayload, BatteryData, DaemonHealth, StatusData, TOP_BUTTON_CID};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

/// Estados de la máquina de ciclo de vida del daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Disconnected,
    Connecting,
    Reconciling,
    Active,
    ShuttingDown,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Reconciling => write!(f, "Reconciling"),
            Self::Active => write!(f, "Active"),
            Self::ShuttingDown => write!(f, "ShuttingDown"),
        }
    }
}

/// Estado global dinámico del daemon.
#[derive(Debug, Clone)]
pub struct DaemonState {
    pub state: State,
    pub device_name: Option<String>,
    pub device_path: Option<String>,
    pub device_index: Option<u8>,
    pub battery_level: Option<u8>,
    pub battery_status: Option<String>,
    pub battery_last_updated: Option<Instant>,
    pub diverted_cids: Vec<u16>,
    pub mapping: HashMap<u16, ActionPayload>,
    pub divert_epoch: u64,
    pub reconnect_count: u64,
    pub consecutive_failures: u32,
    pub queue_depth: usize,
    pub dropped_events: u64,
    pub start_time: Instant,
    pub last_error: Option<String>,
}

impl DaemonState {
    pub fn new() -> Self {
        let mut mapping = HashMap::new();
        // Mapeo por defecto para el botón superior: Super / Meta
        mapping.insert(
            TOP_BUTTON_CID,
            ActionPayload::SingleKey {
                key: "KEY_LEFTMETA".to_string(),
            },
        );

        Self {
            state: State::Disconnected,
            device_name: None,
            device_path: None,
            device_index: None,
            battery_level: None,
            battery_status: None,
            battery_last_updated: None,
            diverted_cids: Vec::new(),
            mapping,
            divert_epoch: 0,
            reconnect_count: 0,
            consecutive_failures: 0,
            queue_depth: 0,
            dropped_events: 0,
            start_time: Instant::now(),
            last_error: None,
        }
    }

    /// Genera la estructura `StatusData` para la respuesta IPC `get_status`.
    pub fn to_status_data(&self) -> StatusData {
        StatusData {
            connected: self.state == State::Active,
            device_name: self.device_name.clone(),
            device_path: self.device_path.clone(),
            battery_level: self.battery_level,
            battery_status: self.battery_status.clone(),
            diverted_cids: self.diverted_cids.clone(),
            mapping: self.mapping.clone(),
        }
    }

    /// Genera la telemetría machine-readable `DaemonHealth` para `get_health`.
    pub fn to_health(&self) -> DaemonHealth {
        DaemonHealth {
            state: self.state.to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            device_name: self.device_name.clone(),
            device_path: self.device_path.clone(),
            device_index: self.device_index,
            divert_epoch: self.divert_epoch,
            reconnect_count: self.reconnect_count,
            consecutive_failures: self.consecutive_failures,
            queue_depth: self.queue_depth,
            dropped_events: self.dropped_events,
            last_error: self.last_error.clone(),
            rss_kb: get_process_rss_kb(),
        }
    }

    /// Genera la estructura `BatteryData` para `get_battery`.
    pub fn to_battery_data(&self) -> Option<BatteryData> {
        match (self.battery_level, &self.battery_status) {
            (Some(pct), Some(status)) => Some(BatteryData {
                percentage: pct,
                status: status.clone(),
            }),
            _ => None,
        }
    }
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

/// Lee el tamaño residente de memoria (RSS) en KiB desde `/proc/self/status`.
pub fn get_process_rss_kb() -> u64 {
    if let Ok(file) = File::open("/proc/self/status") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb;
                    }
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_initial() {
        let state = DaemonState::new();
        assert_eq!(state.state, State::Disconnected);
        assert_eq!(state.divert_epoch, 0);
        assert!(state.mapping.contains_key(&TOP_BUTTON_CID));

        let status = state.to_status_data();
        assert!(!status.connected);

        let health = state.to_health();
        assert_eq!(health.state, "Disconnected");
        assert!(health.rss_kb > 0, "VmRSS debe ser mayor a 0 en Linux");
    }
}
