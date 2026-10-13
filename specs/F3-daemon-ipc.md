# Especificación Técnica (SDD) — F3: Daemon systemd --user & Socket IPC (mx-daemon)

> **Módulo:** `crates/mx-daemon`  
> **Perfil:** tauri-app (Rust Core ultraliviano, < 15 MB RAM)  
> **Revisado por:** Panel Colegiado de 4 Subagentes (`deepseek`, `haiku`, `muse`, `luna`) — 2026-10-13  
> **Árbitro Objetivo:** `cargo test --workspace`, clippy 0 warnings, memoria RSS < 15 MB, reconexión < 5 s, 20/20 reconexiones sin teclas pegadas  
> **Fecha:** 2026-10-13  

---

## 1. Propósito y Alcance

Esta especificación define la arquitectura, protocolo IPC, concurrencia, máquina de estados y ciclo de vida de **`mx-daemon`**, el servicio en segundo plano de nivel de usuario (`systemd --user`) encargado de:

1. **Gestión de Ciclo de Vida del Hardware y Dueño Único:** Detectar el ratón Logitech MX Vertical (`046d:b020`), mantener la sesión HID++ bajo **un único hilo actor dueño exclusivo** (`hid_worker`), aplicar la reconciliación honesta al inicio, activar el desvío de botones (*Button Diversion*) y reconectar automáticamente tras pérdidas de enlace Bluetooth, apagado del ratón o suspensión/reanudación del sistema.
2. **Emulación uinput Desacoplada y Fail-Safe:** Mantener activo el nodo virtual `/dev/uinput` (`MX Options Virtual Device`) con ciclo de vida persistente durante toda la sesión del daemon. Despachar pulsaciones y liberaciones (Down / Up) mediante canal acotado `sync_channel(64)` con latencia interna $p95 < 2\text{ ms}$ (extremo a extremo $p95 < 20\text{ ms}$). Ante cualquier desconexión, emitir **liberación automática (Up) de toda tecla retenida**, previniendo teclas atascadas en el sistema operativo.
3. **Servidor IPC Local Blindado:** Exponer un Unix Domain Socket en `$XDG_RUNTIME_DIR/mx-options.sock` (permisos `0600`, validación `SO_PEERCRED`). Framing delimitado por `\n` con acumulador acotado en `Read::take(64 KB)` y timeout de inactividad de 2 s para evitar ataques slow-loris y desbordes de heap. Sin fallback inseguro a carpetas compartidas (`/tmp`).
4. **Telemetría y Observabilidad Machine-Readable:** Endpoint `get_health` con métricas en tiempo real (`state`, `uptime`, `reconnect_count`, `dropped_events`, `divert_epoch`, `last_error`, `rss_kb`) permitiendo a `mx-diag` y a los tests actuar como árbitros objetivos.
5. **Arquitectura Ultra-Liviana:** Consumo estricto de memoria en reposo $< 15\text{ MB}$ RSS ($\sim 3\text{ MB}$ nominal) utilizando primitivas de `std::os::unix`, `std::sync::mpsc` y `libc::poll`, evitando un runtime multihilo asíncrono sobredimensionado.

---

## 2. Definición de Constantes del Sistema (Pilar 1)

Todas las constantes que gobiernan el daemon se declaran a continuación con su valor y justificación técnica:

