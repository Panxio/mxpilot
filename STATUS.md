# STATUS.md — MX Options Linux

> Bitácora de continuidad entre sesiones. El agente NO recuerda entre sesiones:
> LÉEME PRIMERO al retomar, y ACTUALÍZAME al terminar.
> Perfil del proyecto: tauri-app (Rust Core + Svelte/Tailwind UI)
> Creado: 2026-10-13

> ### 📌 RESUMEN EJECUTIVO DE RELEVO RÁPIDO (LEER AL INICIAR SESIÓN)
> - **Última Sesión Cerrada:** Sesión 8 (Apertura Fase 4: GUI Tauri v2 & System Tray — Revisión Colegiada Completada).
> - **Fases Aprobadas:**
>   - ✅ **Fase 0:** Sonda & Protocolo HID++ (`mx-probe`) — Kill-gate superado.
>   - ✅ **Fase 1:** Divert de Botón Superior & Inyección Virtual (`mx-uinput`, `mx-cli`) — RAII y Down/Up verificado.
>   - ✅ **Fase 2:** Reglas `udev` (`70-mx-options.rules`) — Permisos de usuario sin `sudo`.
>   - ✅ **Fase 3:** Daemon `systemd --user` & Socket IPC (`mx-daemon`) — 28/28 tests pasando, 0 clippy warnings, RSS 3.3 MB (<15 MB), parada limpia <0.1 s, fail-safe Zero Stuck Keys verificado, auditoría colegiada de 4 subagentes superada.
> - **Fase en Progreso:** **Fase 4: GUI Tauri v2 (Svelte + Tailwind & System Tray)**.
> - **Acción Inmediata al Retomar:**
>   1. Inicializar el crate `crates/mx-gui` en el workspace de Cargo y configurar Tauri v2 (`Cargo.toml`, `tauri.conf.json`).
>   2. Implementar `client.rs` (cliente IPC bounded con deadline en `spawn_blocking`) y arnés TDD `tests/client_mock_test.rs`.
>   3. Implementar comandos asíncronos `commands.rs`, motor de sondeo `poller.rs` y System Tray `tray.rs`.
>   4. Montar frontend Svelte + Tailwind en `crates/mx-gui/ui`.

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

