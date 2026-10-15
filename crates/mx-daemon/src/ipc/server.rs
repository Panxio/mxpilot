use super::protocol::*;
use crate::state::DaemonState;
use crate::workers::DaemonCommand;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

const MAX_CLIENTS: usize = 8;
const MAX_PAYLOAD_BYTES: usize = 65_536; // 64 KB
const INCOMPLETE_LINE_TIMEOUT: Duration = Duration::from_secs(2);
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const BATTERY_CACHE_TTL: Duration = Duration::from_secs(60);
const HW_REPLY_TIMEOUT: Duration = Duration::from_millis(2000);

/// Extrae el UID del cliente conectado usando getsockopt con SO_PEERCRED.
fn get_peer_uid(stream: &UnixStream) -> Result<libc::uid_t, std::io::Error> {
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let res = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if res == 0 {
        Ok(cred.uid)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

struct ClientConnection {
    stream: UnixStream,
    buffer: Vec<u8>,
    last_activity: Instant,
}

pub struct IpcServer {
    socket_path: PathBuf,
    listener: UnixListener,
    state: Arc<RwLock<DaemonState>>,
    cmd_sender: SyncSender<DaemonCommand>,
    shutdown: Arc<AtomicBool>,
}

impl IpcServer {
    /// Inicializa y enlaza el servidor Unix Domain Socket en `$XDG_RUNTIME_DIR/mx-options.sock`.
    pub fn new(
        state: Arc<RwLock<DaemonState>>,
        cmd_sender: SyncSender<DaemonCommand>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").map_err(|_| {
            "Variable XDG_RUNTIME_DIR no definida. mx-daemon requiere una sesión de usuario."
                .to_string()
        })?;
        Self::new_with_dir(Path::new(&runtime_dir), state, cmd_sender, shutdown)
    }

    /// Inicializa el servidor Unix Domain Socket en un directorio específico.
    pub fn new_with_dir(
        dir: &Path,
        state: Arc<RwLock<DaemonState>>,
        cmd_sender: SyncSender<DaemonCommand>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let socket_path = dir.join("mx-options.sock");

        // Limpiar socket previo huérfano (el singleton lock garantiza exclusividad previa)
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }

        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("Error enlazando socket IPC {}: {}", socket_path.display(), e))?;

        // Asegurar permisos 0600 (propietario exclusivo)
        let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600));

        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Error configurando nonblocking en socket: {}", e))?;

        log::info!("Servidor IPC iniciado en {}", socket_path.display());

        Ok(Self {
            socket_path,
            listener,
            state,
            cmd_sender,
            shutdown,
        })
    }

    /// Retorna la ruta al socket Unix.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Bucle principal de atención del servidor IPC.
    pub fn run(&self) {
        let mut clients: Vec<ClientConnection> = Vec::with_capacity(MAX_CLIENTS);
        let mut read_buf = [0u8; 4096];

        while !self.shutdown.load(Ordering::Relaxed) {
            // 1. Aceptar nuevas conexiones
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    // Validar SO_PEERCRED
                    if let Ok(peer_uid) = get_peer_uid(&stream) {
                        let expected_uid = unsafe { libc::getuid() };
                        if peer_uid != expected_uid {
                            log::warn!(
                                "Conexión IPC rechazada: UID {} no coincide con UID esperado {}",
                                peer_uid,
                                expected_uid
                            );
                            continue;
                        }
                    } else {
                        log::warn!("No se pudieron leer credenciales SO_PEERCRED del cliente IPC.");
                        continue;
                    }

                    if clients.len() >= MAX_CLIENTS {
                        log::warn!("Límite de conexiones IPC alcanzado ({}). Rechazando cliente.", MAX_CLIENTS);
                        let mut str = stream;
                        let err_resp = format_response(&IpcResponse::error(None, "too many clients"));
                        let _ = str.write_all(err_resp.as_bytes());
                        continue;
                    }

                    if let Err(e) = stream.set_nonblocking(true) {
                        log::warn!("Error configurando nonblocking en cliente: {}", e);
                        continue;
                    }

                    clients.push(ClientConnection {
                        stream,
                        buffer: Vec::new(),
                        last_activity: Instant::now(),
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No hay clientes nuevos pendientes
                }
                Err(e) => {
                    log::debug!("Error aceptando conexión IPC: {}", e);
                }
            }

            // 2. Procesar datos de clientes conectados
            let now = Instant::now();
            let mut disconnected_indices = Vec::new();

            for (i, client) in clients.iter_mut().enumerate() {
                // Timeout anti-slow-loris para líneas incompletas y desconexión de clientes ociosos
                let elapsed_inactivity = now.duration_since(client.last_activity);
                let is_timeout = if client.buffer.is_empty() {
                    elapsed_inactivity > IDLE_CONNECTION_TIMEOUT
                } else {
                    elapsed_inactivity > INCOMPLETE_LINE_TIMEOUT
                };

                if is_timeout {
                    log::warn!("Timeout de inactividad en cliente IPC. Cerrando conexión.");
                    disconnected_indices.push(i);
                    continue;
                }

                match client.stream.read(&mut read_buf) {
                    Ok(0) => {
                        // EOF del cliente
                        disconnected_indices.push(i);
                    }
                    Ok(n) => {
                        client.last_activity = now;
                        client.buffer.extend_from_slice(&read_buf[..n]);

                        if client.buffer.len() > MAX_PAYLOAD_BYTES {
                            log::warn!("Payload excedió el límite de 64 KB. Rechazando.");
                            let err_resp = format_response(&IpcResponse::error(None, "payload too large"));
                            let _ = client.stream.write_all(err_resp.as_bytes());
                            let _ = client.stream.flush();
                            let _ = client.stream.shutdown(std::net::Shutdown::Both);
                            disconnected_indices.push(i);
                            continue;
                        }

                        // Procesar mensajes delimitados por \n
                        while let Some(pos) = client.buffer.iter().position(|&b| b == b'\n') {
                            let line_bytes = client.buffer.drain(..=pos).collect::<Vec<u8>>();
                            let line_str = match std::str::from_utf8(&line_bytes) {
                                Ok(s) => s.trim(),
                                Err(_) => {
                                    let err = format_response(&IpcResponse::error(None, "invalid utf-8"));
                                    let _ = client.stream.write_all(err.as_bytes());
                                    continue;
                                }
                            };

                            if line_str.is_empty() {
                                continue;
                            }

                            let response = self.handle_line(line_str);
                            let resp_str = format_response(&response);
                            if let Err(e) = client.stream.write_all(resp_str.as_bytes()) {
                                log::debug!("Error escribiendo respuesta IPC: {}", e);
                                disconnected_indices.push(i);
                                break;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Sin datos en este ciclo
                    }
                    Err(e) => {
                        log::debug!("Error leyendo de socket IPC: {}", e);
                        disconnected_indices.push(i);
                    }
                }
            }

            // Remover clientes desconectados (en orden inverso)
            for i in disconnected_indices.into_iter().rev() {
                if i < clients.len() {
                    clients.swap_remove(i);
                }
            }

            thread::sleep(Duration::from_millis(20));
        }
    }

    /// Despacha una línea JSON entrante y genera la respuesta adecuada.
    fn handle_line(&self, line: &str) -> IpcResponse {
        let request = match parse_request(line) {
            Ok(req) => req,
            Err(e) => return IpcResponse::error(None, e),
        };

        let req_id = request.id;

        match request.command {
            IpcCommand::Ping => IpcResponse::ok(req_id, "pong"),
            IpcCommand::GetStatus => {
                let status = self.state.read().unwrap_or_else(|e| e.into_inner()).to_status_data();
                IpcResponse::ok(req_id, status)
            }
            IpcCommand::GetHealth => {
                let health = self.state.read().unwrap_or_else(|e| e.into_inner()).to_health();
                IpcResponse::ok(req_id, health)
            }
            IpcCommand::GetBattery => {
                let need_refresh = {
                    let s = self.state.read().unwrap_or_else(|e| e.into_inner());
                    match (s.battery_level, s.battery_last_updated) {
                        (Some(_), Some(updated)) => updated.elapsed() > BATTERY_CACHE_TTL,
                        _ => true,
                    }
                };

                if need_refresh {
                    let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
                    if self.cmd_sender.send(DaemonCommand::RefreshBattery { reply: reply_tx }).is_ok() {
                        if let Ok(Ok((pct, status))) = reply_rx.recv_timeout(HW_REPLY_TIMEOUT) {
                            return IpcResponse::ok(req_id, BatteryData {
                                percentage: pct,
                                status,
                            });
                        }
                    }
                }

                let state_guard = self.state.read().unwrap_or_else(|e| e.into_inner());
                if let Some(bat) = state_guard.to_battery_data() {
                    IpcResponse::ok(req_id, bat)
                } else {
                    IpcResponse::error(req_id, "Battery information not available")
                }
            }
            IpcCommand::SetAction { cid, action } => {
                if !is_valid_cid(cid) {
                    return IpcResponse::error(req_id, format!("CID 0x{:04X} no es reprogramable", cid));
                }

                let is_default = matches!(action, ActionPayload::SystemDefault);
                let divert = !is_default;

                let act = match action.to_action() {
                    Ok(a) => a,
                    Err(e) => return IpcResponse::error(req_id, e),
                };

                // Notificar al hilo actor de hardware ANTES de mutar el estado en memoria
                let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
                let cmd = DaemonCommand::SetAction {
                    cid,
                    action: act,
                    divert,
                    reply: reply_tx,
                };

                if self.cmd_sender.send(cmd).is_ok() {
                    match reply_rx.recv_timeout(HW_REPLY_TIMEOUT) {
                        Ok(Ok(())) => {
                            // Actualizar mapeo en memoria exclusivamente tras éxito verificado en hardware
                            let mut s = self.state.write().unwrap_or_else(|e| e.into_inner());
                            if is_default {
                                s.mapping.remove(&cid);
                                s.diverted_cids.retain(|&c| c != cid);
                            } else {
                                s.mapping.insert(cid, action);
                            }
                            IpcResponse::ok_empty(req_id)
                        }
                        Ok(Err(e)) => IpcResponse::error(req_id, e),
                        Err(_) => IpcResponse::error(req_id, "Hardware timeout aplicando acción"),
                    }
                } else {
                    IpcResponse::error(req_id, "Worker de hardware no disponible")
                }
            }
            IpcCommand::ReloadProfile => {
                log::info!("Petición reload_profile recibida vía IPC.");
                // Honestidad operativa (Regla de Oro): los perfiles persistentes se integran en Fase 4
                IpcResponse::error(req_id, "reload_profile is not implemented yet (profile storage planned for Phase 4)")
            }
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        log::info!("Limpiando socket IPC en {}...", self.socket_path.display());
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
