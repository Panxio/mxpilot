use evdev::KeyCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Presiona y suelta una tecla individual (ej. Meta / Super)
    SingleKey(KeyCode),
    /// Combinación de modificadores + tecla (ej. Ctrl + C, Alt + Tab)
    KeyCombo {
        modifiers: Vec<KeyCode>,
        key: KeyCode,
    },
    /// Clic de botón de ratón (ej. Clic central, clic lateral)
    MouseButton(KeyCode),
    /// Sin acción asignada
    None,
}

impl Action {
    /// Crea una acción de tecla Super (Meta)
    pub fn super_key() -> Self {
        Self::SingleKey(KeyCode::KEY_LEFTMETA)
    }

    /// Crea una acción Ctrl + C
    pub fn copy() -> Self {
        Self::KeyCombo {
            modifiers: vec![KeyCode::KEY_LEFTCTRL],
            key: KeyCode::KEY_C,
        }
    }

    /// Crea una acción Ctrl + V
    pub fn paste() -> Self {
        Self::KeyCombo {
            modifiers: vec![KeyCode::KEY_LEFTCTRL],
            key: KeyCode::KEY_V,
        }
    }

    /// Crea una acción Alt + Tab
    pub fn alt_tab() -> Self {
        Self::KeyCombo {
            modifiers: vec![KeyCode::KEY_LEFTALT],
            key: KeyCode::KEY_TAB,
        }
    }
}
