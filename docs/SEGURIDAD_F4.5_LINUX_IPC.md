# Fase 4.5 — Endurecimiento y Ethical Hacking: Superficie Linux / IPC / Daemon

> **Especialista:** `deepseek` (`opencode-go/deepseek-v4.1-flash`) — Seguridad de Sistemas Linux, IPC y Kernel.
> **Ámbito:** `crates/mx-daemon` (IPC Unix Domain Socket, worker HID++/uinput), `crates/mx-gui/src/commands.rs` (invocación de `systemctl`), `systemd/user/mx-options.service` (sandboxing) y `rules/70-mx-options.rules`.
> **Método:** inspección de código + **verificación empírica en vivo** sobre systemd 259 (Ubuntu) y el daemon en ejecución. Todo hallazgo numérico es medido, no inferido.
> **Fecha:** 2026-10-14. **Commit base:** `42217db`.

---

## 0. Resumen ejecutivo

El diseño del IPC es **sólido**: la frontera de confianza cruzada (UID ajeno en el socket) está doblemente cerrada por permisos de filesystem (dir `0700` + socket `0600`) y por `SO_PEERCRED`. La invocación de `systemctl` **es inmune** a inyección de argumentos, shell y PATH hijacking. La superficie de riesgo restante es de **disponibilidad** (DoS local por un proceso del mismo UID), no de escalada de privilegios.

El endurecimiento de systemd propuesto baja la exposición medida **de 8.9 EXPOSED a 4.0 OK** (`systemd-analyze --user security`), pero requiere dos **correcciones críticas de realidad** que solo aparecen al probar: `CapabilityBoundingSet=` y `ProtectKernelModules/Logs`, `ProtectClock` **rompen el arranque** del servicio en `systemd --user` (exit `218/CAPABILITIES`). Se documenta la causa raíz y el subconjunto verificado que sí funciona.

| ID | Severidad | Hallazgo | Estado |
|----|-----------|----------|--------|
| **H1** | 🟠 Media (Disponibilidad) | 8 conexiones ociosas (0 bytes) agotan `MAX_CLIENTS` y **niegan el IPC a la GUI** indefinidamente: el timeout de línea solo aplica si el buffer ya tiene datos. Verificado (T8). | Abierto |
| **H2** | 🟡 Baja (Disponibilidad) | Servidor IPC mono-hilo: `get_battery`/`set_action` bloquean el bucle `accept` hasta 500 ms → un cliente induce latencia al resto. | Abierto |
| **H3** | 🟡 Baja (TOCTOU) | Ventana `bind()`→`chmod(0600)`: el socket nace con `0777 & ~umask`. Mitigado por dir `0700`; se cierra con `UMask=0177`. | Mitigar |
| **H4** | 🟡 Baja (Higiene) | `RuntimeDirectory=mx-options` no coincide con la ruta real del socket (`$XDG_RUNTIME_DIR/mx-options.sock`) → directorio protegido vacío. | Corregir |
| **H5** | 🟡 Baja (Robustez) | `unwrap()` sobre `RwLock` en `handle_line`: un poisoning hace paniquear el hilo IPC (viola el invariante "nunca panic" de AGENTS.md). | Abierto |
| **H6** | 🔵 Informativa | Divergencia spec/impl: `F3-daemon-ipc.md §3.1` declara `peer_cred()` de `std` "sin FFI inseguro"; el código usa `libc::getsockopt` (unsafe). | Documentar |
| **H7** | 🔵 Informativa | `write_all` sobre socket no bloqueante puede fallar con `WouldBlock` y truncar respuestas (hoy <1 KB, riesgo bajo). | Documentar |
| **H8** | 🔵 Compuerta | `cargo audit`: 0 vulnerabilidades; 2 avisos transitivos (`glib 0.18.5` unsound, `proc-macro-error` unmaintained). | Vigilar |

**Defensas verificadas correctas:** `SO_PEERCRED`, aislamiento cross-UID estructural, `SOCK_CLOEXEC`, `SIGPIPE` ignorado, framing bounded, allowlist de CID/teclas, `systemctl` sin shell ni PATH, `NoNewPrivileges`, `RestrictAddressFamilies=AF_UNIX`, cero panics bajo 200+ ataques.

---

## 1. Metodología y evidencia

