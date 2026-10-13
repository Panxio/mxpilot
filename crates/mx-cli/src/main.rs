use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use mx_core::device::HidppDevice;
use mx_core::features::{
    DivertedButtonsEvent, CID_TOP_BUTTON, FEATURE_REPROGRAMMABLE_KEYS,
};
use mx_core::frame::LONG_REPORT_ID;
use mx_core::hidraw::HidrawTransport;
use mx_core::scanner::scan_hidraw_devices;
use mx_core::Transport;
use mx_uinput::{Action, VirtualInput};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn sigint_handler(_: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn main() {
    env_logger::init();

    // Registrar manejador de señal SIGINT (Ctrl+C) para salida limpia
    unsafe {
        libc::signal(libc::SIGINT, sigint_handler as *const () as usize);
        libc::signal(libc::SIGTERM, sigint_handler as *const () as usize);
    }

    println!("=== MX Options Linux CLI (mx-cli) ===");

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("divert");

    // Buscar dispositivo Logitech MX Vertical
    let devices = scan_hidraw_devices();
    let target = devices.iter().find(|d| d.is_mx_vertical).or_else(|| devices.iter().find(|d| d.is_logitech));

    let target_dev = match target {
        Some(d) => d,
        None => {
            eprintln!("Error: No se detectó ningún ratón Logitech compatible en /dev/hidraw*.");
            std::process::exit(1);
        }
    };

    println!("Dispositivo: {} ({})", target_dev.path.display(), target_dev.name);

    match command {
        "status" => run_status(&target_dev.path),
        "reset" => run_reset(&target_dev.path),
        _ => run_divert(&target_dev.path, &args[2..]),
    }
}

fn run_status(path: &PathBuf) {
    println!("Conectando a {}...", path.display());
    let transport = match HidrawTransport::open(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error abriendo {}: {}", path.display(), e);
            std::process::exit(2);
        }
    };

    let mut device = HidppDevice::new(transport, 0xFF);
    match device.read_battery() {
        Ok(bat) => println!("Batería: {}% ({:?})", bat.percentage, bat.state),
        Err(e) => println!("Error leyendo batería: {}", e),
    }

    match device.list_reprogrammable_keys() {
        Ok(keys) => {
            println!("Botones reprogramables ({}):", keys.len());
            for k in keys {
                let desc = if k.control_id == CID_TOP_BUTTON {
                    " <-- Botón Superior DPI Switch"
                } else {
                    ""
                };
                println!(
                    "  - #{}: CID 0x{:04X}, Divert: {}{}",
                    k.key_index,
                    k.control_id,
                    if k.can_divert { "SÍ" } else { "NO" },
                    desc
                );
            }
        }
        Err(e) => println!("Error listando teclas: {}", e),
    }
}

fn run_reset(path: &PathBuf) {
    println!("Restaurando divert en todos los botones de {}...", path.display());
    let transport = match HidrawTransport::open(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error abriendo {}: {}", path.display(), e);
            std::process::exit(2);
        }
    };

    let mut device = HidppDevice::new(transport, 0xFF);
    match device.set_cid_divert(CID_TOP_BUTTON, false) {
        Ok(_) => println!("[OK] Botón superior restaurado al firmware."),
        Err(e) => eprintln!("Error restaurando botón superior: {}", e),
    }
}

fn run_divert(path: &PathBuf, extra_args: &[String]) {
    // Parseo de acción solicitada (por defecto Super / Meta)
    let action_str = extra_args.first().map(|s| s.as_str()).unwrap_or("super");
    let action = match action_str {
        "copy" => Action::copy(),
        "paste" => Action::paste(),
        "alt-tab" => Action::alt_tab(),
        _ => Action::super_key(),
    };

    let target_cid = CID_TOP_BUTTON; // Botón superior 0x00FD

    println!("Inicializando emulador uinput...");
    let mut virtual_input = match VirtualInput::new() {
        Ok(vi) => {
            println!("[OK] Dispositivo virtual kernel creado: 'MX Options Virtual Device'");
            vi
        }
        Err(e) => {
            eprintln!("Error inicializando /dev/uinput: {}", e);
            eprintln!("[TIP] Asegúrate de que las reglas udev estén instaladas para uinput.");
            std::process::exit(2);
        }
    };

    println!("Conectando con el ratón en {}...", path.display());
    let transport = match HidrawTransport::open(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error abriendo {}: {}", path.display(), e);
            std::process::exit(2);
        }
    };

    let mut device = HidppDevice::new(transport, 0xFF);

    // Obtener el índice dinámico de Feature 0x1B04
    let reprog_feature_idx = match device.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Error: Feature 0x1B04 no encontrada: {}", e);
            std::process::exit(2);
        }
    };

    println!("Activando desvío (Button Diversion) en CID 0x{:04X}...", target_cid);
    if let Err(e) = device.set_cid_divert(target_cid, true) {
        eprintln!("Error activando divert en el hardware: {}", e);
        std::process::exit(2);
    }

    println!("=======================================================");
    println!(">>> BUTTON DIVERSION ACTIVO <<<");
    println!("Botón: Superior DPI Switch (CID 0x{:04X})", target_cid);
    println!("Acción asignada: {:?}", action);
    println!("Presiona el botón superior del ratón para probar.");
    println!("Presiona Ctrl+C para salir y restaurar el control original.");
    println!("=======================================================");

    let mut was_pressed = false;

    // Bucle de captura de eventos
    while RUNNING.load(Ordering::SeqCst) {
        // Escucha con timeout corto para verificar la señal de parada
        match device.transport.receive(Duration::from_millis(100)) {
            Ok(frame) => {
                // Notificación de divertedButtonsEvent (Feature 0x1B04, function 0)
                if frame.report_id == LONG_REPORT_ID
                    && frame.feature_index == reprog_feature_idx
                    && frame.function_id == 0
                {
                    let event = DivertedButtonsEvent::parse(&frame.params);
                    let is_pressed = event.is_cid_pressed(target_cid);

                    if is_pressed && !was_pressed {
                        println!("\n[EVENTO] Botón físico PRESIONADO (Down)");
                        println!(" -> Inyectando evento uinput: {:?}", action);
                        if let Err(e) = virtual_input.emit_action(&action) {
                            eprintln!("Error emitiendo evento uinput: {}", e);
                        }
                        was_pressed = true;
                    } else if !is_pressed && was_pressed {
                        println!("[EVENTO] Botón físico LIBERADO (Up)");
                        was_pressed = false;
                    }
                }
            }
            Err(mx_core::error::HidppError::Timeout) => {
                // Timeout normal, continúa el bucle
            }
            Err(mx_core::error::HidppError::UnknownReportId(_)) => {
                // Ignorar reportes de movimiento estándar
            }
            Err(e) => {
                eprintln!("Aviso de lectura: {}", e);
            }
        }
    }

    println!("\n\nCerrando aplicación... Restaurando firmware del mouse...");
    if let Err(e) = device.set_cid_divert(target_cid, false) {
        eprintln!("Aviso: No se pudo restaurar divert en el hardware: {}", e);
    } else {
        println!("[OK] Button Diversion desactivado en el mouse. Firmware restaurado.");
    }

    println!("[OK] Dispositivo virtual uinput liberado limpiamente.");
}
