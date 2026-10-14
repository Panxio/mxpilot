use crate::client::DaemonClient;
use mx_daemon::ipc::protocol::{ActionPayload, BatteryData, DaemonHealth, StatusData};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

/// Estado compartido inyectado en el contexto de Tauri.
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<DaemonClient>,
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusData, String> {
    let client = Arc::clone(&state.client);
    tauri::async_runtime::spawn_blocking(move || client.get_status())
        .await
        .map_err(|e| format!("Error en worker: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_health(state: State<'_, AppState>) -> Result<DaemonHealth, String> {
    let client = Arc::clone(&state.client);
    tauri::async_runtime::spawn_blocking(move || client.get_health())
        .await
        .map_err(|e| format!("Error en worker: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_battery(state: State<'_, AppState>) -> Result<BatteryData, String> {
    let client = Arc::clone(&state.client);
    tauri::async_runtime::spawn_blocking(move || client.get_battery())
        .await
        .map_err(|e| format!("Error en worker: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_action(
    cid: u16,
    action: ActionPayload,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = Arc::clone(&state.client);
    tauri::async_runtime::spawn_blocking(move || client.set_action(cid, action))
        .await
        .map_err(|e| format!("Error en worker: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_daemon_service(state: State<'_, AppState>) -> Result<String, String> {
    let client = Arc::clone(&state.client);
    tauri::async_runtime::spawn_blocking(move || {
        // Invocar systemctl sin shell de forma segura
        let mut cmd = Command::new("/usr/bin/systemctl");
        cmd.args(["--user", "start", "mx-options.service"]);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let status = cmd.status().map_err(|e| {
            format!("No se pudo ejecutar systemctl: {}", e)
        })?;

        if !status.success() {
            return Err(format!(
                "systemctl --user start mx-options falló con código: {:?}",
                status.code()
            ));
        }

        // Bucle de confirmación con backoff para verificar respuesta real del socket
        for _ in 0..15 {
            std::thread::sleep(Duration::from_millis(200));
            if client.ping().is_ok() {
                return Ok("Servicio mx-options iniciado y respondiendo".into());
            }
        }

        Err("systemctl reportó éxito pero el socket no respondió tras 3 segundos".into())
    })
    .await
    .map_err(|e| format!("Error en worker: {}", e))?
}

#[tauri::command]
pub async fn get_gui_memory() -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(read_own_rss_kb)
        .await
        .map_err(|e| format!("Error en worker de memoria: {}", e))?
}

/// Lee el VmRSS en KB desde /proc/self/status.
fn read_own_rss_kb() -> Result<u64, String> {
    let content = std::fs::read_to_string("/proc/self/status")
        .map_err(|e| format!("No se pudo leer /proc/self/status: {}", e))?;
    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(rss) = parts[1].parse::<u64>() {
                    return Ok(rss);
                }
            }
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_own_rss_kb() {
        let rss = read_own_rss_kb().expect("Debe leer VmRSS de /proc/self/status en Linux");
        assert!(rss > 0, "VmRSS debe ser mayor a 0 KB");
    }
}