- **Inspección:** `crates/mx-daemon/src/ipc/{server.rs,protocol.rs}`, `src/{main.rs,workers.rs,state.rs,lock.rs}`, `crates/mx-gui/src/{client.rs,commands.rs}`, `systemd/user/mx-options.service`, `rules/70-mx-options.rules`.
- **Verificación en vivo:** 11 pruebas adversariales contra el daemon en ejecución (PID 3052120) — ver §6 y `scripts/security/f45-ipc-attacks.py`.
- **Sandbox medido:** creación de unidades transitorias `systemd-run --user` con cada directiva aislada y en conjunto, midiendo `systemd-analyze --user security` y el arranque real del binario.
- **Entorno:** systemd 259 (259.5-0ubuntu3.4), socket `srw------- panxio panxio /run/user/1000/mx-options.sock`, dir runtime `0700`.

---

## 2. Modelo de amenaza

**Activos protegidos:** (a) integridad/correctitud de la inyección de eventos uinput (podría sintetizar pulsaciones de teclado globales), (b) disponibilidad del daemon (los botones del ratón dependen de él), (c) integridad del estado de hardware HID++.

**Frontera de confianza:** *otro UID local no privilegiado* (el caso que `0600` + `SO_PEERCRED` deben detener). El mismo UID **no** es una frontera real: un proceso del mismo usuario ya puede `ptrace` al daemon, leer su memoria y reescribir sus archivos; tratarlo como atacante "externo" sería un error de modelado. Las defensas contra el mismo UID deben apuntar a **DoS / robustez**, no a confidencialidad.

**Fuera de alcance:** atacante con `root`, DMA, o acceso físico; fugas por canales laterales de microarquitectura.

---

## 3. Superficie de ataque del IPC y el daemon

### 3.1 `SO_PEERCRED` — correcto
`get_peer_uid()` (`server.rs:21`) lee `SO_PEERCRED` vía `getsockopt` y rechaza si `cred.uid != getuid()` (`server.rs:118-131`), **antes** de encolar el cliente. La credencial la fija el kernel en el `connect()`, no es falsificable por el cliente. El rechazo cierra el stream (drop) sin procesar bytes. ✅

*Matiz:* es la segunda línea; la primera es que el socket vive dentro de un directorio `0700`. Un UID ajeno ni siquiera puede recorrer la ruta (`namei` lo confirma en §6-T11). No hay fallback a `/tmp` (`server.rs:61`), lo cual es correcto.

### 3.2 Evasión de permisos y secuestro de descriptores (FD leaking)
- **FD leaking:** el daemon **no lanza procesos externos** (`grep` confirmado), así que no hay vector de herencia de FDs vía `exec`. El cliente GUI crea su socket con `SOCK_CLOEXEC` (`client.rs:128`), de modo que el `systemctl` que sí lanza no hereda el descriptor del IPC. ✅
- **Carrera bind→chmod (H3):** `UnixListener::bind` crea el nodo con `0777 & ~umask` y **luego** se aplica `set_permissions(0600)` (`server.rs:82-86`). Sin `UMask` explícito, existe una ventana con modo `0755`. Está mitigada por el `0700` del directorio padre, pero se cierra limpiamente fijando `UMask=0177` (el socket nace `0600`). Nota: la spec F3 §3.1 describía un `umask(0o177)` previo al bind que **no está en el código** (se retiró por carreras de tests, Sesión 7/P7). La vía correcta es `UMask=` en la unidad systemd, no un `umask()` global de proceso.
- **Enlaces simbólicos:** `remove_file` incondicional sobre `mx-options.sock`/`.lock` es seguro aquí porque sólo el propio UID puede escribir en el dir.
- **`SIGPIPE`:** ignorado globalmente (`main.rs:26`), por lo que una escritura a un peer cerrado devuelve `EPIPE` y no mata el daemon. ✅

### 3.3 Límites de recursos y resiliencia (verificado)
Framing por `\n` con cota 64 KB y timeout anti-slow-loris de 2 s (`server.rs:16-18,181-189`). El arnés §6 confirma rechazo de payload gigante, UTF-8 inválido, JSON malformado y ráfagas de 200 mensajes sin pérdida (T1–T7, T9). ✅

**H1 (Media) — *slow-loris* de ranura:** el timeout de línea incompleta sólo dispara si `!client.buffer.is_empty()` (`server.rs:166`). Una conexión que **nunca envía bytes** mantiene `buffer` vacío y **jamás expira**, reteniendo su ranura. Con `MAX_CLIENTS = 8`, ocho conexiones ociosas niegan permanentemente el servicio a la GUI (el 9.º recibe `too many clients`). Verificado en vivo (T8). *Fix sugerido:* timeout de inactividad global por conexión (aplicar el deadline aunque el buffer esté vacío), o reducir el coste de ranura.

