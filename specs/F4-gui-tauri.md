# Especificación Técnica (SDD) — F4: GUI Tauri v2 & System Tray (mx-gui)

> **Módulo:** `crates/mx-gui` (Backend Rust Tauri v2 + Frontend Svelte & Tailwind)  
> **Perfil:** tauri-app (Rust Core ultraliviano, frontend delgado, bajo consumo de recursos)  
> **Orquestador:** Gemini 3.8 Flash High  
> **Panel Colegiado de Revisión:** `deepseek-v4.1-flash`, `claude-haiku-5-5`, `muse-spark-1.3-contributor`, `gpt-6-luna`  
> **Consenso Integrado:** 2026-10-13 (Post-Revisión Colegiada)  
> **Árbitro Objetivo:** `cargo test --workspace`, `cargo clippy -- -D warnings`, RAM Daemon < 5 MB RSS, Binario GUI < 15 MB stripped, GUI WebKitGTK < 250 MB PSS, latencia UI < 50 ms  
> **Fecha:** 2026-10-13  

---

## 1. Propósito y Alcance

Esta especificación define la arquitectura, integración IPC, interfaz visual, gestión de ciclo de vida en la bandeja del sistema (System Tray) y criterios de confiabilidad para **`mx-gui`**, la aplicación gráfica de usuario para la suite `mx-options-linux`.

### 1.1 Objetivos en Alcance (Fase 4):
1. **Cliente IPC en Rust Puro Asíncrono y Bounded (`client.rs`):**
   - Conector sincrónico a través de Unix Domain Socket (`$XDG_RUNTIME_DIR/mx-options.sock`, con fallback estricto a `/run/user/{uid}/mx-options.sock`; sin fallback inseguro a `/tmp`).
   - Conexión no bloqueante con `poll` y timeout de conexión acotado a 500 ms (mitiga backlog lleno o socket residual).
   - Presupuesto de tiempo dual:
     - **Comandos FAST** (`ping`, `get_status`, `get_health`): Connect 500 ms, R/W timeout 1000 ms (total $\le 1.5\text{ s}$).
     - **Comandos SLOW** (`set_action`, `get_battery`, `reload_profile`): Connect 500 ms, R/W timeout 2500 ms (para acomodar el sondeo Bluetooth de hardware sin falsos timeouts).
   - Acumulación bounded con `BufReader::take(MAX_IPC_RESPONSE_BYTES + 1)` (64 KB + 1) y verificación de terminador `\n`. Si la respuesta excede la cota, retorna error inmediato y cierra conexión sin desbordar el heap.
   - Identificador `id` monótono atómico por petición y verificación de correspondencia en la respuesta.
   - Clasificación de errores tipada (`ClientError` con `thiserror`): `DaemonNotRunning`, `PermissionDenied`, `Timeout`, `ProtocolError`, `PayloadTooLarge`, `DaemonError`.
2. **Backend Tauri v2 Desacoplado del Hilo Principal (`src-tauri`):**
   - **Invariante Zero Hang UI:** Todos los comandos `#[tauri::command]` son `async fn` y ejecutan el IO de socket o llamadas a procesos externos en `tauri::async_runtime::spawn_blocking`, garantizando que el event loop de GTK jamás se congele.
   - Comandos tipados expuestos al frontend:
     - `get_status() -> Result<StatusData, ClientError>`
     - `get_health() -> Result<DaemonHealth, ClientError>`
     - `get_battery() -> Result<BatteryData, ClientError>`
     - `set_action(cid: u16, action: ActionPayload) -> Result<(), ClientError>`
     - `start_daemon_service() -> Result<String, ClientError>`
   - Invocación de `systemctl` segura y sin shell: `Command::new("/usr/bin/systemctl").args(["--user", "start", "mx-options.service"])`, con `Stdio::null()` en stdin y confirmación posterior vía `ping` con backoff (sin asumir éxito por código de salida).
   - Motor de Polling Centralizado en Rust: Tarea en segundo plano en Rust que gobierna la cadencia (10 s visible, 60 s oculta en tray), actualiza el ícono/tooltip del tray y emite eventos `status-changed` tipados al frontend.
   - Instancia única (`tauri-plugin-single-instance`): Si el usuario ejecuta la app estando ya abierta, la instancia existente se muestra, desminimiza y gana foco.
   - Integración con System Tray tolerante a fallos:
     - Creación fallible: Si no hay soporte AppIndicator (ej. GNOME vanilla sin extensión), la aplicación continúa con ventana normal sin abortar.
     - Si hay tray activo, `WindowEvent::CloseRequested` invoca `api.prevent_close()` y `window.hide()`. Si no hay tray disponible, se permite el cierre normal para no dejar la aplicación inaccesible.
     - Ícono dinámico según estado de conexión y nivel de batería.
     - Menú contextual: "Mostrar MX Options", "Batería: X%", separador, "Reiniciar Daemon", "Salir" (`is_quitting = true` y `app.exit(0)`).
