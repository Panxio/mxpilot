#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{info, warn};
use mx_gui::client::DaemonClient;
use mx_gui::commands::{self, AppState};
use mx_gui::poller;
use mx_gui::tray;
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

fn main() {
    let client = Arc::new(DaemonClient::new());
    let app_state = AppState {
        client: Arc::clone(&client),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_health,
            commands::get_battery,
            commands::set_action,
            commands::start_daemon_service,
            commands::get_gui_memory,
        ])
        .setup(move |app| {
            // Intentar inicializar system tray de forma fallible
            if let Err(e) = tray::setup_tray(app.handle()) {
                warn!("Bandeja del sistema no disponible: {}", e);
            }

            // Iniciar poller en segundo plano
            poller::start_poller(app.handle().clone(), client);

            // Obtener o crear ventana principal de forma garantizada
            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    info!("Creando ventana 'main' dinámicamente");
                    WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                        .title("MX Options Linux")
                        .inner_size(760.0, 540.0)
                        .resizable(false)
                        .build()?
                }
            };

            let _ = window.center();
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();

            let win = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    if !tray::is_quitting() && tray::has_tray() {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                }
            });

            info!("mx-gui inicializado exitosamente");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error ejecutando aplicación Tauri");
}
