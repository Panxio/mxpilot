use crate::ipc::protocol::{is_valid_cid, TOP_BUTTON_CID};
use crate::state::{DaemonState, State};
use evdev::KeyCode;
use mx_core::device::HidppDevice;
use mx_core::error::HidppError;
use mx_core::features::{DivertedButtonsEvent, FEATURE_REPROGRAMMABLE_KEYS};
use mx_core::frame::LONG_REPORT_ID;
use mx_core::hidraw::HidrawTransport;
use mx_core::scanner::scan_hidraw_devices;
use mx_core::transport::Transport;
use mx_uinput::{Action, VirtualInput};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Eventos enviados desde el hilo HID++ hacia el despachador uinput.
#[derive(Debug, Clone)]
pub enum InputEvent {
    Down { epoch: u64, action: Action },
    Up { epoch: u64, action: Action },
    ReleaseAll,
}

/// Comandos recibidos por `hid_worker` desde el servidor IPC.
pub enum DaemonCommand {
    SetAction {
        cid: u16,
        action: Action,
        divert: bool,
        reply: SyncSender<Result<(), String>>,
    },
    RefreshBattery {
        reply: SyncSender<Result<(u8, String), String>>,
    },
    Shutdown,
}

/// Extrae todos los KeyCodes involucrados en una Action.
fn extract_keys_from_action(action: &Action) -> Vec<KeyCode> {
    match action {
        Action::SingleKey(k) => vec![*k],
        Action::KeyCombo { modifiers, key } => {
            let mut list = modifiers.clone();
            list.push(*key);
            list
        }
        Action::MouseButton(b) => vec![*b],
        Action::None => Vec::new(),
    }
}

/// Hilo trabajador de inyección uinput (Persistente durante toda la sesión).
pub fn run_uinput_worker(
    receiver: Receiver<InputEvent>,
    shutdown: Arc<AtomicBool>,
) {
    let mut vinput: Option<VirtualInput> = VirtualInput::new().ok();
    if vinput.is_none() {
        log::warn!("VirtualInput no disponible inicialmente (¿permisos uinput?). Se intentará bajo demanda.");
    }

    let mut pressed_keys: HashSet<KeyCode> = HashSet::new();
    let mut current_epoch: u64 = 0;

    while !shutdown.load(Ordering::Relaxed) {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => match event {
                InputEvent::Down { epoch, action } => {
                    if epoch < current_epoch {
                        log::debug!("Descartando evento Down de época obsoleta ({} < {})", epoch, current_epoch);
                        continue;
                    }
                    current_epoch = epoch;

                    if vinput.is_none() {
                        vinput = VirtualInput::new().ok();
                    }

                    if let Some(ref mut input) = vinput {
                        let keys = extract_keys_from_action(&action);
                        for k in keys {
                            pressed_keys.insert(k);
                        }
                        if let Err(e) = input.emit_action_down(&action) {
                            log::error!("Error emitiendo emit_action_down: {}", e);
                        }
                    }
                }
                InputEvent::Up { epoch, action } => {
                    if epoch < current_epoch {
                        log::debug!("Descartando evento Up de época obsoleta ({} < {})", epoch, current_epoch);
                        continue;
                    }

                    if let Some(ref mut input) = vinput {
                        let keys = extract_keys_from_action(&action);
                        for k in keys {
                            pressed_keys.remove(&k);
                        }
                        if let Err(e) = input.emit_action_up(&action) {
                            log::error!("Error emitiendo emit_action_up: {}", e);
                        }
                    }
                }
                InputEvent::ReleaseAll => {
                    log::info!("Fail-Safe: liberando {} teclas retenidas", pressed_keys.len());
                    if let Some(ref mut input) = vinput {
                        for k in &pressed_keys {
                            let _ = input.emit_release(*k);
                        }
                    }
                    pressed_keys.clear();
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Chequeo periódico de shutdown
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log::info!("Canal de entrada cerrado, saliendo uinput_worker.");
                break;
            }
        }
    }

    // Limpieza final al salir
    if let Some(ref mut input) = vinput {
        for k in &pressed_keys {
            let _ = input.emit_release(*k);
        }
    }
    pressed_keys.clear();
}

