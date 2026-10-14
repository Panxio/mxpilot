use log::{debug, warn};
use mx_daemon::ipc::protocol::{
    ActionPayload, BatteryData, DaemonHealth, IpcCommand, IpcRequest, IpcResponse, StatusData,
    VALID_CIDS,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Tamaño máximo acotado de respuesta en bytes (64 KB).
pub const MAX_IPC_RESPONSE_BYTES: usize = 65_536;

/// Timeout máximo para establecer conexión con el socket IPC (500 ms).
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Timeout de lectura y escritura para comandos rápidos (1000 ms).
pub const FAST_READ_TIMEOUT: Duration = Duration::from_millis(1000);
pub const FAST_WRITE_TIMEOUT: Duration = Duration::from_millis(500);

/// Timeout de lectura y escritura para comandos con interacción de hardware (2500 ms).
pub const SLOW_READ_TIMEOUT: Duration = Duration::from_millis(2500);
pub const SLOW_WRITE_TIMEOUT: Duration = Duration::from_millis(1000);

/// Contador atómico de IDs de petición para correlación estricta.
static NEXT_REQ_ID: AtomicU64 = AtomicU64::new(1);

/// Errores tipados del cliente IPC para consumo honesto en Tauri y frontend.
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "message")]
pub enum ClientError {
    #[error("mx-daemon no está en ejecución ({0})")]
    DaemonNotRunning(String),

    #[error("Permiso denegado al acceder al socket ({0})")]
    PermissionDenied(String),

    #[error("Timeout comunicándose con mx-daemon ({0})")]
    Timeout(String),

    #[error("La respuesta del daemon excedió la cota de 64 KB sin terminador")]
    PayloadTooLarge,

    #[error("Error de protocolo IPC: {0}")]
    ProtocolError(String),

    #[error("mx-daemon reportó error: {0}")]
    DaemonError(String),

    #[error("Error de E/S: {0}")]
    IoError(String),
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        map_io_error(err)
    }
}

/// Mapea un `std::io::Error` a una variante tipada de `ClientError`.
pub fn map_io_error(err: std::io::Error) -> ClientError {
    match err.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => {
            ClientError::DaemonNotRunning(err.to_string())
        }
        std::io::ErrorKind::PermissionDenied => ClientError::PermissionDenied(err.to_string()),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            ClientError::Timeout(err.to_string())
        }
        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe => {
            ClientError::ProtocolError(format!(
                "Conexión cerrada o reiniciada prematuramente por el daemon (EOF / Reset): {}",
                err
            ))
        }
        _ => ClientError::IoError(err.to_string()),
    }
}