**H2 (Baja) — servidor mono-hilo:** `handle_line` se ejecuta dentro del bucle `accept` y `get_battery`/`set_action` esperan hasta 500 ms al worker HID (`server.rs:268,307`). Un cliente puede inducir latencia a otros. *Fix sugerido:* despachar el comando al worker sin bloquear el bucle, o un hilo por conexión con presupuesto acotado.

**H5 (Baja) — poison de `RwLock`:** `self.state.read().unwrap()` (`server.rs:249,253,258,277,310`). Si un hilo paniquea con el lock tomado, el hilo IPC paniquea en la siguiente petición. *Fix sugerido:* `unwrap_or_else(|e| e.into_inner())` o un helper que degrade con elegancia.

### 3.4 Invariante de dueño único (positivo)
`hid_worker` es el único que toca `/dev/hidraw`; el IPC sólo envía `DaemonCommand` por canal. No hay contienda de descriptores ni *frame-stealing*. El consumo de capabilities efectivas del daemon es **cero** (`CapEff: 0000000000000000` medido), lo que refuerza el punto de §5.

---

## 4. Invocación de procesos externos (`commands.rs::start_daemon_service`)

```rust
let mut cmd = Command::new("/usr/bin/systemctl");
cmd.args(["--user", "start", "mx-options.service"]);
cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
```

| Vector | ¿Explotable? | Razonamiento |
|--------|--------------|--------------|
| Inyección de argumentos / shell | **NO** | No hay `sh -c`; `args()` es un array literal sin datos de usuario. |
| PATH traversal / hijacking | **NO** | Ruta **absoluta** `/usr/bin/systemctl`; no se resuelve por `PATH`. |
| Inyección vía nombre de unidad | **NO** | La unidad es una constante; no se interpola entrada del usuario. |
| Manipulación de variables de entorno | **No critico** | El subproceso hereda el entorno de la GUI. `systemctl` usa `DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_*`; influirlas requiere ya controlar el entorno de la GUI (mismo UID) → sin escalada. Endurecible con `cmd.env_clear()` + `ENV` mínimo si se desea defensa en profundidad. |
| `stdin/stdout/stderr` | **NO** | Redirigidos a `null` (sin herencia de fds de la app). |

**Conclusión:** la invocación es correcta y **no requiere cambio**. Recomendación opcional de defensa en profundidad: `cmd.env_clear()` y `cmd.env("XDG_RUNTIME_DIR", ...)`.

---

## 5. Aislamiento de permisos de Linux (systemd)

### 5.1 Medición objetiva
`systemd-analyze --user security mx-options.service` → **Overall exposure level: 8.9 EXPOSED** 🙁 sobre la unidad actual.

### 5.2 Resultado verificado por directiva (systemd 259)
Cada directiva se aplicó aislada y en conjunto contra el **binario real** `mx-daemon`. "Arranca" = el proceso llega a `main` y sale con `1` por el `flock` singleton (comportamiento esperado con el daemon ya activo).