| Constante | Tipo | Valor | Justificación Técnica |
|---|---|---|---|
| `SOCKET_FILE_NAME` | `&str` | `"mx-options.sock"` | Nombre del socket IPC Unix en `$XDG_RUNTIME_DIR`. |
| `LOCK_FILE_NAME` | `&str` | `"mx-options.lock"` | Nombre del archivo de bloqueo singleton (`flock`). |
| `SOCKET_FILE_MODE` | `u32` | `0o600` | Permisos Unix: solo lectura/escritura para el usuario dueño. |
| `MAX_IPC_PAYLOAD_SIZE` | `usize` | `65_536` (64 KB) | Límite máximo estricto de mensaje entrante acumulado antes del `\n`. |
| `IPC_LINE_TIMEOUT_MS` | `u64` | `2_000` (2 s) | Timeout máximo para completar una línea JSON incompleta (mitiga slow-loris). |
| `MAX_IPC_CLIENTS` | `usize` | `8` | Límite de conexiones simultáneas. Al llegar al 9º cliente se responde error y cierra. |
| `POLL_TIMEOUT_MS` | `i32` | `100` | Tiempo de espera en `libc::poll` en el hilo lector para chequear banderas atómicas. |
| `RECONNECT_INITIAL_MS` | `u64` | `1_000` (1 s) | Tiempo inicial de sondeo tras desconexión del ratón. |
| `RECONNECT_MAX_MS` | `u64` | `4_000` (4 s) | Techo de backoff para sondeo de reconexión. |
| `SYSTEMD_STOP_TIMEOUT_S`| `u32` | `3` | Tiempo concedido por `systemd` antes de `SIGKILL`. |
| `BATTERY_CACHE_TTL_SECS`| `u64` | `60` (1 min) | Período de validez de caché de batería para evitar saturación del bus Bluetooth. |
| `UINPUT_CHANNEL_BOUND` | `usize` | `64` | Capacidad del `sync_channel` entre `hid_worker` y `uinput_worker`. |
| `CIRCUIT_BREAKER_MAX_FAILS` | `usize` | `5` | Máximo de fallos de reconexión en 30 s antes de pasar a backoff extendido (10 s). |

---

## 3. Protocolo IPC sobre Unix Domain Socket

### 3.1 Ubicación y Mecanismo de Seguridad
- **Ruta Estricta:** `$XDG_RUNTIME_DIR/mx-options.sock`. Si `$XDG_RUNTIME_DIR` no está definido en el entorno, el daemon aborta con error explícito descriptivo (`XDG_RUNTIME_DIR missing`). Se descarta cualquier fallback inseguro a carpetas temporales compartidas como `/tmp`.
- **Creación y Permisos:** Se fija `umask(0o177)` previo al `bind` y se verifica `fchmod(fd, 0o600)` antes del `listen`.
- **Validación `SO_PEERCRED`:** Mediante `UnixStream::peer_cred()` de `std` (disponible de forma nativa desde Rust 1.79 sin requerir FFI inseguro). Si `cred.uid != getuid()`, la conexión se rechaza y cierra inmediatamente sin procesar ningún byte.

### 3.2 Framing Acotado y Manejo de Conexiones
1. **Acumulador Incremental con Cota Bounded:** Cada cliente posee un acumulador en memoria limitado a `MAX_IPC_PAYLOAD_SIZE` (64 KB). Si los bytes recibidos antes de un salto de línea `\n` superan este límite, la conexión emite `{"status":"error","message":"payload too large"}` y se cierra de inmediato sin alocar memoria en el heap.
2. **Timeout de Línea Incompleta:** Si un cliente envía una petición parcial y permanece inactivo por más de `IPC_LINE_TIMEOUT_MS` (2 s), el buffer se descarta y la conexión se cierra.
3. **Múltiples Mensajes por Lectura:** Si un `read()` contiene múltiples líneas delimitadas por `\n`, el despachador procesa cada línea secuencialmente y emite su respuesta correspondiente.
4. **Idempotencia y Sin SIGPIPE:** Escrituras que fallen con `BrokenPipe` / `EPIPE` se descartan silenciosamente sin afectar al daemon ni a otros clientes.

### 3.3 Esquema de Peticiones (`IpcRequest`)
Todas las peticiones incorporan un campo opcional `id` numérico que es devuelto en la respuesta para facilitar la correlación asíncrona en clientes GUI:

```json
// 1. Consulta de estado funcional:
{"id": 1, "cmd": "get_status"}

// 2. Consulta de salud y telemetría (machine-readable):
{"id": 2, "cmd": "get_health"}

// 3. Consulta de nivel de batería:
{"id": 3, "cmd": "get_battery"}

// 4. Configuración de acción de un botón (validado contra allowlist):
{
  "id": 4,
  "cmd": "set_action",
  "cid": 253,
  "action": {
    "type": "SingleKey",
    "key": "KEY_LEFTMETA"
  }
}

// 5. Recargar perfil desde configuración:
{"id": 5, "cmd": "reload_profile"}

// 6. Verificación de liveness:
{"id": 6, "cmd": "ping"}
```