### 4.1 Protocolo de Flujo de Trabajo (Inicio y Cierre con Subagentes)
Para garantizar calidad sostenida y mejora continua en cada sesión y fase:
1. **Inicio de Sesión / Apertura de Fase:** Planificación inicial por el orquestador (`specs/`, plan artefacto) $\to$ Despacho de revisión crítica a 4 subagentes (`haiku`, `luna`, `deepseek`, `muse`) $\to$ Consolidación de mejoras en la spec y el plan antes de implementar.
2. **Cierre de Fase / Entrega:** Implementación y validación de compuertas duras (`cargo test`, `clippy`, métricas Pilar 7) $\to$ Emisión de informe de entrega $\to$ Auditoría con los mismos 4 subagentes en busca de regresiones y mejoras $\to$ Aplicación de parches antes del cierre definitivo de la fase.

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
Fase 3: Daemon systemd & IPC           [████████████████████]  APROBADO (Daemon, IPC & Audit verificado)
Fase 4: GUI Tauri v2 (Svelte)          [██████░░░░░░░░░░░░░░]  En progreso (Revisión colegiada aprobada)
Fase 5: Empaquetado .deb               [░░░░░░░░░░░░░░░░░░░░]  Distribución limpia
```

| Fase | Tarea Principal | Modelo Principal / Delegación | Árbitro Objetivo (Criterio de Cierre) |
|---|---|---|---|
| **F0: Protocolo (`mx-probe`)** | Handshake HID++ 2.0/4.5, lectura de batería, dump de features en BT. | **Gemini 3.8 Flash High** (Orquestador) | **APROBADO:** 29 features volcadas, batería 20%, CID botón superior `0x00FD` divertible. Kill-gate superado. |
| **F1: Divert + `uinput` (`mx-cli`)** | Enviar paquete de diversion al CID `0x00FD`; emitir pulsación virtual vía `/dev/uinput`. | **Gemini 3.8 Flash High** + `opencode-go/claude-haiku-5-5` (specs & arnés TDD) | **APROBADO:** Dispositivo virtual `MX Options Virtual Device` activo, captura de eventos 0x1B04, emisión de Super/Meta y restauración RAII al salir. |
| **F2: Permisos `udev`** | Regla en `/etc/udev/rules.d/` con `TAG+="uaccess"` y `GROUP="plugdev"` para `046d` y `uinput`. | `opencode-go/deepseek-v4.1-flash` (revisión de reglas de seguridad Linux) | **APROBADO:** Ejecución de `mx-probe` y `mx-cli` como usuario normal sin `sudo`. |
| **F3: Daemon `systemd` & IPC** | Daemon en segundo plano con reconexión automática tras suspensión; socket Unix seguro. | **Gemini 3.8 Flash High** + 4 subagentes (`haiku`, `luna`, `deepseek`, `muse`) | **APROBADO:** 28 tests pasando, clippy 0 warnings, RSS en hardware 3.3 MB (<15 MB), parada limpia <0.1 s, IPC seguro SO_PEERCRED, auditoría colegiada cerrada. |
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
### 2026-10-13 — Sesión 5: Aplicación y Verificación del Paquete de Parches de Robustez (H1 a H7) (Gemini 3.8 Flash High)
- **Implementación de Parches de Robustez:**
  - **H1 (Desconexión limpia):** `hidraw.rs` captura `POLLHUP`, `POLLERR`, `POLLNVAL`, `EOF` y `ENODEV/EIO` emitiendo `HidppError::Disconnected`. `mx-cli` sale limpiamente sin busy-loop a 100% CPU. `EINTR` tratado como transitorio.
  - **H2 (Guardián RAII Real):** Implementado `DivertGuard<'a, T: Transport>` con `impl Drop` que envía `bfield = 0x02` al salir. Añadida reconciliación al arranque (`reset_all_diverts()`) para normalizar el hardware ante caídas previas. Nueva prueba unitaria `test_divert_guard_raii_drop` pasando en 0.00 s.
  - **H3 & H4 (Semántica Down/Up y Cero Sleeps):** Implementados `emit_action_down` y `emit_action_up` en `mx-uinput`, eliminando los `thread::sleep` bloqueantes del bucle lector. Ahora soporta pulsación sostenida (Hold), arrastre de ventanas con Super en Wayland/X11 y combinaciones sostenidas (`Alt+Tab`).
  - **H5 (Autoconexión Dinámica):** `HidppDevice::auto_connect(transport)` sondea `0xFF` (Bluetooth) y slots `0x01..=0x04` (Unifying/Bolt). Eliminado hardcodeo de `0xFF` en `mx-cli`.
  - **H6 (Reglas udev prioritarias):** Renombrada regla a `rules/70-mx-options.rules` (previo a `73-seat-late.rules`) para correcta asignación de `TAG+="uaccess"`. Actualizado `scripts/install-udev-rules.sh`.
  - **H7 (Error 0x8F):** Corregida decodificación de tramas de error `0x8F` en `frame.rs` y actualizada prueba unitaria.
- **Verificación en Hardware y Compilación:**
  - 14/14 tests unitarios en verde en 0.00 s.
  - `cargo clippy --workspace -- -D warnings` con 0 advertencias.
  - Binarios release compilados exitosamente (`mx-cli`, `mx-probe`, `mx-diag`).
  - `cargo run -p mx-cli -- status` probado en hardware real (`/dev/hidraw3`): autoconexión en `0xFF` exitosa, batería 20%, 7 botones listados.
- **Commit:** `e6db570` (`feat(core,uinput,cli,rules): apply post-audit robustness patches (H1-H7)`).