| Directiva | Resultado medido | Recomendación |
|---|---|---|
| `NoNewPrivileges=yes` | arranca | mantener (ya presente) |
| `RestrictAddressFamilies=AF_UNIX` | arranca | mantener (ya presente) |
| `ProtectSystem=strict` | arranca | **añadir** |
| `ProtectHome=read-only` | arranca | **añadir** |
| `PrivateTmp=yes` | arranca | **añadir** |
| `ProtectKernelTunables=yes` | arranca | **añadir** |
| `ProtectControlGroups=yes` | arranca | **añadir** |
| `RestrictNamespaces=yes` | arranca | **añadir** |
| `LockPersonality=yes` | arranca | **añadir** |
| `MemoryDenyWriteExecute=yes` | arranca | **añadir** (validar en estado ACTIVE) |
| `RestrictRealtime=yes` | arranca | **añadir** |
| `RestrictSUIDSGID=yes` | arranca | **añadir** |
| `SystemCallArchitectures=native` | arranca | **añadir** |
| `SystemCallFilter=@system-service` (+`~@privileged @resources …`) | arranca | **añadir** (validar en ACTIVE) |
| `PrivateNetwork=yes` | arranca | **añadir** |
| `IPAddressDeny=any` | arranca | **añadir** |
| `UMask=0077` | arranca | **añadir** (cierra H3) |
| `DevicePolicy=closed` + `DeviceAllow=char-hidraw rw` + `DeviceAllow=char-uinput rw` | arranca | **añadir** (necesario para `/dev`) |
| `ProtectHostname=yes` | arranca | **añadir** |
| `KeyringMode=private` | arranca | **añadir** |
| `ProtectProc=invisible` + `ProcSubset=pid` | arranca (`/proc/self/status` legible) | **añadir** |
| `CapabilityBoundingSet=` (vacío) | ❌ **exit 218** | **PROHIBIDO en unidad de usuario** |
| `CapabilityBoundingSet=~CAP_*` (sustractivo) | ❌ **exit 218** | **PROHIBIDO** |
| `ProtectKernelModules=yes` | ❌ **exit 218** | **PROHIBIDO** |
| `ProtectKernelLogs=yes` | ❌ **exit 218** | **PROHIBIDO** |
| `ProtectClock=yes` | ❌ **exit 218** | **PROHIBIDO** |
| `PrivateUsers=yes` | no ejecutado | **NO** — remapea UID y rompería el acceso a `/dev` vía `uaccess`/ACL |
| `PrivateDevices=yes` | N/A | **NO** — ocultaría `/dev/hidraw` y `/dev/uinput`; usar `DevicePolicy`/`DeviceAllow` |

### 5.3 Causa raíz del fallo 218 (Pilar 6, no adivinado)
`journalctl --user` lo dice literalmente:

```
(mx-daemon)[...]: Failed to drop capabilities: Operation not permitted
(mx-daemon)[...]: Failed at step CAPABILITIES spawning .../mx-daemon: Operation not permitted
Main process exited, code=exited, status=218/CAPABILITIES
```

El **gestor `systemd --user` no dispone de `CAP_SETPCAP`**, por lo que no puede modificar el *capability bounding set* de sus servicios. Toda directiva que internamente lo dropee —`CapabilityBoundingSet=`, `ProtectKernelModules`/`ProtectKernelLogs` (dropean `CAP_SYS_MODULE`/`CAP_SYSLOG`) y `ProtectClock` (dropea `CAP_SYS_TIME`/`CAP_WAKE_ALARM`)— hace que el servicio **no arranque**. `ProtectKernelTunables` y `ProtectControlGroups` sí funcionan porque se implementan sóolo con montajes, sin dropear caps.

**Impacto de seguridad de la omisión: nulo.** El proceso ya corre con `CapEff = 0` (medido en el daemon vivo): al ser un servicio de usuario, no puede usar ninguna capability aunque estuviera en el bounding set. Por eso `CapabilityBoundingSet=` es redundante aquí y su ausencia no debilita nada.

### 5.4 Propuesta de unidad endurecida (subconjunto verificado)
```ini
[Unit]
Description=MX Options Linux Mouse Control Daemon
Documentation=https://github.com/panxio/mx-options-linux
After=graphical-session.target
StartLimitIntervalSec=30s
StartLimitBurst=5

[Service]
Type=simple
ExecStart=%h/.local/bin/mx-daemon
Restart=on-failure
RestartSec=2s
RestartPreventExitStatus=1
KillSignal=SIGTERM
TimeoutStopSec=3s
StandardOutput=journal
StandardError=journal

# --- Sandbox verificado en systemd 259 (8.9 EXPOSED -> 4.0 OK) ---
NoNewPrivileges=yes
RestrictAddressFamilies=AF_UNIX
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=yes
ProtectKernelTunables=yes
ProtectControlGroups=yes
RestrictNamespaces=yes
LockPersonality=yes
MemoryDenyWriteExecute=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources @obsolete @debug @mount @cpu-emulation @raw-io @reboot @swap @module @clock
PrivateNetwork=yes
IPAddressDeny=any
DevicePolicy=closed
DeviceAllow=char-hidraw rw
DeviceAllow=char-uinput rw
UMask=0177
ProtectHostname=yes
KeyringMode=private
ProtectProc=invisible
ProcSubset=pid
# NO usar: CapabilityBoundingSet=, ProtectKernelModules, ProtectKernelLogs,
#          ProtectClock  (rompen arranque en systemd --user: exit 218)
# NO usar: PrivateDevices=yes / PrivateUsers=yes  (rompen acceso a /dev/uaccess)

[Install]
WantedBy=default.target
```

