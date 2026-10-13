# Brainstorming final — MX Options para Linux (Rust + Tauri)

Síntesis de: documento base (Gemini 3.8 Flash) + 7 subagentes OpenCode Go (deepseek-v4.1-flash, claude-haiku-5-5, mimo-v2.6-flash, muse-spark-1.3, qwen3.8-flash, glm-5.3-flash, gpt-6-luna). Fecha: 2026-10-13.

## 1. Evaluación del documento base
**Bueno (se mantiene):** diagnóstico HID++ y *button diversion*; `uinput` para Wayland y X11; no hace falta desensamblar el `.exe` para el protocolo (logiops, Solaar y el driver del kernel bastan); CLI antes que GUI; stack Rust + Tauri; MVP centrado en el MX Vertical.

**Débil (se corrige):** deja abiertas decisiones que ya se pueden cerrar; no menciona Bluetooth, suspensión, hotplug ni convivencia con Solaar/logiops; sin criterios de aceptación medibles; el plan pone la GUI antes de validar lo difícil; afirma "100 % compatible" y CIDs concretos sin verificarlos por firmware.

**Veredicto:** buen punto de partida (~60 %). El 7/7 de los subagentes coincidió en las mismas lagunas, lo que da confianza en ellas.

## 2. Decisiones cerradas

| Tema | Decisión | Origen |
|---|---|---|
| Arquitectura | **Daemon `systemd --user` + GUI Tauri como cliente.** Core en un crate independiente (`mx-core`) | deepseek, haiku, muse; mimo propone empezar monolito, se descarta porque el daemon se reescribiría |
| Eventos | `uinput` única vía. Interfaz de acciones abstracta (`Key`, `Mouse`, `DBus/MPRIS`) pero solo `Key`/`Mouse` en el MVP | todos |
| Permisos | udev con `TAG+="uaccess"` (hidraw 046d + uinput). Nunca `sudo`. AppImage solo con `--install-udev` | deepseek (mimo sugiere además grupo; no hace falta) |
| Frontend | **Svelte + Tailwind**, SPA estática | deepseek, mimo |
| Empaquetado | **`.deb` primario**; `.rpm` después; AppImage/Flatpak secundarios (tray y udev problemáticos) | deepseek, mimo, luna |
| IPC | Unix socket 0600 en `$XDG_RUNTIME_DIR`, `SO_PEERCRED`, **sin `Exec` arbitrario en el MVP** | deepseek, mimo, luna |
| Licencia | GPL-2.0 (conviven con kernel/logiops); decidir antes de codificar | muse |
| REA / `.exe` | Fuera del camino crítico; arte propio en lugar de assets de Logitech | deepseek, qwen |

## 3. Riesgos añadidos (checklist)
1. **Bluetooth ≠ Unifying/Bolt**: handshake e índices pueden diferir. Validar el día 1 (kill-gate).
2. **Suspensión/hotplug**: `hidraw` se reenumera y el divert se pierde → watcher udev/netlink y re-aplicar estado.
3. **Conflicto con Solaar/logiops/`hid-logitech-hidpp`**: detectar y avisar; no pelear por el divert.
4. **uinput**: crear un nodo con nombre claro y cerrarlo en `Drop`; evitar dispositivos fantasma.
5. **DPI**: no prometer control por software hasta verificar que el MX Vertical expone la feature (0x2201 puede no aplicar).
6. **Estado atascado**: si el daemon muere, el divert no debe dejar botones muertos → restaurar al salir.
7. **Diagnóstico**: `RUST_LOG` + `mx-diag --dump` desde el día 1 (sin esto el soporte BT es ciego).
8. Sesiones headless/SSH: el daemon `--user` debe fallar con gracia.

## 4. Plan por fases con criterios de aceptación
Reordenado: lo difícil primero, GUI al final. Estimaciones solo indicativas (glm/mimo); calibrar tras F0.

| Fase | Contenido | Aceptación medible | Kill-gate |
|---|---|---|---|
| **F0** Protocolo (`mx-probe`) | hidapi/hidraw, handshake HID++ 2.0, dump de features | Dump de features 0x0000/0x0001, 0x1000, 0x1B04 correcto en **USB, receptor y BT**; batería ±10 % vs Solaar en 10 muestras; tests con mock HID en CI | Si el divert del botón superior no funciona en BT ni receptor → replantear |
| **F1** Divert + uinput (CLI) | desviar CID, emitir tecla | 1 evento por pulsación (sin duplicados ni pérdidas) en `evtest`; ≥99 % en 10 min; GNOME Wayland **y** X11 | — |
| **F2** Permisos | reglas udev, instalación sin root | Usuario no-root funciona tras logout/login; sin reglas huérfanas tras purge | — |
| **F3** Daemon + IPC | systemd user, socket, reaplicación de estado | Remapeo activo con GUI cerrada; sobrevive suspensión + 20 reconexiones BT; reconexión < 5 s; p95 botón→evento < 20 ms; daemon < 15 MB RAM | — |
| **F4** GUI | batería, remapeo, DPI (si se verificó), tray | RAM daemon+GUI < 80 MB; bundle < 15 MB; onboarding ≤ 3 pasos con detección de permisos | — |
| **F5** Empaquetado y convivencia | `.deb`, `.rpm`, detección de Solaar/logid | Instalación y desinstalación limpias en Ubuntu LTS y Fedora; no duplica divert si detecta otro daemon | — |

**Fuera del MVP:** gestos, perfiles por ventana, otros dispositivos (MX Master, MX Keys), MPRIS, `Exec`. Abstraer `DeviceProfile` desde el inicio para no reescribir luego.

**Presupuesto realista (qwen):** ~40 % del esfuerzo está en F1–F3 (robustez BT y ciclo de vida del daemon); la GUI es la parte fácil.

## 5. Reglas de trabajo para la sesión nueva
- Carpeta nueva del proyecto; este documento se copia como `docs/BRAINSTORMING.md`.
- Escribir primero el árbitro de cada fase (comando que demuestra la aceptación) y recién después implementar.
- Hardware real necesario desde F0: MX Vertical por USB/receptor **y** BT.
- Si delegas a modelos flash, hazlo por unidad con árbitro objetivo; revisar siempre el código, no el autoinforme (glm y gemini marcaron como verificado lo falso en pruebas anteriores).
- Vigila el contexto: avisos del hook en 160k/200k; abre sesión nueva entre fases.

## 6. Qué no se pudo comprobar
- Los CIDs `0x00C3`/`0x00FD` y la feature DPI del MX Vertical son hipótesis hasta hacer el dump en F0.
- Las estimaciones de semanas vienen de modelos, sin base medida.
- De glm-5.3-flash y gpt-6-luna usé solo el esqueleto de sus respuestas; sus textos completos están en `subagents/out/`.
