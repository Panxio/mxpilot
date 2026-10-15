# Especificación Técnica (SDD) — F5: Empaquetado (.deb / .rpm) y Distribución Limpia

> **Módulo:** `packaging/` & `scripts/package-{deb,rpm}.sh` + `.github/workflows/release.yml`  
> **Perfil:** tauri-app (Rust Core ultraliviano, distribución nativa en Debian/Ubuntu y Fedora/RHEL)  
> **Orquestador:** Gemini 3.8 Flash High  
> **Panel Colegiado de Revisión:** `deepseek-v4.1-flash`, `claude-haiku-5-5`, `muse-spark-1.3-contributor`, `gpt-6-luna`  
> **Fecha de Consolidación:** 2026-10-14 (Post-Revisión Colegiada Aprobada)  
> **Árbitro Objetivo:** `dpkg-deb -I`, `dpkg-deb -c`, `scripts/audit-packages.sh`, tamaño de paquete < 15 MB, 0 errores de permisos (`root:root`, `0755`/`0644`), 0 residuos en desinstalación, `dpkg-shlibdeps` exacto.

---

## 1. Propósito y Alcance

Esta especificación define la arquitectura, jerarquía de directorios FHS (Filesystem Hierarchy Standard), metadatos de control, dependencias de runtime compartidas, scripts del mantenedor (`postinst`/`prerm`/`postrm` en Debian y `%post`/`%preun`/`%postun` en RPM), pipeline de construcción determinista local y automatización de distribución en **GitHub Releases** mediante **GitHub Actions** para los paquetes binarios:
1. **Debian / Ubuntu / Mint / Pop!_OS:** `mx-options_0.1.0_amd64.deb`
2. **Fedora / RHEL / OpenSUSE:** `mx-options-0.1.0-1.x86_64.rpm`

### 1.1 Objetivos en Alcance (Fase 5):

1. **Jerarquía FHS Común para Linux (Base Compartida):**
   - **Binarios en `/usr/bin/` (modo `0755`, `root:root`, stripped):**
     - `mx-daemon`: Servicio en segundo plano para arbitraje HID++, uinput y servidor IPC Unix socket.
     - `mx-gui`: Interfaz gráfica de usuario en Tauri v2 (Svelte 5 + WebKitGTK).
     - `mx-diag`: Herramienta de telemetría, inspección de salud, diagnóstico de hardware y convivencia.
     - `mx-cli`: Utilidad de línea de comandos para control interactivo y consultas rápidas.
     - `mx-probe`: Sonda directa de diagnóstico e inspección de reportes HID++.
   - **Reglas udev en `/usr/lib/udev/rules.d/70-mx-options.rules` (modo `0644`, `root:root`):**
     - Prioridad 70 para evaluarse antes de `73-seat-late.rules`.
     - Permisos `TAG+="uaccess"` y `MODE="0660"` para `/dev/hidraw*` de Logitech (`046d`) y `/dev/uinput`.
   - **Unidad systemd user en `/usr/lib/systemd/user/mx-options.service` (modo `0644`, `root:root`):**
     - Configuración de arranque apuntando canónicamente a `/usr/bin/mx-daemon`.
     - Sandboxing completo verificado (puntuación 4.0 OK: `ProtectSystem=strict`, `ProtectHome=read-only`, `PrivateTmp=yes`, etc.).
     - Arranque bajo demanda gobernado por la GUI (`mx-gui`), sin forzar symlinks globales en `/etc/`.
   - **Lanzador de Escritorio XDG en `/usr/share/applications/mx-options.desktop` (modo `0644`, `root:root`):**
     - `Exec=/usr/bin/mx-gui`.
     - `Icon=mx-options`.
     - `Categories=Settings;HardwareSettings;`.
     - `StartupWMClass=mx-gui`.
   - **Iconografía Hicolor Oficial (Opción 2 Esmeralda Vibrante):**
     - `/usr/share/icons/hicolor/{16x16,24x24,32x32,48x48,64x64,128x128,256x256,512x512}/apps/mx-options.png` (modo `0644`, 8-bit RGBA).
     - `/usr/share/icons/hicolor/scalable/apps/mx-options.svg` (modo `0644`, SVG vectorial).
   - **Documentación de Paquete:**
     - `/usr/share/doc/mx-options/copyright` (licencia GPL-2.0 en formato Debian copyright machine-readable).
     - `/usr/share/doc/mx-options/changelog.Debian.gz` (registro de cambios comprimido).

2. **Empaquetado Debian (`.deb`):**
   - Metadatos en `DEBIAN/control`: `Package: mx-options`, `Version: 0.1.0-1`, `Architecture: amd64`, `Section: utils`, `Priority: optional`.
   - Dependencias automáticas calculadas con `dpkg-shlibdeps` sobre los binarios release stripped + `libayatana-appindicator3-1, udev, systemd`.
   - Scripts del mantenedor:
     - `postinst`: migración de regla udev legacy en `/etc/udev/rules.d/70-mx-options.rules`, recarga udev (`udevadm control --reload-rules && udevadm trigger --subsystem-match=hidraw --subsystem-match=misc`), y actualización de caché de iconos (`gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true`).
     - `prerm`: Invariante Zero Dead Buttons. Si es `remove` o `purge`, envía `pkill -x -TERM mx-daemon` con espera de hasta 2.5 s para forzar la salida limpia en hardware antes de borrar el binario. En `upgrade`, no toca el hardware.
     - `postrm`: recarga limpia de udev y caché de iconos en `remove` o `purge`.