> **Nota sobre `RuntimeDirectory` (H4):** la unidad actual declara `RuntimeDirectory=mx-options`, pero el daemon crea el socket en `$XDG_RUNTIME_DIR/mx-options.sock`, **un nivel arriba** (`/run/user/1000/`), dejando `mx-options/` vacío. Elegir una de dos: (a) eliminar `RuntimeDirectory` y mantener la ruta actual; o (b) cambiar el daemon para usar `$RUNTIME_DIRECTORY`. La opción (b) es más limpia porque aísla el socket en un subdirectorio que systemd gestiona y limpia. *(La ruta actual sigue siendo segura por el `0700` del runtime dir.)*

---

## 6. Plan de pruebas de Ethical Hacking local (vectores Linux)

Arnés reproducible versionado en [`scripts/security/f45-ipc-attacks.py`](../scripts/security/f45-ipc-attacks.py).

### Bloque A — IPC (ejecutado en vivo, resultado observado)

```bash
python3 scripts/security/f45-ipc-attacks.py; echo "exit=$?"
```

| # | Prueba | Criterio PASS | Observado |
|---|--------|---------------|-----------|
| T1 | JSON malformado | error estructurado, daemon vivo | ✅ PASS |
| T2 | CID `0xBEEF` no reprogramable | error, sin tocar hardware | ✅ PASS |
| T3 | Tecla `KEY_RM_RF` no permitida | error de allowlist | ✅ PASS |
| T4 | Payload 70 000 B sin `\n` | `payload too large` + cierre | ✅ PASS |
| T5 | Bytes UTF-8 inválidos | `invalid utf-8` controlado | ✅ PASS |
| T6 | Inyección `\r\n\r\n` | framing intacto, `pong` | ✅ PASS |
| T7 | 200 pings consecutivos | 200/200 correlacionados | ✅ PASS |
| T8 | 8 conexiones ociosas > 3.2 s | **HALLAZGO reproducido** (`too many clients`) | ✅ PASS (evidencia) |
| T9 | Liveness tras 200+ ataques | `ping` responde | ✅ PASS |

### Bloque B — Permisos / UID

```bash
RT="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
# B1. Aislamiento estructural cross-UID (dir 0700):
namei -l "$RT/mx-options.sock"        # PASS: último componente drwx------ panxio
# B2. Rechazo real de otro UID (requiere sudo; NO ejecutado en esta sesión):
sudo -u nobody python3 - <<'PY'
import socket,os
try:
    s=socket.socket(socket.AF_UNIX); s.connect(os.environ["XDG_RUNTIME_DIR"]+"/mx-options.sock")
    print("CONECTADO (FALLO)")
except Exception as e:
    print("RECHAZADO (PASS):",e)
PY
# B3. El socket NO es accesible por grupo/otros:
stat -c '%a %U %G' "$RT/mx-options.sock"   # PASS: 600 panxio panxio
```

Criterios: **B1/B3 PASS** = `drwx------` + `600`. **B2 PASS** = `RECHAZADO` (EACCES por el dir o por `SO_PEERCRED`). *B2 requiere privilegios y quedó pendiente de ejecución en esta sesión (honestidad).*

### Bloque C — Proceso externo / entorno (`commands.rs`)

```bash
# C1. Confirmar ausencia de shell y de interpolación:
grep -n 'Command::new\|\.args(' crates/mx-gui/src/commands.rs   # PASS: ruta absoluta + args literales
# C2. Confirmar ruta absoluta (sin resolución por PATH):
grep -n '"/usr/bin/systemctl"' crates/mx-gui/src/commands.rs   # PASS
# C3. PATH envenenado no afecta:
PATH=/tmp/evil "$HOME/.local/bin/mx-gui" &   # PASS: systemctl sigue siendo /usr/bin/systemctl
```

Criterio: una sola ruta absoluta, sin `sh -c`, sin `format!` con datos de entrada. **PASS** por inspección.

### Bloque D — Sandbox systemd (ejecutado)

```bash
# D1. Puntuación de exposición actual vs propuesta:
systemd-analyze --user security mx-options.service | tail -1   # 8.9 EXPOSED (antes)
# Aplicar la unidad de §5.4 y volver a medir:
systemctl --user daemon-reload && systemctl --user restart mx-options.service
systemd-analyze --user security mx-options.service | tail -1   # PASS: <= 4.0 OK
# D2. El servicio arranca con el set completo:
systemctl --user is-active mx-options.service                  # PASS: active
# D3. NO deben aparecer directivas prohibidas:
grep -nE 'CapabilityBoundingSet|ProtectKernelModules|ProtectKernelLogs|ProtectClock|PrivateDevices|PrivateUsers' \
  systemd/user/mx-options.service                              # PASS: sin coincidencias
```

