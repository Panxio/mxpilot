[Español](README.md)

# MX Pilot

A native, lightweight, and ergonomic configuration suite for Logitech MX mice on Linux.

**MX Pilot** (`mx-options`) replaces *Logitech Options / Options+* on Linux by communicating directly over the **HID++ 2.0/4.5** protocol and injecting input events at the kernel level via **`/dev/uinput`**, running entirely in user space without `root` privileges.

## Architecture and Performance

Unlike Electron-based utilities, MX Pilot pairs a standalone Rust background daemon (`mx-daemon`) with a lightweight Tauri v2 + Svelte 5 desktop interface (`mx-gui`, rendered using the operating system's WebKitGTK engine with no Node.js runtime):

- **Daemon RAM usage (`mx-daemon`):** ~2.3 MB RSS idle
- **Installer package (`.deb` / `.rpm`):** 4.15 MB (includes all 5 binaries, icons, and rules)
- **End-to-end input latency (`hidraw` → `uinput`):** under 5 ms
- **`systemd-analyze security` isolation score:** 4.0 OK

## Features

- **Full Button Diversion (HID++ 2.0/4.5):** remap the top DPI button (`0x00FD`), middle scroll wheel click (`0x0052`), and thumb Forward (`0x0056`) and Back (`0x0053`) buttons on the **Logitech MX Vertical** (over Bluetooth or Unifying/Bolt receiver)
- **Live Native Restoration (*Factory Default*):** restore any button to its native hardware behavior (`BTN_SIDE`, `BTN_EXTRA`, `BTN_MIDDLE`) on the fly without restarting the mouse
- **Universal `/dev/uinput` kernel injection:** works seamlessly across **Wayland** (GNOME, KDE Plasma, Sway, Hyprland) and **X11**, supporting continuous key hold and desktop shortcuts (`Super/Meta`, workspace switching `Ctrl+Alt+→/←`, `Alt+Tab`, `Copy`, `Paste`, `Mute`, `Play/Pause`)
- ***Zero Dead Buttons* & *Zero Stuck Keys* guarantee:** RAII guards and package maintainer scripts automatically restore mouse firmware to native mode when stopping the service or uninstalling the package, ensuring buttons are never left unresponsive
- **Interactive 57° SVG Blueprint:** high-precision vector blueprint of the MX Vertical with reactive button highlighting, category filters, and a live internal telemetry modal
- **System Tray integration:** live battery percentage indicator, charging status, and one-click daemon restart
- **Rootless security:** `udev` rules using `TAG+="uaccess"` paired with a local Unix Domain Socket (`0600`) authenticated via kernel peer credentials (`SO_PEERCRED`)
- **English and Spanish interface:** automatically detects the system language and provides a live `ES | EN` toggle in the top bar

## Included Tools

The package installs five complementary binaries into `/usr/bin`:

| Binary | Role |
|---|---|
| `mx-daemon` | Background user service (`systemd --user`) managing HID++ communication, auto-reconnection, `/dev/uinput`, and the IPC socket |
| `mx-gui` | Desktop graphical interface built with Tauri v2 + Svelte 5 featuring System Tray support and an interactive mouse blueprint |
| `mx-cli` | Command-line utility for querying device status, battery level, and testing button diversion from the terminal |
| `mx-diag` | Diagnostic tool for inspecting `udev` permissions, device coexistence, and daemon health metrics |
| `mx-probe` | Low-level hardware probe for enumerating HID++ 2.0/4.5 features and button CIDs |

## Requirements

MX Pilot uses the web engine and GTK libraries already provided by your Linux distribution instead of bundling its own Chromium runtime. That is why the package weighs only 4.15 MB.

| Distribution | Required Packages | Installed automatically? |
|---|---|---|
| **Debian / Ubuntu / Linux Mint / Pop!_OS** | `libwebkit2gtk-4.1-0`, `libgtk-3-0` (or `libgtk-3-0t64`), `libayatana-appindicator3-1`, `udev`, `systemd`, `procps` | Yes, declared in the `.deb` package and installed automatically by `apt` |
| **Fedora / RHEL / openSUSE** | `webkit2gtk4.1`, `gtk3`, `libayatana-appindicator-gtk3`, `systemd`, `systemd-udev`, `procps-ng` | Yes, declared in the `.rpm` package and installed automatically by `dnf` |

On Ubuntu 24.04 and later, the GTK package is named `libgtk-3-0t64`; the `.deb` package declares dual compatibility (`libgtk-3-0t64 | libgtk-3-0`), so it installs out of the box on both Ubuntu 22.04/Debian 12 and Ubuntu 24.04+.

## Installation

### Debian, Ubuntu, Linux Mint, and derivatives (`.deb`)

Download the `.deb` package from the [**Releases**](https://github.com/Panxio/mxpilot/releases) section. **Use `apt`, not `dpkg`** — `apt` automatically resolves and installs any missing shared libraries:

```bash
sudo apt install ./mx-options_0.1.0-1_amd64.deb
```

### Fedora, RHEL, and openSUSE (`.rpm`)

Download the `.rpm` package from [**Releases**](https://github.com/Panxio/mxpilot/releases) and install it with `dnf`:

```bash
sudo dnf install ./mx-options-0.1.0-1.x86_64.rpm
```

### First Launch

After installing the package, enable the user service for your desktop session and launch the graphical interface from your application menu (**MX Options**) or terminal:

```bash
systemctl --user enable --now mx-options.service
mx-gui
```

### Build from Source

Requires Rust (`1.80+`), Node.js (`20+`), and the development headers for Tauri v2, `libudev`, and `libevdev`.

On Debian/Ubuntu:

```bash
sudo apt update
sudo apt install -y build-essential libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev libudev-dev libevdev-dev dpkg-dev rpm cpio
```

On Fedora:

```bash
sudo dnf install -y gcc gcc-c++ make webkit2gtk4.1-devel gtk3-devel \
  libayatana-appindicator-gtk3-devel librsvg2-devel systemd-devel libevdev-devel rpm-build
```

Build, run the test suites, and generate the `.deb` and `.rpm` packages:

```bash
npm --prefix crates/mx-gui/ui ci
npm --prefix crates/mx-gui/ui test
cargo test --workspace

# Generate distribution packages in target/
./scripts/package-deb.sh
./scripts/package-rpm.sh
```

## License

This project is licensed under the **GNU General Public License v2.0 (GPL-2.0)**. See the [`LICENSE`](LICENSE) file for details.

*Disclaimer: MX Pilot is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Logitech.*
