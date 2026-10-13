use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use mx_core::scan_hidraw_devices;

fn main() {
    println!("=======================================================");
    println!("        MX Options Linux — Diagnóstico (mx-diag)        ");
    println!("=======================================================\n");

    // 1. Información del Sistema y Sesión
    println!("--- 1. Entorno del Sistema ---");
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Desconocido".into());
    let current_desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "Desconocido".into());
    println!("  Sesión gráfica: {} (Escritorio: {})", session_type, current_desktop);

    if let Ok(output) = Command::new("uname").arg("-sr").output() {
        let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("  Kernel Linux: {}", kernel);
    }

    // 2. Comprobación de Servicios Rivales (Convivencia)
    println!("\n--- 2. Comprobación de Servicios Rivales / Convivencia ---");
    check_process("solaar", "Solaar (Gestor Logitech GTK)");
    check_process("logid", "logiops (Daemon C++ Logitech)");

    // 3. Permisos de Kernel: /dev/uinput
    println!("\n--- 3. Acceso a /dev/uinput ---");
    let uinput_path = PathBuf::from("/dev/uinput");
    if uinput_path.exists() {
        match fs::OpenOptions::new().write(true).open(&uinput_path) {
            Ok(_) => println!("  /dev/uinput: [OK] Accesible en escritura con el usuario actual."),
            Err(e) => {
                println!("  /dev/uinput: [BLOQUEADO] {}", e);
                println!("  -> Se requiere regla udev con TAG+=\"uaccess\" para operar sin root.");
            }
        }
    } else {
        println!("  /dev/uinput: [ERROR] El módulo uinput no parece estar cargado.");
    }

    // 4. Dispositivos HIDRAW y Nodos
    println!("\n--- 4. Nodos /dev/hidraw y Permisos ---");
    let devices = scan_hidraw_devices();
    if devices.is_empty() {
        println!("  No se detectaron dispositivos hidraw.");
    } else {
        for d in &devices {
            let accessible = fs::OpenOptions::new().read(true).open(&d.path).is_ok();
            let status = if accessible { "[LEÍBLE]" } else { "[BLOQUEADO: Permiso denegado]" };
            let tag = if d.is_mx_vertical {
                " <-- TARGET: MX VERTICAL"
            } else if d.is_logitech {
                " <-- LOGITECH"
            } else {
                ""
            };
            println!("  - {}: {} {} (0x{:04X}:0x{:04X}){}", d.path.display(), status, d.name, d.vendor_id, d.product_id, tag);
        }
    }

    // 5. Estado Bluetooth
    println!("\n--- 5. Dispositivos Bluetooth Vinculados ---");
    if let Ok(output) = Command::new("bluetoothctl").arg("devices").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            println!("  {}", line);
        }
    }

    println!("\n=======================================================");
    println!("Diagnóstico completado.");
    println!("=======================================================");
}

fn check_process(name: &str, label: &str) {
    let output = Command::new("pgrep").arg("-x").arg(name).output();
    match output {
        Ok(out) if out.status.success() => {
            println!("  [ALERTA] Proceso '{}' ({}) está EN EJECUCIÓN.", name, label);
            println!("           Podría haber interferencia al disputar eventos HID++.");
        }
        _ => {
            println!("  Proceso '{}' ({}) no está activo. [Limpio]", name, label);
        }
    }
}