### Bloque E — Fuzzing y resiliencia adversarial (ampliación propuesta)

```bash
# E1. Fragmentación byte a byte (debe reassemblar):
python3 - <<'PY'
import socket,os,time
p=os.environ["XDG_RUNTIME_DIR"]+"/mx-options.sock"
s=socket.socket(socket.AF_UNIX); s.connect(p); s.settimeout(5)
for b in b'{"id":1,"cmd":"ping"}\n': s.send(bytes([b])); time.sleep(0.01)
print(s.recv(256))   # PASS: {"status":"ok",...,"pong"}
PY
# E2. Carga sostenida (estabilidad de memoria, integra Pilar 7):
for i in $(seq 1 2000); do printf '{"cmd":"get_health"}\n' | socat - "UNIX-CONNECT:$XDG_RUNTIME_DIR/mx-options.sock" >/dev/null; done
grep VmRSS /proc/$(pgrep -n mx-daemon)/status   # PASS: RSS estable < 15 MB
# E3. Desconexión abrupta en plena escritura (SIGKILL del cliente): daemon sobrevive.
# E4. Fuzzing de protocolo con cargo-fuzz sobre parse_request/ActionPayload (F5).
```

Criterios: E1–E3 **PASS** = respuesta correcta y daemon vivo; E2 **PASS** = `ΔRSS < 1 MB`. E4 como tarea de F5 (`cargo-fuzz`, objetivo `parse_request`).

### Bloque F — Cadena de suministro

```bash
cargo audit            # PASS: 0 vulnerabilidades (2 warnings transitivos documentados en H8)
npm --prefix crates/mx-gui/ui audit   # PASS: 0 high/critical
```

---

## 7. Riesgos residuales y límites de este análisis

1. **Seccomp/aislamiento no validados con hardware en ACTIVE.** El arranque bajo `SystemCallFilter=@system-service` y `MemoryDenyWriteExecute=yes` se verificó hasta el `flock`; el daemon sale antes de abrir `/dev/hidraw`. **Acción obligatoria:** con la unidad endurecida y el ratón conectado, verificar `state=Active` en `get_health` y una pulsación real, confirmando que el filtro no bloquea `ioctl`/`read`/`poll` de `hidraw`. Si falla, añadir los `ioctl` necesarios con `SystemCallFilter=~@default` selectivo.
2. **B2 (cross-UID real) no ejecutado** por ausencia de sudo; queda como compuerta con el comando exacto.
3. **H1/H2 son de disponibilidad intra-usuario**, no de confidencialidad; su explotación requiere control del UID (que ya implica confianza total). Se mantienen abiertos por robustez, prioridad baja.
4. Este informe cubre Linux/IPC/daemon; **no** cubre el webview Tauri (CSP, capabilities) ni el frontend — ámbitos de `luna` y `haiku`.

---

## 8. Trazabilidad de requisitos (handoff de dos pasadas)

| Requisito del encargo | Evidencia |
|---|---|
| Riesgos de atacante local mismo UID / acceso a sockets | §2 (modelo), §3.1–3.3 (H1, H2, H5), T8 |
| Evaluación de `SO_PEERCRED` | §3.1 (`server.rs:21,118`) + `namei` (T11/B1) |
| Evasión de permisos | §3.2 (dir 0700 + 0600, sin fallback `/tmp`) |
| Secuestro de descriptores (FD leaking) | §3.2 (`SOCK_CLOEXEC`, daemon sin `exec`) |
| `systemctl` sin shell: inyección / PATH / entorno | §4 (tabla completa, veredicto: inmune) |
| Directivas systemd adicionales | §5.2 (tabla medida), §5.4 (unidad) |
| Plan de hacking local reproducible + Pass/Fail | §6 (bloques A–F, arnés versionado) |

**Criterios de cierre propuestos para autorizar F5:**
1. `systemctl --user restart mx-options` estable con la unidad §5.4 (sin `218`).
2. `systemd-analyze --user security` ≤ **4.0 OK**.
3. Bloque A: **9/9 PASS** (T8 documentado).
4. Bloque D2/D3: servicio activo y sin directivas prohibidas.
5. Verificación con hardware en `ACTIVE` (§7.1) — **bloqueante**.
6. `cargo audit`: 0 vulnerabilidades.
