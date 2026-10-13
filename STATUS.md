# STATUS.md — MX Options Linux

> Bitácora de continuidad entre sesiones. El agente NO recuerda entre sesiones:
> LÉEME PRIMERO al retomar, y ACTUALÍZAME al terminar.
> Perfil del proyecto: tauri-app (Rust Core + Svelte/Tailwind UI)
> Creado: 2026-10-13

---

## 1. Objetivo
Desarrollar una suite moderna, nativa y ultraliviana para Linux que reemplace Logitech Options/Options+, enfocada en el mouse **Logitech MX Vertical** (extensible a la familia MX). La suite provee configuración completa de botones (especialmente el botón superior de DPI y laterales), emulación de atajos mediante `uinput`, control de perfiles y monitorización de batería en la bandeja del sistema.

---

## 2. Alcance y No-goals (SDD)

### En alcance (MVP):
1. **Core y Protocolo HID++ (`mx-core`):**
   - Handshake HID++ 2.0/4.x por USB, receptor (Unifying/Bolt) y Bluetooth.
   - Activación de *Button Diversion* para el botón superior y botones reprogramables.
   - Lectura de nivel de batería y estado de carga.
   - Control de DPI (si la feature de firmware está expuesta y verificada en F0).
2. **Emulación de Entrada (`mx-uinput`):**
   - Emisión de pulsaciones de teclado y clics mediante `/dev/uinput` a nivel de kernel (100% transparente para Wayland y X11).
3. **Servicio en segundo plano (`mx-daemon`):**
   - Servicio `systemd --user` que mantiene la conexión, maneja hotplug y suspensión/wake-up (re-aplica el divert).
   - Socket Unix local (`0600` en `$XDG_RUNTIME_DIR`) con validación `SO_PEERCRED`.
4. **Permisos sin root:**
   - Reglas `udev` con `TAG+="uaccess"` para `/dev/hidraw*` y `/dev/uinput`.
5. **Interfaz de usuario (`mx-gui` en Tauri v2):**
   - Frontend en Svelte + Tailwind con System Tray, selector de acciones y visualización de batería.
6. **Empaquetado:**
   - Paquete `.deb` primario para Ubuntu/Debian/Mint.

### Fuera de alcance (No-goals del MVP):
- Gestos complejos de movimiento de ratón (mouse gestures).
- Detección de ventana activa para cambio dinámico de perfiles por aplicación.
- Ejecución de comandos shell arbitrarios (`Exec`) a través del socket IPC.
- Soporte extendido para teclados u otros dispositivos de la familia MX (abstraer `DeviceProfile`, pero implementar solo MX Vertical en el MVP).

---

## 3. Metas Numéricas de Cierre (Pilar 7)

| Métrica | Objetivo Declarado | Medición / Árbitro |
|---|---|---|
| **Memoria del Daemon** | `< 15 MB` en reposo | `ps_mem` o `smem` sobre `mx-daemon` |
| **Memoria Total (Daemon + GUI)** | `< 80 MB` con ventana abierta | `ps_mem` sumado |
| **Latencia de Entrada** | `p95 < 20 ms` botón físico → evento `uinput` | Benchmark con timestamps hardware vs evtest |
| **Tamaño del Paquete** | `< 15 MB` binario empaquetado `.deb` | `ls -lh mx-options.deb` |
| **Confiabilidad de Pulsación** | `≥ 99%` eventos capturados sin pérdidas ni duplicados | Prueba de 10 min en `evtest` |
| **Resiliencia Bluetooth** | Sobrevivir a suspensión y `≥ 20` reconexiones BT continuas | Script de estrés de reconexión |
| **Tiempo de Reconexión** | `< 5 s` tras volver de suspensión o encender mouse | Medición con log udev/netlink |
| **Lectura de Batería** | Margen de error `±10%` comparado con Solaar | 10 muestras comparativas |

---

## 4. Política de Contexto y Alerta de Tokens

Para no perder contexto ni agotar la cuota de la sesión:
* **Alarma en 160k tokens:** No iniciar nuevas unidades de trabajo. Comenzar cierre documental en `STATUS.md` y preparar el relevo.
* **Aviso Final en 200k tokens:** Guardar todo cambio pendiente y sugerir al usuario rotar de sesión en un punto natural.
* **Mecanismo verificado tras update de Antigravity:** Hook global `PreInvocation` activo en `~/.gemini/config/hooks.json` ejecutando `~/.gemini/config/hooks/hook-contexto-agy.sh`. Probado funcional (inyecta advertencias sin bloquear herramientas).