#### Validación en Frontera de Confianza (`set_action`):
- `cid`: Debe pertenecer a la lista de botones divertibles conocidos (`0x00FD`, etc.) o descubiertos por la feature `0x1B04`.
- `key`: Debe pertenecer a la allowlist de teclas soportadas por `VirtualInput` (`KEY_LEFTMETA`, `KEY_LEFTCTRL`, etc.).
- Si la validación falla, se retorna error inmediato `{"status":"error","message":"invalid cid or key"}` sin tocar el hardware.

### 3.4 Esquema de Respuestas (`IpcResponse`)

```json
// Respuesta a "get_health":
{
  "id": 2,
  "status": "ok",
  "data": {
    "state": "Active",
    "uptime_seconds": 1240,
    "device_name": "Logitech MX Vertical",
    "device_path": "/dev/hidraw3",
    "device_index": 255,
    "divert_epoch": 3,
    "reconnect_count": 2,
    "consecutive_failures": 0,
    "queue_depth": 0,
    "dropped_events": 0,
    "last_error": null,
    "rss_kb": 3120
  }
}

// Respuesta a "get_status":
{
  "id": 1,
  "status": "ok",
  "data": {
    "connected": true,
    "device_name": "Logitech MX Vertical",
    "device_path": "/dev/hidraw3",
    "battery_level": 20,
    "battery_status": "Discharging",
    "diverted_cids": [253],
    "mapping": {
      "253": {
        "type": "SingleKey",
        "key": "KEY_LEFTMETA"
      }
    }
  }
}

// Respuesta de Error:
{
  "id": 4,
  "status": "error",
  "message": "Device not connected"
}
```

---

## 4. Arquitectura de Concurrencia y Máquina de Estados

### 4.1 Principio de Dueño Único (Actor `hid_worker`)
Para erradicar por completo la contienda de descriptores y el robo de tramas (*frame-stealing*):
1. **`hid_worker` es el ÚNICO hilo que interactúa con `HidppDevice` y lee `/dev/hidraw`.**
2. El hilo principal (servidor IPC) **nunca** lee ni escribe en `/dev/hidraw`. Cuando el cliente solicita una acción que requiere hardware (`set_action`, refresco forzado de batería o reconciliación), envía un `DaemonCommand` a través de un canal hacia `hid_worker` y aguarda la respuesta vía un canal one-shot (`oneshot::channel` o `mpsc::sync_channel(1)`).
3. Las consultas rápidas (`get_status`, `get_health`, `ping`) se resuelven de forma **inmediata en $O(1)$** leyendo un `StateSnapshot` atómico/inmutable publicado periódicamente por `hid_worker`, garantizando latencia $< 2\text{ ms}$ sin bloquearse en timeouts de hardware.

### 4.2 Desacoplamiento de Entrada y Fail-Safe contra Teclas Pegadas
1. **Canal de Entrada Acotado:** El hilo principal crea un canal `sync_channel(64)` al arrancar. El `uinput_worker` posee el `Receiver` y permanece vivo durante toda la ejecución del daemon.
2. **`hid_worker` Efímero:** En cada conexión, el `hid_worker` recibe un clon del `SyncSender`.
3. **Generación / Época (`divert_epoch`):** Cada transición a `ACTIVE` incrementa monótonamente `divert_epoch`. Cada evento emitido por `hid_worker` viaja con su época; si el despachador detecta un evento de una época anterior, lo descarta.
4. **Liberación Automática (Fail-Safe Release-All):** El `uinput_worker` mantiene un conjunto `pressed_keys: HashSet<evdev::Key>`. Al recibir la notificación de desconexión o al salir de `ACTIVE`, el worker emite automáticamente `Key Up` para todas las teclas registradas en `pressed_keys` y vacía el conjunto, erradicando el riesgo de teclas retenidas en el sistema.

### 4.3 Máquina de Estados Completa y Tabla de Transiciones

