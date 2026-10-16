use log::warn;
use mx_daemon::ipc::protocol::StatusData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

static IS_QUITTING: AtomicBool = AtomicBool::new(false);
static HAS_TRAY: AtomicBool = AtomicBool::new(false);
static SHOW_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);
static BATTERY_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);
static RESTART_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);
static QUIT_ITEM: Mutex<Option<MenuItem<tauri::Wry>>> = Mutex::new(None);
static CURRENT_LOCALE: Mutex<Option<String>> = Mutex::new(None);
static LAST_STATUS: Mutex<Option<Result<StatusData, String>>> = Mutex::new(None);

pub const MAIN_TRAY_ID: &str = "main-tray";

pub fn is_quitting() -> bool {
    IS_QUITTING.load(Ordering::SeqCst)
}

pub fn set_quitting(val: bool) {
    IS_QUITTING.store(val, Ordering::SeqCst);
}

pub fn has_tray() -> bool {
    HAS_TRAY.load(Ordering::SeqCst)
}

fn detect_env_locale() -> String {
    let raw = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "es".to_string());
    if raw.to_lowercase().starts_with("es") {
        "es".to_string()
    } else {
        "en".to_string()
    }
}

pub fn get_tray_locale() -> String {
    if let Ok(guard) = CURRENT_LOCALE.lock() {
        if let Some(ref loc) = *guard {
            return loc.clone();
        }
    }
    detect_env_locale()
}

fn static_labels(locale: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    if locale == "en" {
        ("Show MX Options", "Battery: --", "Restart Daemon", "Quit")
    } else {
        ("Mostrar MX Options", "Batería: --", "Reiniciar Daemon", "Salir")
    }
}

/// Cambia el idioma del System Tray en caliente y refresca todos los textos del menú.
pub fn set_tray_locale(app: &AppHandle, locale: &str) {
    let valid = if locale == "en" { "en" } else { "es" };
    if let Ok(mut guard) = CURRENT_LOCALE.lock() {
        *guard = Some(valid.to_string());
    }

    if !has_tray() {
        return;
    }

    let (show_txt, _, restart_txt, quit_txt) = static_labels(valid);

    if let Ok(guard) = SHOW_ITEM.lock() {
        if let Some(ref item) = *guard {
            let _ = item.set_text(show_txt);
        }
    }
    if let Ok(guard) = RESTART_ITEM.lock() {
        if let Some(ref item) = *guard {
            let _ = item.set_text(restart_txt);
        }
    }
    if let Ok(guard) = QUIT_ITEM.lock() {
        if let Some(ref item) = *guard {
            let _ = item.set_text(quit_txt);
        }
    }

    let cached = LAST_STATUS.lock().ok().and_then(|g| g.clone());
    match cached {
        Some(Ok(ref status)) => render_tray_status(app, status, valid),
        Some(Err(ref err)) => render_tray_error(app, err, valid),
        None => {
            let (_, batt_txt, _, _) = static_labels(valid);
            if let Ok(guard) = BATTERY_ITEM.lock() {
                if let Some(ref item) = *guard {
                    let _ = item.set_text(batt_txt);
                }
            }
        }
    }
}

fn render_tray_status(app: &AppHandle, status: &StatusData, locale: &str) {
    let is_en = locale == "en";
    let (battery_text, tooltip_text) = match (status.connected, status.battery_level) {
        (true, Some(lvl)) => {
            let is_charging = status
                .battery_status
                .as_deref()
                .map(|st| st.to_lowercase().contains("charg") || st.to_lowercase().contains("carg"))
                .unwrap_or(false);

            let txt = match (is_en, is_charging) {
                (true, true) => format!("Battery: {}% (Charging)", lvl),
                (true, false) => format!("Battery: {}%", lvl),
                (false, true) => format!("Batería: {}% (Cargando)", lvl),
                (false, false) => format!("Batería: {}%", lvl),
            };
            let tip = format!("MX Options - {}%", lvl);
            (txt, tip)
        }
        (true, None) => {
            if is_en {
                (
                    "Battery: Detecting...".to_string(),
                    "MX Options - Connected".to_string(),
                )
            } else {
                (
                    "Batería: Detectando...".to_string(),
                    "MX Options - Conectado".to_string(),
                )
            }
        }
        (false, _) => {
            if is_en {
                (
                    "Battery: Disconnected".to_string(),
                    "MX Options - Disconnected".to_string(),
                )
            } else {
                (
                    "Batería: Desconectado".to_string(),
                    "MX Options - Desconectado".to_string(),
                )
            }
        }
    };

    if let Ok(guard) = BATTERY_ITEM.lock() {
        if let Some(ref item) = *guard {
            let _ = item.set_text(&battery_text);
        }
    }

    if let Some(tray) = app.tray_by_id(MAIN_TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip_text));
    }
}

fn render_tray_error(app: &AppHandle, _err: &str, locale: &str) {
    let (batt_txt, tip_txt) = if locale == "en" {
        ("Battery: Daemon Stopped", "MX Options - Daemon Stopped")
    } else {
        ("Batería: Daemon Detenido", "MX Options - Daemon Detenido")
    };

    if let Ok(guard) = BATTERY_ITEM.lock() {
        if let Some(ref item) = *guard {
            let _ = item.set_text(batt_txt);
        }
    }

    if let Some(tray) = app.tray_by_id(MAIN_TRAY_ID) {
        let _ = tray.set_tooltip(Some(tip_txt));
    }
}

/// Actualiza el estado visual del System Tray (tooltip y elemento de menú de batería)
pub fn update_tray_status(app: &AppHandle, status: &StatusData) {
    if let Ok(mut guard) = LAST_STATUS.lock() {
        *guard = Some(Ok(status.clone()));
    }
    if !has_tray() {
        return;
    }
    let locale = get_tray_locale();
    render_tray_status(app, status, &locale);
}

/// Actualiza el System Tray en caso de error o daemon no disponible
pub fn update_tray_error(app: &AppHandle, err: &str) {
    if let Ok(mut guard) = LAST_STATUS.lock() {
        *guard = Some(Err(err.to_string()));
    }
    if !has_tray() {
        return;
    }
    let locale = get_tray_locale();
    render_tray_error(app, err, &locale);
}

/// Inicializa el System Tray de forma fallible (tolerante a entornos sin AppIndicator).
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let locale = get_tray_locale();
    let (show_lbl, batt_lbl, restart_lbl, quit_lbl) = static_labels(&locale);

    let show_item = MenuItem::with_id(app, "show", show_lbl, true, None::<&str>)?;
    let battery_item = MenuItem::with_id(app, "battery", batt_lbl, false, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart_daemon", restart_lbl, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_lbl, true, None::<&str>)?;

    if let Ok(mut guard) = SHOW_ITEM.lock() {
        *guard = Some(show_item.clone());
    }
    if let Ok(mut guard) = BATTERY_ITEM.lock() {
        *guard = Some(battery_item.clone());
    }
    if let Ok(mut guard) = RESTART_ITEM.lock() {
        *guard = Some(restart_item.clone());
    }
    if let Ok(mut guard) = QUIT_ITEM.lock() {
        *guard = Some(quit_item.clone());
    }

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

    match TrayIconBuilder::with_id(MAIN_TRAY_ID)
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
                    tauri::async_runtime::spawn(async move {
                        let _ = std::process::Command::new("/usr/bin/systemctl")
                            .args(["--user", "restart", "mx-options.service"])
                            .status();
                    });
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