---

## 5. Decisiones Arquitectónicas Cerradas

1. **Daemon Headless + GUI Cliente (Desacoplados):**  
   *Justificación:* El daemon corre como `systemd --user` asegurando que los botones funcionen siempre, incluso si la GUI está cerrada o el usuario no la abre al iniciar sesión.
2. **`uinput` como única vía de eventos:**  
   *Justificación:* Funciona universalmente en Wayland (GNOME, KDE, Hyprland, Sway) y X11 sin requerir hacks como `xdotool` o `ydotool`.
3. **Reglas `udev` con `TAG+="uaccess"`:**  
   *Justificación:* Otorga acceso seguro al usuario activo de escritorio sin requerir membresía a grupos de riesgo ni `sudo`.
4. **IPC mediante Unix Domain Socket con `SO_PEERCRED`:**  
   *Justificación:* Comunicación local rápida, acotada al UID del usuario, con permisos `0600` en `$XDG_RUNTIME_DIR`.
5. **Frontend Svelte + Tailwind en Tauri v2:**  
   *Justificación:* Mínimo tamaño de bundle, renderizado reactivo ultrarrápido y mínimo consumo de memoria frente a React o Electron.
6. **Descartar desensamblado del `.exe` / REA en camino crítico:**  
   *Justificación:* El protocolo HID++ ya está suficientemente documentado en `logiops`, `Solaar` y el kernel. Usar arte propio para la interfaz gráfica.

---

## 6. Carta Gantt y Matriz de Delegación de Subagentes

```
Fase 0: Sonda & Protocolo (mx-probe)   [████████████████████]  Kill-gate APROBADO (Hardware verificado)
Fase 1: Divert & uinput (mx-cli)       [████████████████████]  APROBADO (uinput + RAII verificado)
Fase 2: Permisos udev                  [████████████████████]  Reglas instaladas y funcionales
Fase 3: Daemon systemd & IPC           [░░░░░░░░░░░░░░░░░░░░]  Siguiente paso (Daemon & Socket)
Fase 4: GUI Tauri v2 (Svelte)          [░░░░░░░░░░░░░░░░░░░░]  Tray & remapeador
Fase 5: Empaquetado .deb               [░░░░░░░░░░░░░░░░░░░░]  Distribución limpia
```

| Fase | Tarea Principal | Modelo Principal / Delegación | Árbitro Objetivo (Criterio de Cierre) |
|---|---|---|---|
| **F0: Protocolo (`mx-probe`)** | Handshake HID++ 2.0/4.5, lectura de batería, dump de features en BT. | **Gemini 3.8 Flash High** (Orquestador) | **APROBADO:** 29 features volcadas, batería 20%, CID botón superior `0x00FD` divertible. Kill-gate superado. |
| **F1: Divert + `uinput` (`mx-cli`)** | Enviar paquete de diversion al CID `0x00FD`; emitir pulsación virtual vía `/dev/uinput`. | **Gemini 3.8 Flash High** + `opencode-go/claude-haiku-5-5` (specs & arnés TDD) | **APROBADO:** Dispositivo virtual `MX Options Virtual Device` activo, captura de eventos 0x1B04, emisión de Super/Meta y restauración RAII al salir. |
| **F2: Permisos `udev`** | Regla en `/etc/udev/rules.d/` con `TAG+="uaccess"` y `GROUP="plugdev"` para `046d` y `uinput`. | `opencode-go/deepseek-v4.1-flash` (revisión de reglas de seguridad Linux) | **APROBADO:** Ejecución de `mx-probe` y `mx-cli` como usuario normal sin `sudo`. |
| **F3: Daemon `systemd` & IPC** | Daemon en segundo plano con reconexión automática tras suspensión; socket Unix seguro. | **Gemini 3.8 Flash High** + `opencode-go/qwen3.8-flash` (resiliencia Bluetooth) | Supera suspensión + 20 reconexiones BT; reconexión < 5 s; p95 < 20 ms; RAM < 15 MB. |
| **F4: GUI Tauri v2** | Interfaz en Svelte + Tailwind con System Tray, selector de atajos y estado de batería. | **Gemini 3.8 Flash High** | RAM total < 80 MB; bundle < 15 MB; control reactivo del daemon vía socket. |
| **F5: Empaquetado & Convivencia** | Creación de `.deb`; detector de conflictos con `solaar` o `logid`. | **Gemini 3.8 Flash High** + `opencode-go/muse-spark-1.3` (telemetría / `mx-diag`) | Instalación limpia en Ubuntu/Debian; detección y aviso sin duplicar divert. |

