# MX Pilot (mxpilot)

<div align="center">
  <h3>Suite nativa, ultraliviana y ergonómica de configuración y desvío de botones para ratones Logitech MX en Linux</h3>

  [![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](LICENSE)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland%20%2F%20X11)-orange.svg)](https://github.com/Panxio/mxpilot)
  [![Rust](https://img.shields.io/badge/Backend-Rust%201.80+-red.svg)](https://www.rust-lang.org/)
  [![Tauri](https://img.shields.io/badge/GUI-Tauri%20v2%20%2B%20Svelte%205-purple.svg)](https://tauri.app/)
  [![Package](https://img.shields.io/badge/Package-.deb%20(4.15%20MB)-emerald.svg)](https://github.com/Panxio/mxpilot/releases)
</div>

---

**MX Pilot** es una solución open-source diseñada para cubrir el vacío que deja Logitech al no ofrecer software oficial (*Logitech Options / Options+*) para Linux. Desarrollado en **Rust**, **Tauri v2** y **Svelte 5**, permite reprogramar los botones del ratón **Logitech MX Vertical** (y dispositivos de la familia MX) mediante el protocolo nativo **HID++ 2.0/4.x** e inyección de eventos por **/dev/uinput**, sin requerir privilegios de `root` ni colgar el sistema.

---

## ✨ Características Principales

* 🎯 **Button Diversion Completo (HID++ 2.0/4.x):**
  Desvía eventos del botón superior de DPI (`CID 0x00FD`), botones laterales de pulgar (`0x0056` Adelante, `0x0053` Atrás) y clic de rueda central (`0x0052`), permitiendo asignarles cualquier acción o devolverlos a su comportamiento de fábrica (`SystemDefault`) en caliente.
* ⌨️ **Emulación de Entrada por `/dev/uinput` a nivel de Kernel:**
  Totalmente compatible con **Wayland (GNOME, KDE Plasma, Sway, Hyprland)** y **X11**. Emula pulsaciones reales (`Super/Meta`, `Ctrl+C`, `Ctrl+V`, `Silenciar`, `Play/Pausa`) y navegación de escritorios virtuales (`Ctrl+Alt+→`, `Ctrl+Alt+←`).
* 🛡️ **Garantía Zero Dead Buttons & Zero Stuck Keys:**
  Arquitectura con limpieza RAII y manejo de señales POSIX. Si el daemon se detiene o se desinstala el paquete, el hardware se restaura automáticamente a modo fábrica antes de salir, impidiendo botones muertos en el ratón.
* 🪶 **Consumo Mínimo de Recursos:**
  El daemon en segundo plano (`mx-daemon`) consume apenas **~2 a 3 MB de memoria RAM** en reposo (a diferencia de clientes basados en Electron que consumen cientos de megabytes).
* 🎨 **Interfaz Gráfica Moderna (Tauri v2 + Svelte 5):**
  Blueprint vectorial interactivo fotorrealista del MX Vertical con iluminación reactiva, menú contextual en la bandeja del sistema (System Tray con porcentaje de batería) y layout optimizado con anclaje superior (*Top-Pinning*).
* 🔒 **Seguridad y Ejecución sin `sudo`:**
  Reglas `udev` modernas con `TAG+="uaccess"` y comunicación IPC por Unix Domain Socket (`0600`) verificado con `SO_PEERCRED`. Sandboxing estricto de usuario.

---

## 📦 Herramientas Incluidas

La suite empaqueta 5 utilidades complementarias:

| Binario | Rol / Propósito |
|---|---|
| `mx-daemon` | Servicio en segundo plano para arbitraje HID++, uinput y servidor de socket IPC. |
| `mx-gui` | Interfaz gráfica de usuario con System Tray y blueprint interactivo del mouse. |
| `mx-cli` | Herramienta de línea de comandos para control interactivo y consultas de estado. |
| `mx-diag` | Utilidad de telemetría, inspección de salud, diagnóstico de memoria y convivencia. |
| `mx-probe` | Sonda de diagnóstico directo para volcado de características e inspección de reportes HID++. |

---

## 🚀 Instalación

### En Debian / Ubuntu / Linux Mint / Pop!_OS (`.deb`)

Descarga el último instalador desde la pestaña [**Releases**](https://github.com/Panxio/mxpilot/releases):

```bash
# Instalar paquete deb
sudo apt install ./mx-options_0.1.0-1_amd64.deb
```

El instalador:
1. Registra las reglas `udev` en `/usr/lib/udev/rules.d/70-mx-options.rules` y las recarga en caliente.
2. Registra el servicio de usuario en `/usr/lib/systemd/user/mx-options.service`.
3. Añade el lanzador al menú de aplicaciones (`/usr/share/applications/mx-options.desktop`) con tema de iconos Hicolor.

Para iniciar la interfaz gráfica, búscala como **MX Options** en el menú de aplicaciones de tu escritorio o ejecuta:

```bash
mx-gui
```

---

## ⚙️ Compilación desde el Código Fuente

### Requisitos previos (Debian/Ubuntu):
```bash
sudo apt update
sudo apt install -y build-essential libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libudev-dev libevdev-dev dpkg-dev
```

### Compilar y probar localmente:
```bash
# 1. Clonar el repositorio
git clone https://github.com/Panxio/mxpilot.git
cd mxpilot

# 2. Compilar frontend Svelte
npm --prefix crates/mx-gui/ui install
npm --prefix crates/mx-gui/ui run build

# 3. Compilar workspace en modo release
cargo build --release --workspace

# 4. Ejecutar tests unitarios
cargo test --workspace
```

---

## 📄 Licencia

Este proyecto está distribuido bajo la licencia **GNU General Public License v2.0 (GPL-2.0)**. Consulta el archivo [LICENSE](LICENSE) para más detalles.

*Aviso: Este proyecto es un desarrollo independiente de código abierto y no está afiliado, respaldado ni respaldado oficialmente por Logitech.*