```
                         ┌────────────────────────────────────────────────────────┐
                         │                                                        │
                         ▼                                                        │
                 ┌──────────────┐   Hardware detectado   ┌──────────────┐         │ Desconexión /
                 │ DISCONNECTED │ ─────────────────────> │  CONNECTING  │         │ Suspensión
                 └──────────────┘                        └──────────────┘         │ (POLLHUP/ERR)
                         ▲                                      │                 │
                         │ Error / Timeout                      │ Handshake OK    │
                         │                                      ▼                 │
                         │                               ┌──────────────┐         │
                         │             Error             │ RECONCILING  │         │
                         ├────────────────────────────── │ (reset_divert│         │
                         │                               └──────────────┘         │
                         │                                      │                 │
                         │                                      │ Reconciliación OK
                         │                                      ▼                 │
                         │                               ┌──────────────┐         │
                         │                               │    ACTIVE    │ ────────┘
                         │                               └──────────────┘
                         │                                      │
                         │                                      │ SIGTERM / SIGINT
                         │                                      ▼ (desde CUALQUIER estado)
                         │                               ┌──────────────┐
                         └────────────────────────────── │ SHUTTING_DOWN│
                                                         └──────────────┘
```

#### Transiciones Exhaustivas:
- **`DISCONNECTED → CONNECTING`:** Nodo `/dev/hidraw*` candidato detectado por escaneo pasivo con backoff (`1 s` a `4 s`).
- **`CONNECTING → RECONCILING`:** `auto_connect()` resuelve el slot (`0xFF` o receptor) exitosamente.
- **`CONNECTING → DISCONNECTED`:** Timeout de conexión superado o error de permisos.
- **`RECONCILING → ACTIVE`:** `reset_all_diverts()` completa exitosamente, limpia cualquier CIDs huérfano y aplica el divert al botón superior (`0x00FD`).
- **`RECONCILING → DISCONNECTED`:** Error o desconexión durante la reconciliación.
- **`ACTIVE → DISCONNECTED`:** Detección de `POLLHUP`, `POLLERR` o `ENODEV`. Se emite **Release-All** de teclas virtuales, se cierra el fd viejo y se pasa a backoff pasivo.
- **`CUALQUIER ESTADO → SHUTTING_DOWN`:** Al recibir `SIGTERM` o `SIGINT`:
  - Si está en `ACTIVE`, desactiva únicamente los CIDs activamente desviados (usando el `feature_index` previamente cacheado para no superar el presupuesto de 3 s).
  - Emite Release-All en `uinput` y destruye el dispositivo virtual.
  - Cierra sockets, elimina `mx-options.sock`, libera `flock` y sale limpiamente en $\le 2\text{ s}$.

### 4.4 Exclusividad de Instancia (Singleton Lock)
- Apertura o creación de `$XDG_RUNTIME_DIR/mx-options.lock`.
- Adquisición de `libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB)`.
- Si ya existe otra instancia (`EWOULDBLOCK` o `EAGAIN`), escribir mensaje claro en stderr y salir con código `1`.
- Si el bloqueo es exitoso, eliminar socket residual previo con `std::fs::remove_file(socket_path)` antes de crear el listener.

---

## 5. Servicio de Usuario systemd (`systemd --user`)

### 5.1 Ubicación y Contenido de la Unidad
Archivo: `systemd/user/mx-options.service`

```ini
[Unit]
Description=MX Options Linux Mouse Control Daemon
Documentation=https://github.com/panxio/mx-options-linux
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/mx-daemon
Restart=on-failure
RestartSec=2s
RestartPreventExitStatus=1
StartLimitIntervalSec=30s
StartLimitBurst=5
KillSignal=SIGTERM
TimeoutStopSec=3s
RuntimeDirectory=mx-options
RuntimeDirectoryMode=0700
StandardOutput=journal
StandardError=journal

# Endurecimiento defensivo
NoNewPrivileges=yes
RestrictAddressFamilies=AF_UNIX

[Install]
WantedBy=default.target
```

---

## 6. Invariantes del Sistema (Pilar 2)