---

## 7. Registro de Sesiones

### 2026-10-13 — Sesión 0: Inicialización y Gobernanza (Gemini 3.8 Flash High)
- Creación de `/home/panxio/Projects/mx-options-linux` mediante `factory/setup/init-project.sh` (perfil `tauri-app`).
- Migración y consolidación de decisiones de brainstorming de la sesión previa (`e3641738-eb0c-4c7a-8ef8-67bdb2465ed4`): síntesis de los 7 subagentes OpenCode Go y Gemini.
- Creación de `docs/BRAINSTORMING.md`, `docs/SUBAGENTES.md`, `scripts/run-dev.sh` y `hooks/`.
- Definición de objetivos, no-goals, guante de restricciones y metas numéricas (Pilar 7).

### 2026-10-13 — Sesión 1: Validación de Entorno Antigravity 2.0 y Plan de Fases (Gemini 3.8 Flash High)
- **Verificación post-actualización de Antigravity 2.0:**
  - El hook de cuota de tokens (`hook-contexto-agy.sh`) fue testeado con simulación de 165k tokens: funciona al 100% inyectando la alarma efímera sin romper herramientas.
  - Se configuró el archivo de permisos del nuevo proyecto en `~/.gemini/config/projects/3a078d09-6b3d-4d17-92ee-6238cc30fea1.json` con `autoExecutionPolicy` para evitar prompts continuos en `/home/panxio/Projects/mx-options-linux`.
  - Se catalogaron los modelos activos en `agy` (`gemini-3.8-flash-high`, etc.) y `opencode` (`deepseek-v4.1-flash`, `claude-haiku-5-5`, etc.).
- Elaboración de la Carta Gantt y matriz de asignación de subagentes para las fases F0 a F5.
- Estado: Plan aprobado por el usuario e inicio de la Fase 0.

### 2026-10-13 — Sesión 2: Cierre Exitoso de Fase 0 (Kill-gate APROBADO) (Gemini 3.8 Flash High)
- **SDD:** Redacción formal de la especificación técnica en `specs/F0-mx-probe.md`.
- **Workspace Cargo:** Creación del Cargo workspace con crates `mx-core` y `mx-probe`.
- **TDD en Rust:**
  - `mx-core`: Implementación de `HidppFrame`, `Transport`, `MockTransport`, `BatteryInfo`, `KeyInfo` y `HidppDevice`.
  - 10 pruebas unitarias pasando al 100% en menos de 0.01 s.
  - `cargo clippy --workspace -- -D warnings` pasando con 0 advertencias.
- **Diagnóstico y udev (Pilar 6 - Diagnóstico de causa raíz):**
  - Se descubrió que los nodos Bluetooth LE bajo `/devices/virtual/misc/uhid/` no reciben ACL automática de `uaccess`.
  - Solución: Regla `99-mx-options.rules` con `MODE="0660", GROUP="plugdev"`, permitiendo ejecución sin `sudo`.
- **Hallazgo Crítico en Bluetooth (Long Reports 0x11):**
  - El descriptor HID de Bluetooth del MX Vertical no expone reportes cortos `0x10`, solo reportes largos de 20 bytes (`0x11`).
  - Se refactorizó `HidppDevice` para utilizar tramas largas universales de 20 bytes.
- **Validación del Kill-Gate en Hardware Real (Logitech MX Vertical `046D:B020`):**
  - Handshake HID++ 4.5 exitoso en índice `0xFF`.
  - Batería leída: `20% (Discharging)` (coincidente con Bluez).
  - Feature `0x1B04` detectada en índice `0x0A`.
  - **7 botones reprogramables identificados.**
  - **Botón superior de DPI identificado con CID exacto `0x00FD`, Flags `0x71`, Divert: SÍ.**
  - **Feature `0x2201` (Adjustable DPI) expuesta en índice `0x12` (control de DPI por software disponible).**
  - 29 características volcadas en total.
- **Estado:** Fase 0 cerrada con éxito rotundo.

### 2026-10-13 — Sesión 3: Fase 1 Completada (Divert & Emulación uinput en mx-cli) (Gemini 3.8 Flash High)
- **SDD:** Redacción formal de la especificación técnica en `specs/F1-divert-uinput.md`.
- **Módulo de Entrada Virtual (`mx-uinput`):**
  - Implementación de `VirtualInput` usando el crate `evdev` de Linux.
  - Creación del nodo de kernel `MX Options Virtual Device` con teclas modificadoras, alfanuméricas, multimedia y botones de mouse.
  - Abstracción de acciones en `Action` (Super/Meta, Ctrl+C, Ctrl+V, Alt+Tab, Clics de ratón).
