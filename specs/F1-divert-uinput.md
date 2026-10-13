# Especificación Técnica (SDD) — F1: Button Diversion y Emulación uinput (mx-cli)

## 1. Propósito y Alcance
Esta especificación define la arquitectura y el comportamiento exacto para:
1. Activar y desactivar el desvío de botones (*Button Diversion*) mediante la característica HID++ `0x1B04` (`REPROG_CONTROLS_V4`) sobre el ratón Logitech MX Vertical (`046d:b020`), enfocándose en el botón superior de DPI (CID `0x00FD`) y botones laterales.
2. Escuchar y procesar las tramas de notificación asíncronas `divertedButtonsEvent` emitidas por el mouse cuando un botón desviado cambia de estado.
3. Emitir pulsaciones de teclado y clics de ratón a nivel de kernel utilizando `/dev/uinput` a través del nuevo crate `mx-uinput`.
4. Garantizar la liberación limpia de recursos (RAII): al detenerse la aplicación, el desvío se desactiva y el nodo virtual `/dev/uinput` se destruye inmediatamente sin dejar botones muertos ni dispositivos fantasmas.

---

## 2. Protocolo de Diversion (Feature 0x1B04)

### 2.1 Activación y Desactivación (`setCidReporting`)
Para cambiar el estado de desvío de un Control ID (`cid`), el host envía un reporte largo (`0x11`) a la función `0x03` (`setCidReporting`) de la feature `0x1B04`:
- **Report ID:** `0x11` (Long Report — 20 bytes)
- **Device Index:** `0xFF`
- **Feature Index:** Índice dinámico de `0x1B04` (en el MX Vertical es `0x0A`)
- **Function ID & SW ID:** `0x31` (Function `0x03`, Software ID cíclico `0x01`..`0x0F`)
- **Parámetros (Bytes 4..8):**
  - Bytes 0-1: `CID` en formato Big Endian (ej. `0x00FD` para el botón superior).
  - Byte 2: `bfield`:
    - Activar Divert: `0x03` (`DIVERTED = 0x01` | `DIVERTED_VALID = 0x02`)
    - Desactivar Divert: `0x02` (`DIVERTED = 0x00` | `DIVERTED_VALID = 0x02`)
  - Bytes 3-4: `remap` en formato Big Endian (`0x0000` para mantener la función nativa sin remapeo de firmware).
  - Bytes 5-15: Relleno con ceros (`0x00`).

### 2.2 Evento Asíncrono de Pulsación (`divertedButtonsEvent`)
Cuando un botón desviado es presionado o soltado, el firmware emite un reporte de notificación:
- **Report ID:** `0x11`
- **Device Index:** `0xFF`
- **Feature Index:** Índice de `0x1B04` (`0x0A`)
- **Function ID / Event:** `0x00` (Evento 0)
- **Parámetros:** Lista ordenada de hasta 4 CIDs actualmente presionados:
  - `params[0..2]`: CID 1 (MSB, LSB)
  - `params[2..4]`: CID 2 (MSB, LSB)
  - `params[4..6]`: CID 3 (MSB, LSB)
  - `params[6..8]`: CID 4 (MSB, LSB)
- **Regla de Estado:**
  - Si el CID `0x00FD` está presente en la lista $\to$ **Pulsado (Key Down)**.
  - Si el CID `0x00FD` desaparece de la lista (o la lista está vacía con `0x0000`) $\to$ **Liberado (Key Up)**.

---

## 3. Emulador de Entrada Virtual (`mx-uinput`)

### 3.1 Dispositivo Virtual
- **Nombre:** `MX Options Virtual Device`
- **Bus:** `BUS_USB`
- **Vendor:** `0x046D` (Logitech)
- **Product:** `0xB020` (MX Vertical)
- **Version:** `1`
- **Capacidades Habilitadas:**
  - Teclas de teclado (`EV_KEY`): teclas modificadoras (`KEY_LEFTMETA`, `KEY_LEFTCTRL`, `KEY_LEFTALT`, `KEY_LEFTSHIFT`), letras y números (`KEY_C`, `KEY_V`, `KEY_TAB`, etc.), y teclas multimedia (`KEY_PLAYPAUSE`, `KEY_MUTE`, `KEY_VOLUMEUP`, `KEY_VOLUMEDOWN`).
  - Botones de ratón (`BTN_LEFT`, `BTN_RIGHT`, `BTN_MIDDLE`, `BTN_SIDE`, `BTN_EXTRA`).

### 3.2 Acciones Soportadas en MVP
```rust
pub enum VirtualAction {
    /// Pulsación de una sola tecla (ej. Super / Meta para abrir launcher)
    SingleKey(evdev::Key),
    /// Combinación con modificadores (ej. Ctrl + C, Alt + Tab)
    KeyCombo { modifiers: Vec<evdev::Key>, key: evdev::Key },
    /// Clic de botón de ratón (ej. Clic central)
    MouseButton(evdev::Key),
}
```

---

## 4. Invariantes del Sistema (Pilar 2)
1. **Garantía RAII (Zero Dead Buttons):** El guardián de divert implementa el trait `Drop`. Al salir del proceso por cualquier motivo (incluyendo `SIGINT` / `Ctrl+C`), se envía automáticamente `bfield = 0x02` para desactivar el divert y devolver el control al microcontrolador.
2. **Debounce y Deduplicación:** El bucle de eventos mantiene el estado anterior (`is_pressed: bool`). Solo se emite un evento de pulsación (`Key Down` + `SYN_REPORT`) al pasar de falso a verdadero, y un evento de liberación (`Key Up` + `SYN_REPORT`) al pasar de verdadero a falso.
3. **Aislamiento en Tests:** `VirtualInput` puede probarse de forma simulada en tests unitarios sin requerir permisos de kernel directos.

---

## 5. Árbitros Objetivos (Criterios de Cierre de F1)
1. **Arnés TDD:** `cargo test --workspace` pasa al 100% en menos de 1 segundo.
2. **Clippy:** `cargo clippy --workspace -- -D warnings` pasa con 0 advertencias.
3. **Árbitro en Hardware (`evtest`):** Al presionar físicamente el botón superior del MX Vertical, `evtest` sobre el nodo creado por `mx-uinput` registra exactamente 1 evento `DOWN` al presionar y 1 evento `UP` al soltar, con $\ge 99\%$ de confiabilidad en Wayland y X11.