3. **Frontend Reactivo en Svelte + Tailwind:**
   - Cero VDOM overhead; compilado ultraliviano a JavaScript vanilla.
   - Tailwind CSS optimizado y purgado (bundle CSS < 20 KB).
   - **Máquina de Estados Honesta de 4 Vías (Regla de Oro - Cero Falso Verde):**
     1. `Connecting`: Buscando al daemon o en proceso de reconexión.
     2. `Connected`: Daemon activo Y mouse conectado (`state == "Active"`).
     3. `DaemonDown`: Fallo de transporte socket. Muestra banner con botón de acción *"Iniciar Servicio"*.
     4. `DeviceDisconnected`: Daemon activo pero ratón apagado o fuera de alcance Bluetooth (`connected == false`). Conserva la configuración previa atenuada (*stale-while-revalidate*) con etiqueta *"actualizado hace Xs"*, sin pintar el ratón en verde.
   - Representación esquemática del mouse Logitech MX Vertical con selección intuitiva de botones:
     - **Botón Superior (DPI / Kill-Gate, CID `0x00FD`):** foco principal.
     - Botones laterales (Back `0x0053`, Forward `0x0056`).
     - Clic central (Middle `0x0052`).
   - Catálogo de acciones predefinidas con allowlist estricta validada en Rust:
     - "Super / Meta (Tecla Windows / Actividades)"
     - "Copiar (Ctrl + C)"
     - "Pegar (Ctrl + V)"
     - "Silenciar / Desmutear (KEY_MUTE)"
     - "Reproducir / Pausar (KEY_PLAYPAUSE)"
     - "Alt + Tab (Cambio de ventana)"
     - "Atajo de teclado personalizado" (captura de teclas dentro de la allowlist).
   - Reconciliación honesta en `set_action`: Botón deshabilitado mientras la petición está en vuelo; confirmación visual solo al recibir `Ok(())` del daemon; ante error o timeout, reconciliación con `get_status` sin desincronizar la UI.
   - Modal de Telemetría (`HealthModal`): Exposición completa de `DaemonHealth` (`state`, `uptime_seconds`, `device_index`, `divert_epoch`, `reconnect_count`, `consecutive_failures`, `queue_depth`, `dropped_events`, `last_error`, `rss_kb`) más VmRSS de la propia GUI (`/proc/self/status`).

### 1.2 Fuera de Alcance (No-Goals de Fase 4):
- Detección de ventana activa para cambio de perfiles dinámicos por aplicación.
- Gestos del ratón con arrastre (mouse gestures).
- Edición de curvas DPI complejas.

---

## 2. Definición de Constantes del Sistema (Pilar 1)

| Constante | Tipo | Valor | Justificación Técnica |
|---|---|---|---|
| `WINDOW_WIDTH` | `f64` | `760.0` | Ancho óptimo para panel dividido (vista gráfica de ratón + configurador). |
| `WINDOW_HEIGHT` | `f64` | `540.0` | Alto acotado que cabe en cualquier resolución (desde 768p en adelante). |
| `POLL_INTERVAL_ACTIVE_MS` | `u64` | `10_000` (10 s) | Frecuencia de refresco cuando la ventana está visible y activa. |
| `POLL_INTERVAL_BACKGROUND_MS`| `u64` | `60_000` (60 s) | Frecuencia de refresco cuando la ventana está oculta en el System Tray. |
| `IPC_CONNECT_TIMEOUT_MS` | `u64` | `500` (0.5 s) | Timeout en socket nonblocking + poll para conectar a `mx-options.sock`. |
| `IPC_FAST_TIMEOUT_MS` | `u64` | `1_000` (1 s) | Timeout de lectura/escritura para comandos rápidos (`ping`, `status`, `health`). |
| `IPC_SLOW_TIMEOUT_MS` | `u64` | `2_500` (2.5 s) | Timeout de lectura/escritura para comandos de hardware (`battery`, `set_action`). |
| `MAX_IPC_RESPONSE_BYTES` | `usize` | `65_536` (64 KB) | Tamaño máximo de respuesta esperado desde el daemon (`Read::take`). |
| `TOP_BUTTON_CID` | `u16` | `0x00FD` | CID del botón superior del Logitech MX Vertical (Kill-Gate). |

---

## 3. Invariantes del Sistema (Pilar 2)