### 2026-10-13 — Sesión 6: Especificación SDD de Fase 3 y Revisión Colegiada (4 Subagentes) (Gemini 3.8 Flash High)
- **Redacción Inicial SDD:** Creación de `specs/F3-daemon-ipc.md` y del plan de implementación en `f3_daemon_ipc_plan.md`.
- **Tanda de Revisión Colegiada (4 Subagentes):**
  - Despacho y monitoreo mediante `scripts/run-f3-review-tanda.sh` bajo `systemd-run --user --scope -p MemoryMax=2G`, `< /dev/null`, timeout y concurrencia máxima de 2 puestos.
  - Modelos revisores: `deepseek-v4.1-flash`, `claude-haiku-5-5`, `muse-spark-1.3-contributor`, `gpt-6-luna`.
  - Más de 61.000 caracteres de análisis técnico extraídos en `.superpowers/review_f3/reports/`.
- **Incorporación de Consensos Críticos en `specs/F3-daemon-ipc.md`:**
  1. *Actor de Dueño Único:* Solo `hid_worker` toca `/dev/hidraw`. El IPC interactúa mediante canal de comandos (`DaemonCommand`), erradicando contención y robo de tramas (*frame-stealing*).
  2. *Framing Bounded & Anti Slow-Loris:* Acumulador por conexión con cota estricta `Read::take(64 KB)` y timeout de inactividad de 2 s por línea incompleta.
  3. *Zero Stuck Keys (Fail-Safe):* El despachador mantiene `pressed: HashSet<Key>` y emite `Key Up` inmediato de toda tecla retenida al desconectarse el mouse o salir de `ACTIVE`.
  4. *Ciclo de Vida de Canales Desacoplado:* `main` posee `sync_channel(64)`, `uinput_worker` persiste durante toda la sesión y los `hid_worker` efímeros usan `divert_epoch` monótono para descartar eventos rezagados.
  5. *Presupuesto de Parada (2 s):* Cacheo de `feature_index` de `0x1B04` y reseteo en parada exclusivamente de los CIDs activamente desviados, garantizando salida en $\le 2\text{ s}$ frente a `TimeoutStopSec=3s`.
  6. *Telemetría Machine-Readable:* Comando IPC `get_health` con métricas estructuradas para arbitraje cuantitativo (`mx-diag` y tests).
  7. *Sin Fallback Inseguro:* Eliminado fallback a `/tmp`; requerimiento estricto de `$XDG_RUNTIME_DIR` y `RuntimeDirectory=mx-options` en systemd.
  8. *Validación en Frontera:* `set_action` valida CIDs y teclas contra allowlist estricta antes de tocar hardware.
- **Estado:** Especificación SDD blindada y consensos aprobados por el usuario.

### 2026-10-13 — Sesión 7: Implementación de Fase 3 (mx-daemon), Auditoría Colegiada y Cierre (Gemini 3.8 Flash High)
- **Implementación de `crates/mx-daemon`:**
  - `src/ipc/protocol.rs`: Framing NDJSON, comandos `ping`, `get_status`, `get_health`, `get_battery`, `set_action`, `reload_profile`, allowlist estricta de CIDs y teclas evdev.
  - `src/ipc/server.rs`: Servidor Unix Domain Socket en `$XDG_RUNTIME_DIR/mx-options.sock`, permisos `0600`, validación de `SO_PEERCRED` por kernel UID, límite de 8 clientes, buffer bounded a 64 KB y timeout anti slow-loris de 2 s.
  - `src/lock.rs`: Exclusión mutua `flock(LOCK_EX | LOCK_NB)` en `$XDG_RUNTIME_DIR/mx-options.lock`.
  - `src/state.rs`: Máquina de estados (`Disconnected`, `Connecting`, `Reconciling`, `Active`, `ShuttingDown`), medición de VmRSS en tiempo real desde `/proc/self/status`.
  - `src/workers.rs`:
    - `hid_worker`: Actor de dueño único de `/dev/hidraw`, reconexión con backoff, canal de comandos sincrónico.
    - `uinput_worker`: Hilo persistente de inyección con `sync_channel(64)`, `pressed_keys: HashSet<KeyCode>` y fail-safe `ReleaseAll` en desconexión o cambio de época (`divert_epoch`).
  - `src/main.rs`: Manejo de señales (`SIGINT`, `SIGTERM`, `SIGPIPE`), ciclo de vida y apagado ordenado.
  - `systemd/user/mx-options.service`: Unidad de servicio de usuario con `Restart=on-failure`, `KillSignal=SIGTERM`, `TimeoutStopSec=3s`.
  - `scripts/install-systemd-service.sh`: Script de instalación automática del binario y servicio systemd.
