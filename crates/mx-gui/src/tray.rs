use log::warn;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

static IS_QUITTING: AtomicBool = AtomicBool::new(false);
static HAS_TRAY: AtomicBool = AtomicBool::new(false);

pub fn is_quitting() -> bool {
    IS_QUITTING.load(Ordering::SeqCst)
}

pub fn set_quitting(val: bool) {
    IS_QUITTING.store(val, Ordering::SeqCst);
}

pub fn has_tray() -> bool {
    HAS_TRAY.load(Ordering::SeqCst)
}

/// Inicializa el System Tray de forma fallible (tolerante a entornos sin AppIndicator).
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Mostrar MX Options", true, None::<&str>)?;
    let battery_item = MenuItem::with_id(app, "battery", "Batería: --", false, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart_daemon", "Reiniciar Daemon", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &battery_item,
            &restart_item,
            &quit_item,
        ],
    )?;

    let app_handle = app.clone();
    let icon = app.default_window_icon().cloned().unwrap_or_else(|| {
        tauri::include_image!("icons/32x32.png")
    });

    match TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("MX Options Linux")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "restart_daemon" => {
                    let _ = std::process::Command::new("/usr/bin/systemctl")
                        .args(["--user", "restart", "mx-options.service"])
                        .spawn();
                }
                "quit" => {
                    set_quitting(true);
                    crate::poller::stop_poller();
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = app_handle.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)
    {
        Ok(_) => {
            HAS_TRAY.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(e) => {
            warn!("System Tray no disponible en el entorno de escritorio: {}. La app operará en modo ventana", e);
            HAS_TRAY.store(false, Ordering::SeqCst);
            Err(Box::new(e))
        }
    }
}
