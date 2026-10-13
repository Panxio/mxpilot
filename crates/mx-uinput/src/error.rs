use thiserror::Error;

#[derive(Error, Debug)]
pub enum UinputError {
    #[error("Error al crear dispositivo virtual uinput: {0}")]
    CreateDevice(String),

    #[error("Error al emitir evento uinput: {0}")]
    EmitEvent(String),

    #[error("Permiso denegado al acceder a /dev/uinput. Verifique las reglas udev.")]
    PermissionDenied,
}
