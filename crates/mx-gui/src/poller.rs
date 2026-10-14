use crate::client::DaemonClient;
use log::{debug, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const POLL_INTERVAL_ACTIVE: Duration = Duration::from_secs(10);
pub const POLL_INTERVAL_BACKGROUND: Duration = Duration::from_secs(60);

static STOP_POLLER: AtomicBool = AtomicBool::new(false);

/// Detiene la ejecución del hilo de sondeo.
pub fn stop_poller() {
    STOP_POLLER.store(true, Ordering::SeqCst);
}

/// Inicia la tarea de sondeo periódica desacoplada del event loop de GTK.
pub fn start_poller(app: AppHandle, client: Arc<DaemonClient>) {
    STOP_POLLER.store(false, Ordering::SeqCst);
    if let Err(e) = thread::Builder::new()
        .name("mx-gui-poller".into())
        .spawn(move || {
            debug!("Iniciando hilo de sondeo continuo mx-gui");
            while !STOP_POLLER.load(Ordering::SeqCst) {
                // Determinar cadencia según la visibilidad de la ventana
                let is_visible = app
                    .get_webview_window("main")
                    .and_then(|w| w.is_visible().ok())
                    .unwrap_or(true);

                let interval = if is_visible {
                    POLL_INTERVAL_ACTIVE
                } else {
                    POLL_INTERVAL_BACKGROUND
                };

                // Espera en micro-bloques para responder inmediatamente ante salida
                let step = Duration::from_millis(500);
                let mut waited = Duration::ZERO;
                while waited < interval {
                    if STOP_POLLER.load(Ordering::SeqCst) {
                        return;
                    }
                    thread::sleep(step);
                    waited += step;
                }

                // Ejecutar sondeo del estado del daemon
                match client.get_status() {
                    Ok(status) => {
                        let _ = app.emit("status-changed", &status);
                        crate::tray::update_tray_status(&app, &status);
                    }
                    Err(err) => {
                        let err_str = err.to_string();
                        let _ = app.emit("status-error", &err_str);
                        crate::tray::update_tray_error(&app, &err_str);
                    }
                }
            }
        })
    {
        warn!("No se pudo inicializar el hilo de sondeo de mx-gui: {}", e);
    }
}