- **Tanda Obligatoria de Auditoría de Cierre (4 Subagentes):**
  - Despachada mediante `scripts/run-f3-audit-cierre-tanda.sh` bajo `systemd-run -p MemoryMax=2G`, `< /dev/null`.
  - Modelos auditores: `opencode-go/deepseek-v4.1-flash`, `opencode-go/claude-haiku-5-5`, `opencode-go/muse-spark-1.3-contributor`, `gpt-6-luna`.
  - Reportes en `.superpowers/audit_cierre_f3/reports/` identificaron 4 observaciones críticas (C1–C4) y mejoras de calidad.
- **Parches Aplicados Post-Auditoría:**
  - *Parche P1 (Systemd syntax):* Claves `StartLimitIntervalSec=30s` y `StartLimitBurst=5` trasladadas a sección `[Unit]`. Verificado con `systemd-analyze --user verify`.
  - *Parche P2 (Zero Stuck Keys en remap):* Almacenamiento de la acción presionada exacta en `Down` para liberar esa misma acción en `Up`, impidiendo teclas pegadas si el mapeo cambia durante la pulsación.
  - *Parche P3 (Estado honesto - Regla de Oro):* La transición a `State::Active` y publicación de `diverted_cids` se realiza estrictamente tras confirmación exitosa de `device.set_cid_divert` en hardware.
  - *Parche P4 (Atomicidad de SetAction):* El mapeo en memoria `DaemonState.mapping` se actualiza exclusivamente cuando el hardware confirma la aplicación de la acción (`Ok(())`).
  - *Parche P5 (Caché y TTL de Batería):* `GetBattery` evalúa `BATTERY_CACHE_TTL_SECS = 60s`. Si expira o no existe, solicita refresco al `hid_worker` vía `DaemonCommand::RefreshBattery`.
  - *Parche P6 (Telemetría completa de salud):* Exposición de `device_index`, `consecutive_failures`, `queue_depth` y conteo honesto de `dropped_events` en `DaemonHealth`.
  - *Parche P7 (Arnés y Concurrencia de Tests):* Erradicación de `umask(0177)` global de proceso que provocaba carreras de permisos entre hilos de test. Incorporación de 4 tests de integración en `tests/ipc_tests.rs` validando atomicidad, allowlist, TTL y rechazo estricto de payloads > 64 KB.
- **Medición Numérica de Cierre (Pilar 7):**
  - Tests automáticos: **28/28 tests pasando al 100%** (`mx-core` 13, `mx-daemon` lib 10, integración 4, `mx-uinput` 1).
  - Clippy: **0 warnings** con `cargo clippy --workspace --all-targets -- -D warnings`.
  - Tamaño de binario release: **2.6 MB** (sin strip) / ~1.2 MB (stripped) (meta: < 15 MB).
  - Memoria residente en hardware real: **`VmRSS = 3348 kB (~3.27 MB)`** (meta: < 15 MB).
  - Tiempo de parada limpia ante SIGTERM: **< 0.1 s** (meta: < 2 s).
- **Estado:** Fase 3 **APROBADA Y CERRADA**.

### 2026-10-13 — Sesión 8: Apertura de Fase 4 (GUI Tauri v2), Revisión Colegiada y Consolidación (Gemini 3.8 Flash High)
- **Apertura de Fase 4:**
  - Redacción inicial de la especificación técnica en `specs/F4-gui-tauri.md`.
  - Creación del plan de implementación y arquitectura en el artefacto `f4_gui_tauri_plan.md`.
- **Tanda Obligatoria de Revisión Colegiada (4 Subagentes):**
  - Despachada mediante `scripts/run-f4-review-tanda.sh` bajo `systemd-run -p MemoryMax=2G`, `< /dev/null` con concurrencia máxima de 2 puestos.
  - Modelos revisores: `opencode-go/deepseek-v4.1-flash`, `opencode-go/claude-haiku-5-5`, `opencode-go/muse-spark-1.3-contributor`, `gpt-6-luna`.
  - Reportes técnicos completos (>54.000 caracteres) archivados en `.superpowers/review_f4/reports/`.