1. **Memoria Acotada:** El consumo de memoria RSS de `mx-daemon` nunca debe exceder los $15\text{ MB}$ en reposo (objetivo nominal $\le 5\text{ MB}$, monitoreado vía `VmRSS` en `/proc/self/status`).
2. **Actor de Dueño Único:** Ningún hilo excepto `hid_worker` interactúa con el descriptor `/dev/hidraw`. Cero carreras de lectura de paquetes o corrupción de tramas HID++.
3. **Zero Stuck Keys (Fail-Safe):** Ante cualquier desconexión física del ratón o apagado del daemon, todas las teclas virtuales sostenidas se liberan de forma instantánea.
4. **Reconciliación Confiable y Rápida:** La reconciliación al arranque y el reset al apagar operan con `feature_index` cacheado y verificación estricta de respuesta, sin exceder `TimeoutStopSec=3s`.
5. **Aislamiento Local:** El socket IPC valida `SO_PEERCRED` en el kernel, rechazando conexiones de otros usuarios locales.

---

## 7. Casos Borde y Contingencias (Pilar 4)

1. **Cliente Malicioso o Incompleto (Slow-Loris):** Lecturas limitadas con `Read::take(64 KB)` y timeout de 2 s por línea evitan bloqueos de descriptores o fugas de memoria.
2. **Corte Abrupto de Cliente (`EPIPE` / `ECONNRESET`):** Escrituras protegidas contra `SIGPIPE` se descartan sin interrumpir la operación del daemon.
3. **Suspensión / Despertar (Suspend to RAM):** Al reanudar, BlueZ puede reasignar un nodo nuevo (`/dev/hidraw4`). El escaneo resuelve el hardware por `VID:PID` y re-aplica el divert automáticamente.
4. **Permisos Transitorios tras Resume:** Si `/dev/hidraw` reaparece antes de que udev aplique el tag `uaccess` (`PermissionDenied`), se trata como transitorio con reintento, evitando crash-loops.
5. **Convivencia con `mx-cli`:** `mx-cli` detecta si el daemon está corriendo mediante comprobación del lock/socket IPC y canaliza sus solicitudes vía IPC en lugar de contender por el nodo de hardware.

---

## 8. Arnés de Verificación y Pruebas (Pilar 5)

1. **Pruebas Unitarias de Protocolo IPC y Framing:**
   - Serialización y deserialización de todas las variantes de `IpcRequest` e `IpcResponse` (incluyendo `get_health`).
   - Test de framing adversarial: envío fragmentado byte a byte, dos JSONs en una sola trama, y mensaje excediendo 64 KB rechazado sin panic.
2. **Pruebas de Máquina de Estados:**
   - Tabla de transiciones completa ejecutada contra `MockTransport`.
   - Test de inyección de desconexión mientras una tecla está presionada: verificación de emisión inmediata de `Key Up`.
3. **Pruebas de Concurrencia y Canales:**
   - Verificación de que `hid_worker` responde a comandos del hilo IPC sin interferir con la recepción de eventos de notificación.
4. **Pruebas de Exclusión Mutua (`flock`):**
   - Intento simultáneo de dos procesos sobre `mx-options.lock` retornando `Err` en el segundo intento.

---

## 9. Árbitros Objetivos (Criterios de Cierre de F3 - Pilar 7)

1. **Compilación y Linters:**
   - `cargo test --workspace` pasando al 100%.
   - `cargo clippy --workspace -- -D warnings` con 0 advertencias.
2. **Medición de Memoria Cuantitativa:**
   - Lectura de `VmRSS` en `/proc/<pid>/status` sobre `mx-daemon` en reposo demostrando $\text{RSS} < 15\text{ MB}$ (tolerancia $\Delta\text{RSS} < 1\text{ MB}$ tras 1000 peticiones IPC).
3. **Medición de Latencia Interna:**
   - Tiempo monotónico entre recepción de notificación HID++ y emisión de evento uinput con $p95 < 2\text{ ms}$.
4. **Resiliencia de Reconexión:**
   - 20/20 reconexiones simuladas o físicas exitosas en $< 5\text{ s}$ sin teclas pegadas ni fugas de file descriptors.
5. **Presupuesto de Parada:**
   - `SIGTERM` procesado con desvío liberado y proceso terminado en $\le 2\text{ s}$.