/// Hilo trabajador de hardware HID++ (Actor de Dueño Único).
pub fn run_hid_worker(
    cmd_receiver: Receiver<DaemonCommand>,
    input_sender: SyncSender<InputEvent>,
    state: Arc<RwLock<DaemonState>>,
    shutdown: Arc<AtomicBool>,
) {
    let mut backoff_ms: u64 = 1000;

    while !shutdown.load(Ordering::Relaxed) {
        // Atender posibles comandos IPC pendientes en estado desconectado
        drain_commands_disconnected(&cmd_receiver);

        // 1. Escaneo de nodos hidraw
        let devices = scan_hidraw_devices();
        let target_device = devices.into_iter().find(|d| d.is_mx_vertical);

        let node = match target_device {
            Some(dev) => dev,
            None => {
                // Actualizar estado a Disconnected
                {
                    let mut s = state.write().unwrap();
                    if s.state != State::Disconnected {
                        s.state = State::Disconnected;
                        s.device_name = None;
                        s.device_path = None;
                    }
                }

                // Dormir con backoff en porciones cortas para responder a shutdown
                let slice = Duration::from_millis(100);
                let iterations = (backoff_ms / 100).max(1);
                for _ in 0..iterations {
                    if shutdown.load(Ordering::Relaxed) {
                        return;
                    }
                    drain_commands_disconnected(&cmd_receiver);
                    thread::sleep(slice);
                }
                backoff_ms = (backoff_ms * 2).min(4000);
                continue;
            }
        };

        // Reseteo de backoff al encontrar dispositivo
        backoff_ms = 1000;
        log::info!("Dispositivo MX Vertical detectado en {}. Iniciando conexión...", node.path.display());

        // 2. Conectar
        {
            let mut s = state.write().unwrap();
            s.state = State::Connecting;
            s.device_path = Some(node.path.to_string_lossy().to_string());
            s.device_name = Some(node.name.clone());
        }

        let transport = match HidrawTransport::open(&node.path) {
            Ok(t) => t,
            Err(e) => {
                log::warn!("No se pudo abrir {}: {}. Reintentando...", node.path.display(), e);
                let mut s = state.write().unwrap();
                s.consecutive_failures += 1;
                s.last_error = Some(format!("Error abriendo {}: {}", node.path.display(), e));
                thread::sleep(Duration::from_millis(500));
                continue;
            }
        };

        let mut device = match HidppDevice::auto_connect(transport) {
            Ok(dev) => dev,
            Err(e) => {
                log::warn!("Fallo handshake auto_connect con {}: {}. Reintentando...", node.path.display(), e);
                let mut s = state.write().unwrap();
                s.consecutive_failures += 1;
                s.last_error = Some(format!("Fallo auto_connect: {}", e));
                thread::sleep(Duration::from_millis(500));
                continue;
            }
        };

        // 3. Reconciliación
        {
            let mut s = state.write().unwrap();
            s.state = State::Reconciling;
        }

        log::info!("Reconciliando hardware (reset_all_diverts)...");
        let _ = device.reset_all_diverts();

        let reprog_feature_idx = match device.get_feature_index(FEATURE_REPROGRAMMABLE_KEYS) {
            Ok(idx) => idx,
            Err(e) => {
                log::warn!("Feature 0x1B04 no disponible: {}. Desconectando.", e);
                let mut s = state.write().unwrap();
                s.consecutive_failures += 1;
                s.last_error = Some(format!("Feature 0x1B04 no disponible: {}", e));
                continue;
            }
        };

        log::info!("Aplicando divert en botón superior (CID 0x{:04X})...", TOP_BUTTON_CID);
        if let Err(e) = device.set_cid_divert(TOP_BUTTON_CID, true) {
            log::error!("Fallo al activar divert en 0x{:04X}: {}", TOP_BUTTON_CID, e);
            let mut s = state.write().unwrap();
            s.consecutive_failures += 1;
            s.last_error = Some(format!("Fallo activando divert: {}", e));
            s.state = State::Disconnected;
            continue;
        }

        // 4. Entrar en ACTIVE solo tras confirmación real del hardware
        let epoch = {
            let mut s = state.write().unwrap();
            s.divert_epoch += 1;
            s.reconnect_count += 1;
            s.consecutive_failures = 0;
            s.state = State::Active;
            s.device_index = Some(device.device_index());
            s.diverted_cids = vec![TOP_BUTTON_CID];
            s.last_error = None;
            s.divert_epoch
        };

        // Lectura inicial de batería
        if let Ok(battery) = device.read_battery() {
            let mut s = state.write().unwrap();
            s.battery_level = Some(battery.percentage);
            s.battery_status = Some(format!("{:?}", battery.state));
            s.battery_last_updated = Some(Instant::now());
        }

        log::info!(">>> DAEMON ACTIVO EN ÉPOCA {} <<< (Capturando eventos en {})", epoch, node.path.display());

        // 5. Bucle Activo de Eventos HID++ (guarda la acción exacta en Down para liberar la misma en Up)
        let mut tracked_pressed: HashMap<u16, Option<Action>> = HashMap::new();
        tracked_pressed.insert(TOP_BUTTON_CID, None);
        let mut active_diverted_cids: HashSet<u16> = HashSet::new();
        active_diverted_cids.insert(TOP_BUTTON_CID);

        // Re-aplicar divert a todos los CIDs mapeados en state previamente
        {
            let s = state.read().unwrap();
            for &cid in s.mapping.keys() {
                if is_valid_cid(cid) && cid != TOP_BUTTON_CID {
                    if let Ok(()) = device.set_cid_divert(cid, true) {
                        active_diverted_cids.insert(cid);
                        tracked_pressed.insert(cid, None);
                    }
                }
            }
        }
        {
            let mut s = state.write().unwrap();
            s.diverted_cids = active_diverted_cids.iter().copied().collect();
        }

        while !shutdown.load(Ordering::Relaxed) {
            // Atender comandos IPC
            while let Ok(cmd) = cmd_receiver.try_recv() {
                match cmd {
                    DaemonCommand::SetAction { cid, action: _, divert, reply } => {
                        if is_valid_cid(cid) {
                            let res = device.set_cid_divert(cid, divert).map_err(|e| e.to_string());
                            if res.is_ok() {
                                let mut s = state.write().unwrap();
                                if divert {
                                    active_diverted_cids.insert(cid);
                                    tracked_pressed.entry(cid).or_insert(None);
                                    if !s.diverted_cids.contains(&cid) {
                                        s.diverted_cids.push(cid);
                                    }
                                } else {
                                    active_diverted_cids.remove(&cid);
                                    tracked_pressed.remove(&cid);
                                    s.diverted_cids.retain(|&c| c != cid);
                                    s.mapping.remove(&cid);
                                }
                            }
                            let _ = reply.send(res);
                        } else {
                            let _ = reply.send(Err(format!("CID 0x{:04X} no soportado para desvío", cid)));
                        }
                    }
                    DaemonCommand::RefreshBattery { reply } => {
                        match device.read_battery() {
                            Ok(bat) => {
                                {
                                    let mut s = state.write().unwrap();
                                    s.battery_level = Some(bat.percentage);
                                    s.battery_status = Some(format!("{:?}", bat.state));
                                    s.battery_last_updated = Some(Instant::now());
                                }
                                let _ = reply.send(Ok((bat.percentage, format!("{:?}", bat.state))));
                            }
                            Err(e) => {
                                let _ = reply.send(Err(e.to_string()));
                            }
                        }
                    }
                    DaemonCommand::Shutdown => {
                        break;
                    }
                }
            }

            // Escuchar reporte de hardware
            match device.transport_mut().receive(Duration::from_millis(100)) {
                Ok(frame) => {
                    if frame.report_id == LONG_REPORT_ID
                        && frame.feature_index == reprog_feature_idx
                        && frame.function_id == 0
                    {
                        let event = DivertedButtonsEvent::parse(&frame.params);
                        for &cid in &active_diverted_cids {
                            let is_pressed = event.is_cid_pressed(cid);
                            let was_pressed = tracked_pressed.get(&cid).and_then(|opt| opt.as_ref()).is_some();

                            if is_pressed && !was_pressed {
                                let current_action = {
                                    let s = state.read().unwrap();
                                    s.mapping.get(&cid).and_then(|p| p.to_action().ok()).unwrap_or_else(|| {
                                        if cid == TOP_BUTTON_CID {
                                            Action::super_key()
                                        } else {
                                            Action::None
                                        }
                                    })
                                };
                                tracked_pressed.insert(cid, Some(current_action.clone()));
                                if input_sender.send(InputEvent::Down {
                                    epoch,
                                    action: current_action,
                                }).is_err() {
                                    let mut s = state.write().unwrap();
                                    s.dropped_events += 1;
                                }
                            } else if !is_pressed && was_pressed {
                                if let Some(pressed_action) = tracked_pressed.insert(cid, None).flatten() {
                                    if input_sender.send(InputEvent::Up {
                                        epoch,
                                        action: pressed_action,
                                    }).is_err() {
                                        let mut s = state.write().unwrap();
                                        s.dropped_events += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(HidppError::Timeout) => {
                    // Normal timeout
                }
                Err(HidppError::Disconnected) => {
                    log::warn!("Desconexión de hardware detectada en {}. Emitiendo ReleaseAll.", node.path.display());
                    let _ = input_sender.send(InputEvent::ReleaseAll);
                    break;
                }
                Err(e) => {
                    log::debug!("Error en lectura de transporte: {}", e);
                }
            }
        }

        // Salida limpia de conexión
        log::info!("Cerrando sesión de divert en época {}...", epoch);
        let _ = input_sender.send(InputEvent::ReleaseAll);
        for &cid in &active_diverted_cids {
            let _ = device.set_cid_divert(cid, false);
        }
    }

    // Salida global del daemon
    let _ = input_sender.send(InputEvent::ReleaseAll);
}

/// Drena y responde negativamente a comandos cuando no hay hardware conectado.
fn drain_commands_disconnected(cmd_receiver: &Receiver<DaemonCommand>) {
    while let Ok(cmd) = cmd_receiver.try_recv() {
        match cmd {
            DaemonCommand::SetAction { reply, .. } => {
                let _ = reply.send(Err("Dispositivo no conectado".to_string()));
            }
            DaemonCommand::RefreshBattery { reply } => {
                let _ = reply.send(Err("Dispositivo no conectado".to_string()));
            }
            DaemonCommand::Shutdown => {}
        }
    }
}