/// Cliente IPC para comunicarse con `mx-daemon` sobre Unix Domain Socket.
#[derive(Debug, Clone)]
pub struct DaemonClient {
    socket_path: PathBuf,
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonClient {
    /// Inicializa el cliente resolviendo la ruta del socket desde el entorno.
    pub fn new() -> Self {
        Self {
            socket_path: resolve_default_socket(),
        }
    }

    /// Inicializa el cliente con una ruta personalizada (usado principalmente en tests).
    pub fn with_socket_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            socket_path: path.into(),
        }
    }

    /// Retorna la ruta del socket configurada.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Conecta con el socket aplicando un deadline estricto sin bloqueo indefinido.
    fn connect_with_deadline(&self, timeout: Duration) -> Result<UnixStream, ClientError> {
        if !self.socket_path.exists() {
            return Err(ClientError::DaemonNotRunning(format!(
                "Socket no encontrado en {}",
                self.socket_path.display()
            )));
        }

        unsafe {
            let fd: RawFd = libc::socket(
                libc::AF_UNIX,
                libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
                0,
            );
            if fd < 0 {
                return Err(ClientError::from(std::io::Error::last_os_error()));
            }

            let mut addr: libc::sockaddr_un = std::mem::zeroed();
            addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

            let path_bytes = self.socket_path.as_os_str().as_bytes();
            if path_bytes.len() >= addr.sun_path.len() {
                libc::close(fd);
                return Err(ClientError::ProtocolError(
                    "Ruta de socket excede el límite sockaddr_un".into(),
                ));
            }

            for (dest, &src) in addr.sun_path.iter_mut().zip(path_bytes.iter()) {
                *dest = src as libc::c_char;
            }

            let path_len = path_bytes.len();
            let addr_len = (std::mem::size_of::<libc::sa_family_t>() + path_len + 1) as libc::socklen_t;

            let res = libc::connect(
                fd,
                &addr as *const _ as *const libc::sockaddr,
                addr_len,
            );

            if res < 0 {
                let err = std::io::Error::last_os_error();
                let raw_code = err.raw_os_error();
                if raw_code == Some(libc::EINPROGRESS) || raw_code == Some(libc::EALREADY) {
                    let mut pfd = libc::pollfd {
                        fd,
                        events: libc::POLLOUT,
                        revents: 0,
                    };
                    let poll_res = libc::poll(&mut pfd, 1, timeout.as_millis() as i32);
                    if poll_res == 0 {
                        libc::close(fd);
                        return Err(ClientError::Timeout(format!(
                            "Timeout de conexión de {} ms superado",
                            timeout.as_millis()
                        )));
                    } else if poll_res < 0 {
                        let poll_err = std::io::Error::last_os_error();
                        libc::close(fd);
                        return Err(ClientError::from(poll_err));
                    }

                    // Verificar si la conexión tuvo éxito mediante getsockopt SO_ERROR
                    let mut sock_err: libc::c_int = 0;
                    let mut opt_len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
                    libc::getsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_ERROR,
                        &mut sock_err as *mut _ as *mut libc::c_void,
                        &mut opt_len,
                    );
                    if sock_err != 0 {
                        libc::close(fd);
                        return Err(map_io_error(std::io::Error::from_raw_os_error(sock_err)));
                    }
                } else {
                    libc::close(fd);
                    return Err(map_io_error(err));
                }
            }

            // Restaurar a modo bloqueante para lectura/escritura con timeouts SO_RCVTIMEO/SO_SNDTIMEO
            let flags = libc::fcntl(fd, libc::F_GETFL, 0);
            if flags >= 0 {
                libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
            }

            let stream = UnixStream::from_raw_fd(fd);
            Ok(stream)
        }
    }

    /// Envía un comando IPC tipado y procesa la respuesta validando framing, ID y cota de 64 KB.
    pub fn send_command(&self, cmd: IpcCommand) -> Result<Option<serde_json::Value>, ClientError> {
        let is_slow = matches!(
            cmd,
            IpcCommand::SetAction { .. } | IpcCommand::GetBattery | IpcCommand::ReloadProfile
        );
        let read_timeout = if is_slow {
            SLOW_READ_TIMEOUT
        } else {
            FAST_READ_TIMEOUT
        };
        let write_timeout = if is_slow {
            SLOW_WRITE_TIMEOUT
        } else {
            FAST_WRITE_TIMEOUT
        };

        let stream = self.connect_with_deadline(CONNECT_TIMEOUT)?;
        stream.set_read_timeout(Some(read_timeout))?;
        stream.set_write_timeout(Some(write_timeout))?;

        let req_id = NEXT_REQ_ID.fetch_add(1, Ordering::SeqCst);
        let request = IpcRequest {
            id: Some(req_id),
            command: cmd,
        };

        let mut req_bytes = serde_json::to_vec(&request)
            .map_err(|e| ClientError::ProtocolError(format!("Error serializando petición: {}", e)))?;
        req_bytes.push(b'\n');

        // Escritura atómica
        let mut writer = &stream;
        writer.write_all(&req_bytes)?;
        writer.flush()?;

        // Lectura con cota estricta de 64 KB + 1 byte para detectar desbordes
        let mut take_reader = BufReader::new(&stream).take(MAX_IPC_RESPONSE_BYTES as u64 + 1);
        let mut line = String::new();
        let bytes_read = take_reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Err(ClientError::ProtocolError(
                "El daemon cerró la conexión sin emitir respuesta (EOF)".into(),
            ));
        }

        if bytes_read > MAX_IPC_RESPONSE_BYTES {
            return Err(ClientError::PayloadTooLarge);
        }

        if !line.ends_with('\n') {
            return Err(ClientError::ProtocolError(
                "Respuesta truncada sin terminador de línea".into(),
            ));
        }

        let resp: IpcResponse = serde_json::from_str(&line)
            .map_err(|e| ClientError::ProtocolError(format!("Error deserializando JSON: {}", e)))?;

        match resp {
            IpcResponse::Ok { id, data } => {
                if let Some(resp_id) = id {
                    if resp_id != req_id {
                        return Err(ClientError::ProtocolError(format!(
                            "ID de correlación no coincide: enviado {} vs recibido {}",
                            req_id, resp_id
                        )));
                    }
                }
                Ok(data)
            }
            IpcResponse::Error { id, message } => {
                if let Some(resp_id) = id {
                    if resp_id != req_id {
                        warn!(
                            "ID de correlación erróneo en mensaje de error: enviado {} vs recibido {}",
                            req_id, resp_id
                        );
                    }
                }
                Err(ClientError::DaemonError(message))
            }
        }
    }

    /// Verifica liveness emitiendo `ping`.
    pub fn ping(&self) -> Result<String, ClientError> {
        let val = self.send_command(IpcCommand::Ping)?;
        match val {
            Some(serde_json::Value::String(s)) => Ok(s),
            _ => Ok("pong".to_string()),
        }
    }

    /// Obtiene el estado funcional actual del daemon y hardware.
    pub fn get_status(&self) -> Result<StatusData, ClientError> {
        let val = self.send_command(IpcCommand::GetStatus)?;
        match val {
            Some(v) => serde_json::from_value(v)
                .map_err(|e| ClientError::ProtocolError(format!("StatusData inválido: {}", e))),
            None => Err(ClientError::ProtocolError("StatusData vacío (null)".into())),
        }
    }

    /// Obtiene las métricas completas de salud y telemetría del daemon.
    pub fn get_health(&self) -> Result<DaemonHealth, ClientError> {
        let val = self.send_command(IpcCommand::GetHealth)?;
        match val {
            Some(v) => serde_json::from_value(v)
                .map_err(|e| ClientError::ProtocolError(format!("DaemonHealth inválido: {}", e))),
            None => Err(ClientError::ProtocolError("DaemonHealth vacío (null)".into())),
        }
    }

    /// Obtiene la telemetría de batería fresca o en caché.
    pub fn get_battery(&self) -> Result<BatteryData, ClientError> {
        let val = self.send_command(IpcCommand::GetBattery)?;
        match val {
            Some(v) => serde_json::from_value(v)
                .map_err(|e| ClientError::ProtocolError(format!("BatteryData inválido: {}", e))),
            None => Err(ClientError::ProtocolError("BatteryData vacío (null)".into())),
        }
    }

    /// Asigna una acción a un CID previa validación de allowlist.
    pub fn set_action(&self, cid: u16, action: ActionPayload) -> Result<(), ClientError> {
        if !VALID_CIDS.contains(&cid) {
            return Err(ClientError::ProtocolError(format!(
                "CID {:#06X} no es reprogramable",
                cid
            )));
        }
        // Validar que la acción sea convertible antes de tocar el socket
        action.to_action().map_err(ClientError::ProtocolError)?;

        self.send_command(IpcCommand::SetAction { cid, action })?;
        Ok(())
    }
}

/// Resuelve la ruta estándar de `mx-options.sock` según especificación F3 y F4.
pub fn resolve_default_socket() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime_dir.trim().is_empty() {
            return PathBuf::from(runtime_dir).join("mx-options.sock");
        }
    }

    // Fallback estándar de usuario en Linux: /run/user/<UID>/mx-options.sock
    let uid = unsafe { libc::getuid() };
    let fallback = PathBuf::from(format!("/run/user/{}/mx-options.sock", uid));
    debug!("XDG_RUNTIME_DIR no definido; usando fallback {}", fallback.display());
    fallback
}