- **Consensos Técnicos Críticos Integrados en `specs/F4-gui-tauri.md`:**
  1. *Comandos Tauri Asíncronos:* Todos los `#[tauri::command]` son `async fn` delegando el IO de socket a `tauri::async_runtime::spawn_blocking`, evitando congelar el event loop de GTK (Zero Hang UI).
  2. *Cliente IPC Bounded con Deadline:* Conexión no bloqueante con `poll` (timeout 500 ms) + R/W acotado por clase (FAST 1.5 s, SLOW 2.5 s) + `take(64 KB + 1)` para evitar desbordes de memoria.
  3. *Máquina de Estados de 4 Vías en UI (Regla de Oro - Cero Falso Verde):* Distinción clara entre `Connecting`, `Connected` (activo en hardware), `DaemonDown` (transporte caído) y `DeviceDisconnected` (mouse apagado/fuera de rango BT con mapeos atenuados *stale-while-revalidate*).
  4. *Motor Central de Polling en Rust:* Una tarea en segundo plano en Rust gobierna la cadencia (10 s visible / 60 s oculta en tray), actualiza el tray y emite eventos `status-changed` tipados al frontend.
  5. *System Tray Tolerante a Fallos:* Creación fallible para compatibilidad en GNOME Wayland sin AppIndicator (continúa en modo ventana sin crashear); intercepción de cierre con `api.prevent_close()` y `window.hide()`.
  6. *Invocación Segura de `systemctl`:* Argumentos fijos `Command::new("/usr/bin/systemctl").args(["--user", "start", "mx-options.service"])` con `Stdio::null()` y verificación de disponibilidad mediante `ping` con backoff.
  7. *Medición Honesta de Recursos:* Medición integral del árbol de procesos (`mx-gui` + `WebKitWebProcess` + `WebKitNetworkProcess`) vía cgroup `memory.current` con script dedicado `scripts/measure-gui.sh`.
- **Estado:** Especificación y plan blindados. Lista para iniciar scaffolding de `crates/mx-gui`.

---

## 8. Instrucciones Detalladas para la Implementación de Fase 4 (GUI Tauri v2 & Tray)

### Pasos de Ejecución Inmediatos:
1. **Creación del Crate `crates/mx-gui`:**
   - Registrar `crates/mx-gui` en `Cargo.toml` raíz.
   - Configurar `crates/mx-gui/Cargo.toml` con dependencias Tauri v2 (`tauri`, `tauri-build`, `tauri-plugin-single-instance`, `serde`, `serde_json`, `thiserror`, `log`).
   - Crear `tauri.conf.json` con configuración de ventana (760x540), identificador único, plugins y tray icon.
2. **Implementación del Backend Rust (`crates/mx-gui/src/`):**
   - `client.rs`: Cliente IPC sincrónico con conexión no bloqueante, deadline, timeouts FAST/SLOW y `Read::take(64 KB)`.
   - `tests/client_mock_test.rs`: Arnés de pruebas con mock `UnixListener` validando timeouts, EOF, rechazo de payloads >64 KB y estados de error.
   - `commands.rs`: Comandos `async fn` en `spawn_blocking`.
   - `poller.rs`: Motor de sondeo Rust que emite eventos `status-changed` y actualiza el tray.
   - `tray.rs`: Gestión del tray icon y menú contextual.
3. **Scaffolding del Frontend (`crates/mx-gui/ui/`):**
   - Configuración de Vite + Svelte + Tailwind.
   - Implementación de stores (`UiState` de 4 vías) y componentes: `MouseViewer`, `ActionPicker`, `BatteryBadge`, `HealthModal`.
4. **Verificación y Medición:**
   - `cargo test -p mx-gui` y `cargo clippy -- -D warnings`.
   - Script `scripts/measure-gui.sh` para verificar RAM total < 80 MB con ventana abierta y < 35 MB en tray.

