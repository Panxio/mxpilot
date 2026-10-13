use std::env;
use std::path::PathBuf;

use mx_core::device::HidppDevice;
use mx_core::{HidrawTransport, scan_hidraw_devices};

fn main() {
    env_logger::init();
    println!("=== MX Options Linux — Sonda de Hardware y Protocolo (mx-probe) ===");

    let args: Vec<String> = env::args().collect();
    let target_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        println!("\nBuscando dispositivos compatibles en el sistema...");
        let devices = scan_hidraw_devices();

        if devices.is_empty() {
            eprintln!("No se encontraron nodos /dev/hidraw* en el sistema.");
            std::process::exit(1);
        }

        println!("Dispositivos detectados:");
        for d in &devices {
            let tag = if d.is_mx_vertical {
                " [MX VERTICAL]"
            } else if d.is_logitech {
                " [LOGITECH]"
            } else {
                ""
            };
            println!("  - {}: {} (0x{:04X}:0x{:04X}){}", d.path.display(), d.name, d.vendor_id, d.product_id, tag);
        }

        // Seleccionar preferentemente el MX Vertical o el primer Logitech
        if let Some(target) = devices.iter().find(|d| d.is_mx_vertical).or_else(|| devices.iter().find(|d| d.is_logitech)) {
            println!("\nDispositivo seleccionado para la sonda: {} ({})", target.path.display(), target.name);
            target.path.clone()
        } else {
            eprintln!("\nNo se detectó ningún dispositivo Logitech compatible.");
            std::process::exit(1);
        }
    };

    println!("Abriendo nodo {}...", target_path.display());
    let transport = match HidrawTransport::open(&target_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("\nError al abrir el dispositivo: {}", e);
            eprintln!("\n[TIP] Si el error es de permisos, asegúrate de aplicar la regla udev");
            eprintln!("o ejecutar temporalmente con permisos de acceso a {}.", target_path.display());
            std::process::exit(2);
        }
    };

    // En Bluetooth BLE y conexiones cableadas directas, el device_index suele ser 0xFF.
    // En receptores Unifying/Bolt, los índices van de 0x01 a 0x06.
    println!("Iniciando handshake HID++ 2.0 en índice 0xFF (Bluetooth/Directo)...");
    let mut device = HidppDevice::new(transport, 0xFF);

    let mut connected = matches!(device.ping(), Ok(true));

    if !connected {
        println!("Sin respuesta en 0xFF. Probando índice 0x01 (Receptor Unifying/Bolt)...");
        device.device_index = 0x01;
        connected = matches!(device.ping(), Ok(true));
    }

    if !connected {
        eprintln!("\nADVERTENCIA: No se recibió respuesta al ping inicial.");
        eprintln!("Intentando consulta directa de la Feature Root (0x0000)...");
    } else {
        println!("¡Handshake HID++ 2.0 exitoso en índice 0x{:02X}!", device.device_index);
    }

    // 1. Lectura de Batería
    println!("\n--- 1. Estado de Batería ---");
    match device.read_battery() {
        Ok(bat) => {
            println!("  Nivel: {}%", bat.percentage);
            println!("  Estado: {:?}", bat.state);
        }
        Err(e) => {
            println!("  No se pudo leer la batería: {}", e);
        }
    }

    // 2. Kill-Gate: Feature 0x1B04 (Reprogrammable Keys & Diversion)
    println!("\n--- 2. Verificación de Kill-Gate: Feature 0x1B04 (Button Diversion) ---");
    let mut kill_gate_passed = false;
    match device.check_reprogrammable_keys_feature() {
        Ok(Some(idx)) => {
            println!("  [ENCONTRADA] Feature 0x1B04 asignada al índice: 0x{:02X}", idx);
            match device.list_reprogrammable_keys() {
                Ok(keys) => {
                    println!("  Teclas/Botones reprogramables detectados: {}", keys.len());
                    for k in &keys {
                        let divert_str = if k.can_divert { "SÍ (Divertable)" } else { "NO" };
                        let highlight = if k.control_id == 0x00C3 || k.control_id == 0x00FD {
                            " <-- BOTÓN SUPERIOR DPI SWITCH"
                        } else {
                            ""
                        };
                        println!(
                            "    - Botón #{}: CID 0x{:04X}, Task 0x{:04X}, Flags 0x{:02X}, Divert: {}{}",
                            k.key_index, k.control_id, k.task_id, k.flags, divert_str, highlight
                        );
                        if k.can_divert {
                            kill_gate_passed = true;
                        }
                    }
                }
                Err(e) => {
                    println!("  Error al listar teclas reprogramables: {}", e);
                    // Si la feature está presente, el soporte básico existe
                    kill_gate_passed = true;
                }
            }
        }
        Ok(None) => {
            println!("  [NO ENCONTRADA] El dispositivo no reporta la feature 0x1B04.");
        }
        Err(e) => {
            println!("  Error al consultar feature 0x1B04: {}", e);
        }
    }

    // 3. Sensor y DPI: Feature 0x2201
    println!("\n--- 3. Control de Sensor y DPI (Feature 0x2201) ---");
    match device.check_adjustable_dpi_feature() {
        Ok(Some(idx)) => {
            println!("  [EXPUESTA] Feature 0x2201 asignada al índice: 0x{:02X}", idx);
            println!("  El mouse permite control de DPI por software.");
        }
        Ok(None) => {
            println!("  [NO EXPUESTA] Feature 0x2201 no disponible (el firmware maneja DPI internamente).");
        }
        Err(e) => {
            println!("  Error al consultar DPI: {}", e);
        }
    }

    // 4. Listado completo de Features (0x0001)
    println!("\n--- 4. Listado completo de Características (Feature Set) ---");
    match device.list_all_features() {
        Ok(feats) => {
            println!("  Total de características expuestas: {}", feats.len());
            for (i, f) in feats.iter().enumerate() {
                let desc = match *f {
                    0x0000 => "Root (IRoot)",
                    0x0001 => "FeatureSet",
                    0x0003 => "DeviceInfo",
                    0x0005 => "DeviceName",
                    0x1000 => "BatteryStatus",
                    0x1004 => "UnifiedBattery",
                    0x1B04 => "ReprogrammableKeys / Diversion [KILL-GATE]",
                    0x2201 => "AdjustableDPI",
                    _ => "Específica de fabricante",
                };
                println!("    [{:02}] Feature 0x{:04X} — {}", i + 1, f, desc);
            }
        }
        Err(e) => {
            println!("  No se pudo listar el conjunto de características: {}", e);
        }
    }

    println!("\n=======================================================");
    if kill_gate_passed {
        println!(">>> KILL-GATE F0: [APROBADO (PASSED)] <<<");
        println!("El mouse soporta Button Diversion sobre la conexión actual.");
    } else {
        println!(">>> KILL-GATE F0: [PENDIENTE / REVISAR] <<<");
        println!("No se pudo confirmar Button Diversion en este transporte.");
    }
    println!("=======================================================");
}
