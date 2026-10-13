use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::error::HidppError;
use crate::frame::HidppFrame;
use crate::transport::Transport;

pub struct HidrawTransport {
    file: File,
}

impl HidrawTransport {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HidppError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => HidppError::Io(format!(
                    "Permiso denegado al abrir {}. Requiere permisos udev o root.",
                    path.as_ref().display()
                )),
                std::io::ErrorKind::NotFound => HidppError::DeviceNotFound,
                _ => HidppError::Io(e.to_string()),
            })?;

        Ok(Self { file })
    }
}

impl Transport for HidrawTransport {
    fn send(&mut self, frame: &HidppFrame) -> Result<(), HidppError> {
        let bytes = frame.to_bytes();
        self.file
            .write_all(&bytes)
            .map_err(|e| HidppError::Io(format!("Error enviando trama HID++: {}", e)))?;
        Ok(())
    }

    fn receive(&mut self, timeout: Duration) -> Result<HidppFrame, HidppError> {
        let mut pfd = libc::pollfd {
            fd: self.file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };

        let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
        let ret = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) {
                return Err(HidppError::Timeout);
            }
            return Err(HidppError::Io(err.to_string()));
        }
        if ret == 0 {
            return Err(HidppError::Timeout);
        }

        // Si el kernel reporta error, cuelgue o nodo inválido, el hardware se desconectó
        if (pfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL)) != 0 {
            return Err(HidppError::Disconnected);
        }

        let mut buf = [0u8; 64];
        let n = match self.file.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                if let Some(code) = e.raw_os_error() {
                    if code == libc::ENODEV || code == libc::EIO || code == libc::ESHUTDOWN {
                        return Err(HidppError::Disconnected);
                    }
                }
                return Err(HidppError::Io(format!("Error leyendo /dev/hidraw: {}", e)));
            }
        };

        if n == 0 {
            return Err(HidppError::Disconnected);
        }

        HidppFrame::from_bytes(&buf[..n])
    }

    fn request(&mut self, frame: &HidppFrame, timeout: Duration) -> Result<HidppFrame, HidppError> {
        self.send(frame)?;
        let start = Instant::now();

        // Bucle de lectura para descartar reportes no solicitados (movimiento, notificaciones de fondo)
        while start.elapsed() < timeout {
            let remaining = timeout.saturating_sub(start.elapsed());
            match self.receive(remaining) {
                Ok(resp) => {
                    // Si coincide con la feature solicitada y el software ID
                    if resp.feature_index == frame.feature_index
                        && resp.software_id == frame.software_id
                    {
                        return Ok(resp);
                    }
                    // O si es la feature 0x00 (Root) respondiendo a consultas iniciales
                    if frame.feature_index == 0x00 && resp.feature_index == 0x00 {
                        return Ok(resp);
                    }
                }
                Err(HidppError::UnknownReportId(_)) => {
                    // Ignorar reportes HID estándar del mouse (ej. 0x02 movimiento)
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(HidppError::Timeout)
    }
}
