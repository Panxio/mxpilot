use std::fs::{File, OpenOptions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

/// Nombre por defecto del archivo de bloqueo singleton.
pub const LOCK_FILE_NAME: &str = "mx-options.lock";

/// Estructura guardiana que retiene el bloqueo exclusivo de archivo (`flock`).
pub struct SingletonLock {
    _file: File,
    path: PathBuf,
}

impl SingletonLock {
    /// Intenta adquirir el bloqueo exclusivo sobre `$XDG_RUNTIME_DIR/mx-options.lock`.
    pub fn acquire() -> Result<Self, String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map_err(|_| "Variable de entorno XDG_RUNTIME_DIR no definida. mx-daemon requiere una sesión de usuario activa.".to_string())?;
        Self::acquire_in_dir(Path::new(&runtime_dir))
    }

    /// Intenta adquirir el bloqueo exclusivo en un directorio específico.
    pub fn acquire_in_dir(dir: &Path) -> Result<Self, String> {
        let lock_path = dir.join(LOCK_FILE_NAME);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| format!("Error abriendo lock file {}: {}", lock_path.display(), e))?;

        // Asegurar permisos 0600
        let _ = file.set_permissions(std::fs::Permissions::from_mode(0o600));

        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

        if ret != 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EWOULDBLOCK)
                || err.raw_os_error() == Some(libc::EAGAIN)
            {
                return Err(format!(
                    "Ya existe otra instancia activa de mx-daemon en ejecución (lock {} bloqueado).",
                    lock_path.display()
                ));
            } else {
                return Err(format!(
                    "Fallo al adquirir flock sobre {}: {}",
                    lock_path.display(),
                    err
                ));
            }
        }

        log::info!("SingletonLock adquirido exitosamente en {}", lock_path.display());
        Ok(Self {
            _file: file,
            path: lock_path,
        })
    }

    /// Retorna la ruta al archivo de lock.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_acquire_and_contention() {
        // Usar directorio temporal para no colisionar con runtime real
        let temp_dir = std::env::temp_dir().join(format!("mx_lock_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        std::env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

        let lock1 = SingletonLock::acquire_in_dir(&temp_dir);
        assert!(lock1.is_ok(), "Primer lock debe adquirirse");

        let lock2 = SingletonLock::acquire_in_dir(&temp_dir);
        assert!(
            lock2.is_err(),
            "Segundo intento concurrente debe retornar Err"
        );

        drop(lock1);

        // Tras liberar, debe poder adquirirse nuevamente
        let lock3 = SingletonLock::acquire_in_dir(&temp_dir);
        assert!(lock3.is_ok(), "Debe re-adquirirse tras liberar lock previo");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