- **Protocolo de Diversion en `mx-core`:**
  - Implementación de `set_cid_divert` para enviar `setCidReporting` (Feature `0x1B04`, función `0x03`).
  - Implementación de `DivertedButtonsEvent` para decodificar reportes de notificación asíncronos (`Event 0`).
  - Refactorización de `HidrawTransport` y `scan_hidraw_devices` al crate `mx-core` para consumo compartido.
- **Herramienta CLI interactiva (`mx-cli`):**
  - Soporte de subcomandos: `divert`, `status`, `reset`.
  - Ciclo de vida RAII: al terminar por `Ctrl+C` o señal `SIGTERM`, desactiva el divert en el firmware y destruye el dispositivo uinput sin dejar botones muertos.
- **Verificación y Pruebas en Hardware Real:**
  - 13/13 tests unitarios pasando en 0.00 s en el workspace.
  - `cargo clippy --workspace -- -D warnings` con 0 advertencias.
  - Compilación release exitosa con binarios ultralivianos (`mx-cli`: 2.1 MB, `mx-probe`: 2.0 MB, `mx-diag`: 530 KB).
  - Verificado en hardware real: pulsación del botón superior (CID 0x00FD) inyectó `SingleKey(KEY_LEFTMETA)` (Super/Meta) en GNOME Wayland con captura asíncrona.

### 2026-10-13 — Sesión 4: Auditoría Grupal F0 & F1 (7 Subagentes) y Monitoreo de Tanda (Gemini 3.8 Flash High)
- **Ejecución de la Tanda Colegiada:**
  - Se lanzó `scripts/run-audit-tanda.sh` con los 7 subagentes bajo `systemd-run --user --scope -p MemoryMax=2G`, `< /dev/null` y concurrencia máxima de 2 puestos simultáneos.
  - Modelos evaluadores: `deepseek-v4.1-flash`, `claude-haiku-5-5`, `muse-spark-1.3-contributor`, `glm-5.3-flash`, `gpt-6-luna`, `mimo-v2.6-flash`, `qwen3.8-flash`.
  - Monitoreo en vivo de PIDs, timestamps, exit codes y tamaño de logs en `.superpowers/logs/`.
  - Más de 85.000 caracteres de análisis técnico extraídos en `.superpowers/audit_f0_f1/reports/`.
- **Dictamen Consolidado en [`docs/AUDITORIA_FASE_0_1.md`](docs/AUDITORIA_FASE_0_1.md):**
  - **Calificación:** 🟡 Aprobado como Prototipo / 🔴 Riesgo Alto para Daemon si no se aplican mejoras previas.
  - **Hallazgos Críticos detectados:**
    1. *H1:* Lazo de CPU al 100% en `mx-cli` cuando el ratón se apaga o pierde Bluetooth (`POLLERR/POLLHUP` no tratado).
    2. *H2:* La promesa de "Zero Dead Buttons" / RAII no está implementada con `impl Drop` (solo código lineal al final de `while`); un `SIGKILL` o caída deja el botón muerto en firmware sin reconciliación al inicio.
    3. *H3:* Semántica Down/Up rota: el código emite un clic completo instantáneo (`emit_click`) al detectar press, impidiendo mantener teclas presionadas (hold, drag con Super, Alt+Tab).
    4. *H4:* `sleep(10ms)` bloqueante en el camino caliente del bucle de lectura de `/dev/hidraw`.
    5. *H5:* `device_index = 0xFF` hardcodeado en `mx-cli`, incompatible con receptores Unifying/Bolt (`0x01..0x06`).
    6. *H6:* Regla udev en `99-*.rules` evalúa `uaccess` tarde (después de `73-seat-late.rules`); dependencia de grupo `plugdev` no portable a Arch/Fedora.
    7. *H7:* Bug en decodificación de reportes de error HID++ 1.0 (`0x8F`) en `frame.rs`.
- **Estado Actual:** Auditoría completada y documentada. Próximo paso: aplicar el Paquete de Parches de Robustez (H1 a H7) y proceder a la especificación de la **Fase 3 (Daemon `systemd --user` & Socket IPC)** con diseño ultra-liviano (<15 MB RAM).