3. **Empaquetado Fedora / RPM (`.rpm`):**
   - Especificación RPM (`packaging/rpm/mx-options.spec`):
     - `Name: mx-options`, `Version: 0.1.0`, `Release: 1%{?dist}`.
     - `Summary: Native control and configuration suite for Logitech MX Vertical and MX series mice on Linux`.
     - `License: GPL-2.0-only`.
     - `Requires: webkit2gtk4.1, gtk3, libayatana-appindicator-gtk3, systemd, systemd-udev`.
     - Secciones `%post`, `%preun`, `%postun` sincronizadas con la semántica Debian: `%post` para udev/iconos; `%preun` ($1 = 0) para parada limpia; `%postun` ($1 = 0) para limpieza final.

4. **Pipeline Determinista Local:**
   - `npm --prefix crates/mx-gui/ui run build` previo a la compilación release.
   - `cargo build --release --locked --workspace`.
   - `strip --strip-unneeded` sobre los 5 binarios (`mx-daemon`, `mx-gui`, `mx-diag`, `mx-cli`, `mx-probe`).
   - Staging unificado en `target/staging/`.
   - `scripts/package-deb.sh` y `scripts/package-rpm.sh`.

5. **Distribución y CI/CD en GitHub (`.github/workflows/release.yml`):**
   - Disparo por tags semánticos (ej. `v0.1.0`).
   - Compilación y empaquetado determinista.
   - Publicación automática de instaladores `.deb` y `.rpm` con sumas SHA-256 en **GitHub Releases**.

---

## 2. Definición de Constantes del Sistema (Pilar 1)

| Constante | Tipo | Valor | Justificación Técnica |
|---|---|---|---|
| `PKG_NAME` | `&str` | `"mx-options"` | Nombre canónico del paquete (minúsculas, estándar Debian/RPM). |
| `PKG_VERSION` | `&str` | `"0.1.0"` | Versión alineada con `Cargo.toml`. |
| `PKG_DEB_REVISION` | `&str` | `"1"` | Revisión del paquete Debian (`0.1.0-1`). |
| `MAX_PACKAGE_SIZE_BYTES` | `u64` | `15_728_640` (15 MB) | Meta numérica estricta declarada en el Pilar 7 de `STATUS.md`. |
| `GRACEFUL_STOP_TIMEOUT_SECS` | `f64` | `2.5` | Tiempo límite de espera en `prerm` / `%preun` antes de timeout. |
| `INSTALL_DIR_BIN` | `&str` | `"/usr/bin"` | Destino FHS para ejecutables del sistema. |
| `INSTALL_DIR_UDEV` | `&str` | `"/usr/lib/udev/rules.d"` | Destino estándar para reglas udev provistas por paquetes. |
| `INSTALL_DIR_SYSTEMD` | `&str` | `"/usr/lib/systemd/user"` | Destino estándar para servicios systemd de usuario del sistema. |
| `INSTALL_DIR_DESKTOP` | `&str` | `"/usr/share/applications"` | Directorio XDG de lanzadores de escritorio. |
| `INSTALL_DIR_ICONS` | `&str` | `"/usr/share/icons/hicolor"` | Directorio base de temas de iconos FreeDesktop. |

---

## 3. Declaración de Invariantes (Pilar 2)

1. **Invariante 1 — Cero Fugas de Entorno de Desarrollo (Zero Local Paths):**
   Ningún archivo del paquete (binarios, lanzador `.desktop`, servicio `.service`) debe contener rutas hardcodeadas al directorio de usuario del desarrollador (`/home/panxio`). Todas las referencias deben ser `/usr/bin/mx-*`.
2. **Invariante 2 — Zero Dead Buttons en Desinstalación:**
   En desinstalación completa (`remove` o `purge` / RPM `$1 = 0`), los scripts de mantenedor deben enviar `SIGTERM` limpio a las instancias activas de `mx-daemon` para permitir que el hilo de hardware ejecute `set_cid_divert(cid, false)` y `ReleaseAll` antes de que el binario desaparezca del disco. En actualización (`upgrade` / RPM `$1 = 1`), no se toca el hardware.
3. **Invariante 3 — Propiedad y Permisos Canónicos FHS:**
   Todos los directorios deben tener permisos `0755` y todos los archivos deben pertenecer a `root:root`. Los binarios ejecutables deben tener permisos `0755` y los archivos de configuración/activos `0644`.
4. **Invariante 4 — Idempotencia y Resiliencia en Scripts de Mantenimiento:**
   Todos los scripts deben usar `set -e`, pero las invocaciones a utilidades auxiliares de entorno gráfico (`gtk-update-icon-cache`, `update-desktop-database`) deben protegerse con `|| true` para no fallar en entornos mínimos, servidores headless o chroots de CI sin subsistema gráfico.
5. **Invariante 5 — Integridad y Automatización de Dependencias (Regla de Oro):**
   Las dependencias compartidas no se inventan ni se copian de memoria: se computan mediante `dpkg-shlibdeps` sobre los binarios stripped compilados para la arquitectura objetivo.

---

## 4. Arnés de Verificación Automatizado (`scripts/audit-packages.sh`)

Script que se ejecuta sobre el paquete `.deb` (y `.rpm`) generado:
1. Verifica que el tamaño no supere los 15 MB.
2. Verifica con `dpkg-deb -c` que el 100% de los archivos y carpetas pertenezcan a `root/root`.
3. Verifica que los permisos sean `0755` para directorios y binarios, y `0644` para archivos de datos.
4. Inspecciona con `strings` que no exista ninguna referencia a `/home/panxio` en los ejecutables o archivos de configuración.
5. Inspecciona metadatos con `dpkg-deb -I`.