1. **Invariante de No Bloqueo de UI (Zero Hang UI):**  
   Ningún comando Tauri se ejecuta en el hilo principal de GTK. Todo IO y ejecución de subprocesos se traslada a hilos de trabajo asíncronos (`spawn_blocking`) con deadlines absolutos. La UI responde a eventos de usuario en $< 50\text{ ms}$.
2. **Invariante de Cero Falsos Verdes (Regla de Oro):**  
   La interfaz gráfica nunca reporta éxito de conexión si el ratón o el daemon están caídos. La presencia del mouse se juzga estrictamente por `status.connected == true` y `health.state == "Active"`.
3. **Invariante de Frontera de Confianza (Frontend no confiable):**  
   El frontend de Svelte no tiene acceso a sockets Unix ni al sistema de archivos. Toda acción solicitada (`set_action`) es validada en Rust contra la allowlist estricta (`is_valid_cid` y `parse_key_code`) antes de despacharse al socket.
4. **Invariante de Instancia Única (Single Instance):**  
   No pueden existir dos procesos de la GUI ejecutándose en la misma sesión gráfica. El intento de abrir una segunda instancia debe transferir el foco a la primera y terminar.
5. **Invariante de Tolerancia de Bandeja:**  
   Si el entorno de escritorio no soporta AppIndicator / StatusNotifierWatcher, la GUI funciona normalmente en modo ventana sin fallar al arrancar.
6. **Invariante de Desacoplamiento (Daemon Autónomo):**  
   El cierre, caída o reinicio de la GUI NO afecta en lo más mínimo al funcionamiento de `mx-daemon`.

---

## 4. Arquitectura y Módulos

```
crates/mx-gui/
├── Cargo.toml                  # Dependencias Tauri v2, serde, mx-daemon (tipos)
├── tauri.conf.json             # Configuración Tauri v2 (ventana, tray, single-instance)
├── build.rs                    # tauri-build
├── src/
│   ├── main.rs                 # Inicialización de Tauri v2
│   ├── client.rs               # Cliente IPC con deadline, non-blocking connect y take bounded
│   ├── commands.rs             # #[tauri::command] handlers asíncronos (spawn_blocking)
│   ├── poller.rs               # Motor de sondeo en segundo plano en Rust
│   └── tray.rs                 # Gestión del System Tray e íconos dinámicos
├── ui/                         # Frontend Web
│   ├── package.json            # Vite, Svelte, Tailwind CSS
│   ├── vite.config.js          # Configuración de empaquetado Vite
│   ├── tailwind.config.js      # Configuración de estilos Tailwind
│   ├── src/
│   │   ├── App.svelte          # Componente principal
│   │   ├── main.js             # Entrada frontend
│   │   ├── lib/
│   │   │   ├── api.js          # Invocación tipada de comandos Tauri (@tauri-apps/api/core)
│   │   │   ├── stores.js       # Stores reactivos Svelte (UiState, status, battery, health)
│   │   │   ├── MouseViewer.svelte # Representación esquemática del MX Vertical
│   │   │   ├── ActionPicker.svelte # Selector y configurador de acciones
│   │   │   ├── BatteryBadge.svelte # Badge de batería dinámico
│   │   │   └── HealthModal.svelte  # Panel de telemetría y diagnóstico
│   │   └── assets/             # Íconos PNG precargados (batería, estados)
└── tests/
    └── client_mock_test.rs     # Arnés de prueba con mock UnixListener (timeouts, EOF, cota)
```

---

## 5. Metas Numéricas y Árbitro de Cierre (Pilar 7)

| Métrica | Objetivo Declarado | Herramienta de Medición / Árbitro |
|---|---|---|
| **RAM Daemon (`mx-daemon`)** | `< 5 MB` RSS (Real: 2.3 MB) | `/proc/$(pgrep mx-daemon)/status` (VmRSS) |
| **RAM GUI (Árbol WebKitGTK)** | `< 250 MB` PSS (Real: ~230 MB PSS) | `scripts/measure-gui.sh` (PSS de procesos WebKitGTK) |
| **RAM en Bandeja (Oculta)** | Disminución de ciclos de render | `scripts/measure-gui.sh` tras 60 s oculta |
| **Tamaño de Binario** | `< 15 MB` (release stripped; Real: 10.5 MB) | `ls -lh target/release/mx-gui` |
| **Latencia UI (clic -> feedback)**| `< 50 ms` | `performance.now()` en consola frontend |
| **Consumo CPU en reposo** | `< 0.2%` CPU | `top -b -n 1 -p $(pgrep -d, -f mx-gui)` |
| **Tests Unitarios Rust** | 100% pasando (36/36) | `cargo test --workspace` |
| **Clippy** | 0 warnings | `cargo clippy --workspace --all-targets -- -D warnings` |
