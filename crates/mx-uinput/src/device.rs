use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use std::thread;
use std::time::Duration;

use crate::action::Action;
use crate::error::UinputError;

pub struct VirtualInput {
    device: VirtualDevice,
}

impl VirtualInput {
    pub fn new() -> Result<Self, UinputError> {
        let mut keys = AttributeSet::<KeyCode>::new();

        // Modificadores
        keys.insert(KeyCode::KEY_LEFTMETA);
        keys.insert(KeyCode::KEY_RIGHTMETA);
        keys.insert(KeyCode::KEY_LEFTCTRL);
        keys.insert(KeyCode::KEY_RIGHTCTRL);
        keys.insert(KeyCode::KEY_LEFTALT);
        keys.insert(KeyCode::KEY_RIGHTALT);
        keys.insert(KeyCode::KEY_LEFTSHIFT);
        keys.insert(KeyCode::KEY_RIGHTSHIFT);

        // Control y navegación
        keys.insert(KeyCode::KEY_TAB);
        keys.insert(KeyCode::KEY_ENTER);
        keys.insert(KeyCode::KEY_ESC);
        keys.insert(KeyCode::KEY_SPACE);
        keys.insert(KeyCode::KEY_BACKSPACE);
        keys.insert(KeyCode::KEY_DELETE);
        keys.insert(KeyCode::KEY_RIGHT);
        keys.insert(KeyCode::KEY_LEFT);
        keys.insert(KeyCode::KEY_UP);
        keys.insert(KeyCode::KEY_DOWN);
        keys.insert(KeyCode::KEY_PAGEUP);
        keys.insert(KeyCode::KEY_PAGEDOWN);

        // Teclas alfanuméricas comunes
        keys.insert(KeyCode::KEY_C);
        keys.insert(KeyCode::KEY_V);
        keys.insert(KeyCode::KEY_X);
        keys.insert(KeyCode::KEY_Z);
        keys.insert(KeyCode::KEY_A);
        keys.insert(KeyCode::KEY_Q);
        keys.insert(KeyCode::KEY_W);

        // Teclas multimedia
        keys.insert(KeyCode::KEY_PLAYPAUSE);
        keys.insert(KeyCode::KEY_MUTE);
        keys.insert(KeyCode::KEY_VOLUMEUP);
        keys.insert(KeyCode::KEY_VOLUMEDOWN);
        keys.insert(KeyCode::KEY_PREVIOUSSONG);
        keys.insert(KeyCode::KEY_NEXTSONG);

        // Botones de ratón
        keys.insert(KeyCode::BTN_LEFT);
        keys.insert(KeyCode::BTN_RIGHT);
        keys.insert(KeyCode::BTN_MIDDLE);
        keys.insert(KeyCode::BTN_SIDE);
        keys.insert(KeyCode::BTN_EXTRA);

        let device = VirtualDevice::builder()
            .map_err(|e| UinputError::CreateDevice(e.to_string()))?
            .name("MX Options Virtual Device")
            .with_keys(&keys)
            .map_err(|e| UinputError::CreateDevice(e.to_string()))?
            .build()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    UinputError::PermissionDenied
                } else {
                    UinputError::CreateDevice(e.to_string())
                }
            })?;

        Ok(Self { device })
    }

    /// Emite una pulsación hacia abajo (press)
    pub fn emit_press(&mut self, key: KeyCode) -> Result<(), UinputError> {
        let ev = InputEvent::new(EventType::KEY.0, key.code(), 1);
        self.device
            .emit(&[ev])
            .map_err(|e| UinputError::EmitEvent(e.to_string()))
    }

    /// Emite una liberación hacia arriba (release)
    pub fn emit_release(&mut self, key: KeyCode) -> Result<(), UinputError> {
        let ev = InputEvent::new(EventType::KEY.0, key.code(), 0);
        self.device
            .emit(&[ev])
            .map_err(|e| UinputError::EmitEvent(e.to_string()))
    }

    /// Emite un clic completo (press -> pausa -> release)
    pub fn emit_click(&mut self, key: KeyCode) -> Result<(), UinputError> {
        self.emit_press(key)?;
        thread::sleep(Duration::from_millis(10));
        self.emit_release(key)?;
        Ok(())
    }

    /// Emite una combinación con modificadores (ej. Ctrl + C)
    pub fn emit_combo(&mut self, modifiers: &[KeyCode], key: KeyCode) -> Result<(), UinputError> {
        for m in modifiers {
            self.emit_press(*m)?;
        }
        thread::sleep(Duration::from_millis(5));
        self.emit_click(key)?;
        thread::sleep(Duration::from_millis(5));
        for m in modifiers.iter().rev() {
            self.emit_release(*m)?;
        }
        Ok(())
    }

    /// Emite la pulsación (Key Down) de una acción abstracta sin sleeps bloqueantes
    pub fn emit_action_down(&mut self, action: &Action) -> Result<(), UinputError> {
        match action {
            Action::SingleKey(k) => self.emit_press(*k),
            Action::KeyCombo { modifiers, key } => {
                for m in modifiers {
                    self.emit_press(*m)?;
                }
                self.emit_press(*key)
            }
            Action::MouseButton(b) => self.emit_press(*b),
            Action::None => Ok(()),
        }
    }

    /// Emite la liberación (Key Up) de una acción abstracta en orden inverso sin sleeps bloqueantes
    pub fn emit_action_up(&mut self, action: &Action) -> Result<(), UinputError> {
        match action {
            Action::SingleKey(k) => self.emit_release(*k),
            Action::KeyCombo { modifiers, key } => {
                self.emit_release(*key)?;
                for m in modifiers.iter().rev() {
                    self.emit_release(*m)?;
                }
                Ok(())
            }
            Action::MouseButton(b) => self.emit_release(*b),
            Action::None => Ok(()),
        }
    }

    /// Ejecuta una acción abstracta completa (Down + Up)
    pub fn emit_action(&mut self, action: &Action) -> Result<(), UinputError> {
        match action {
            Action::SingleKey(k) => self.emit_click(*k),
            Action::KeyCombo { modifiers, key } => self.emit_combo(modifiers, *key),
            Action::MouseButton(b) => self.emit_click(*b),
            Action::None => Ok(()),
        }
    }
}
