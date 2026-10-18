#!/usr/bin/env bash
# ==============================================================================
# package-deb.sh — Generador Reproducible del Paquete Debian (.deb)
# ==============================================================================
# Produce target/mx-options_0.1.0-1_amd64.deb con:
# 1. Compilación release stripped de todo el workspace (5 binarios)
# 2. Reglas udev, unidad systemd user y lanzador XDG FreeDesktop
# 3. Tema completo de iconos Hicolor (16x16 a 512x512 y SVG escalable)
# 4. Cálculo exacto y determinista de dependencias compartidas vía dpkg-shlibdeps
# 5. Scripts de mantenedor (postinst, prerm, postrm) con garantía Zero Dead Buttons
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

echo "=============================================================================="
echo " INICIANDO PIPELINE DE EMPAQUETADO DEBIAN (.DEB) — MX OPTIONS LINUX"
echo "=============================================================================="

# Versión derivada dinámicamente de Cargo.toml o de argumentos de invocación
PKG_VERSION="${1:-$(grep '^version' "$REPO/Cargo.toml" | head -n1 | cut -d'"' -f2)}"
PKG_DEB_REV="${2:-1}"
FULL_VERSION="${PKG_VERSION}-${PKG_DEB_REV}"

# 0. Verificación de herramientas necesarias
for cmd in cargo npm strip dpkg-deb dpkg-shlibdeps gzip; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "[!] Error: Falta la herramienta requerida: $cmd" >&2
        exit 1
    fi
done

# 1. Compilación del frontend Svelte
echo "--> [1/6] Compilando frontend Svelte..."
npm --prefix "$REPO/crates/mx-gui/ui" run build

# 2. Compilación de crates Rust en modo release con prefijos remapeados (Cero fugas de $HOME)
echo "--> [2/6] Compilando workspace Rust en modo release..."
RUSTFLAGS="--remap-path-prefix $HOME=/build" cargo build --release --locked --workspace

# 3. Preparación del árbol de Staging FHS
echo "--> [3/6] Preparando estructura FHS en staging..."
STAGING="$REPO/target/debian/staging"
rm -rf "$REPO/target/debian"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/lib/udev/rules.d"
mkdir -p "$STAGING/usr/lib/systemd/user"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/usr/share/doc/mx-options"
mkdir -p "$STAGING/DEBIAN"

# Copia y strip de binarios
BINARIOS=("mx-daemon" "mx-gui" "mx-diag" "mx-cli" "mx-probe")
for bin in "${BINARIOS[@]}"; do
    if [[ ! -f "$REPO/target/release/$bin" ]]; then
        echo "[!] Error: No se encontró el binario target/release/$bin" >&2
        exit 1
    fi
    cp "$REPO/target/release/$bin" "$STAGING/usr/bin/$bin"
    strip --strip-unneeded "$STAGING/usr/bin/$bin"
    chmod 0755 "$STAGING/usr/bin/$bin"
done

# Sanitización de rutas de compilación (Zero Local Paths - Invariante 1)
python3 -c "
import os
staging_bin = '$STAGING/usr/bin'
for name in os.listdir(staging_bin):
    fpath = os.path.join(staging_bin, name)
    if not os.path.isfile(fpath) or os.path.islink(fpath):
        continue
    with open(fpath, 'rb') as f:
        data = f.read()
    old_p = b'$REPO'
    new_p = b'/usr/src/mx-options-linux'.ljust(len(old_p), b'/')
    if old_p in data:
        data = data.replace(old_p, new_p)
        with open(fpath, 'wb') as f:
            f.write(data)
"

# Copia de archivos de configuración y servicios
cp "$REPO/packaging/common/mx-options.service" "$STAGING/usr/lib/systemd/user/mx-options.service"
chmod 0644 "$STAGING/usr/lib/systemd/user/mx-options.service"

mkdir -p "$STAGING/usr/lib/systemd/user-preset"
cp "$REPO/packaging/common/50-mx-options.preset" "$STAGING/usr/lib/systemd/user-preset/50-mx-options.preset"
chmod 0644 "$STAGING/usr/lib/systemd/user-preset/50-mx-options.preset"

cp "$REPO/packaging/common/70-mx-options.rules" "$STAGING/usr/lib/udev/rules.d/70-mx-options.rules"
chmod 0644 "$STAGING/usr/lib/udev/rules.d/70-mx-options.rules"

cp "$REPO/packaging/common/mx-options.desktop" "$STAGING/usr/share/applications/mx-options.desktop"
chmod 0644 "$STAGING/usr/share/applications/mx-options.desktop"

cp "$REPO/packaging/common/copyright" "$STAGING/usr/share/doc/mx-options/copyright"
chmod 0644 "$STAGING/usr/share/doc/mx-options/copyright"

cp "$REPO/LICENSE" "$STAGING/usr/share/doc/mx-options/LICENSE"
chmod 0644 "$STAGING/usr/share/doc/mx-options/LICENSE"

# Copia del tema de iconos Hicolor
ICON_SIZES=(16 24 32 48 64 128 256 512)
for size in "${ICON_SIZES[@]}"; do
    dest_dir="$STAGING/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$dest_dir"
    cp "$REPO/crates/mx-gui/icons/${size}x${size}.png" "$dest_dir/mx-options.png"
    chmod 0644 "$dest_dir/mx-options.png"
done

mkdir -p "$STAGING/usr/share/icons/hicolor/scalable/apps"
cp "$REPO/crates/mx-gui/icons/mx-options.svg" "$STAGING/usr/share/icons/hicolor/scalable/apps/mx-options.svg"
chmod 0644 "$STAGING/usr/share/icons/hicolor/scalable/apps/mx-options.svg"

# Generación de changelog Debian comprimido
cat > "$STAGING/usr/share/doc/mx-options/changelog.Debian" << EOF
mx-options ($FULL_VERSION) stable; urgency=medium

  * Initial release for Linux (Logitech MX Vertical & MX mice support).
  * Native background daemon, systemd user service, and udev rules.
  * GUI built on Tauri v2 (Svelte 5 + WebKitGTK) with top-pinning.
  * Fail-safe Zero Dead Buttons and Zero Stuck Keys.

 -- MX Options Linux Team <mx-options@panxio.dev>  $(date -R)
EOF
gzip -9n -f "$STAGING/usr/share/doc/mx-options/changelog.Debian"
chmod 0644 "$STAGING/usr/share/doc/mx-options/changelog.Debian.gz"

# 4. Cálculo de dependencias compartidas mediante dpkg-shlibdeps
echo "--> [4/6] Calculando dependencias compartidas reales (dpkg-shlibdeps)..."
TMP_CTRL_DIR=$(mktemp -d)
mkdir -p "$TMP_CTRL_DIR/debian"
cat > "$TMP_CTRL_DIR/debian/control" << 'EOF'
Source: mx-options
Package: mx-options
Architecture: amd64
EOF

RAW_SHLIBDEPS=$(cd "$TMP_CTRL_DIR" && dpkg-shlibdeps -O "$STAGING/usr/bin/"* 2>/dev/null | grep "^shlibs:Depends=" | sed 's/^shlibs:Depends=//')
rm -rf "$TMP_CTRL_DIR"

# Compatibilidad t64 (Noble 24.04 <-> Jammy 22.04 / Debian 12 Bookworm)
COMPAT_SHLIBDEPS=$(echo "$RAW_SHLIBDEPS" | sed 's/libgtk-3-0t64/libgtk-3-0t64 | libgtk-3-0/g' | sed 's/libglib2.0-0t64/libglib2.0-0t64 | libglib2.0-0/g')

# Dependencias completas: shlibdeps + dependencias del sistema indispensables (incluyendo procps para pkill)
FINAL_DEPENDS="$COMPAT_SHLIBDEPS, libayatana-appindicator3-1, udev, systemd, procps"

# Cálculo de tamaño instalado (en KiB)
INSTALLED_SIZE=$(du -ks "$STAGING" | cut -f1)

# Versión derivada dinámicamente de Cargo.toml o de argumentos de invocación
PKG_VERSION="${1:-$(grep '^version' "$REPO/Cargo.toml" | head -n1 | cut -d'"' -f2)}"
PKG_DEB_REV="${2:-1}"
FULL_VERSION="${PKG_VERSION}-${PKG_DEB_REV}"

# 5. Generación del archivo DEBIAN/control
echo "--> [5/6] Generando DEBIAN/control y scripts de mantenedor (versión: $FULL_VERSION)..."
cat > "$STAGING/DEBIAN/control" << EOF
Package: mx-options
Version: $FULL_VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: MX Options Linux Team <mx-options@panxio.dev>
Installed-Size: $INSTALLED_SIZE
Depends: $FINAL_DEPENDS
Recommends: libgtk-3-bin, desktop-file-utils
Homepage: https://github.com/Panxio/mxpilot
Description: Control y configuracion nativa para ratones Logitech MX en Linux
 mx-options es una suite ligera y eficiente para configurar ratones
 Logitech MX Vertical y serie MX bajo Linux, sin permisos de root.
 Incluye el servicio de sesion (mx-daemon), la GUI en Tauri v2 (mx-gui),
 la interfaz CLI (mx-cli), la sonda de protocolo (mx-probe) y telemetria (mx-diag).
EOF
chmod 0644 "$STAGING/DEBIAN/control"

# Copia de scripts del mantenedor
cp "$REPO/packaging/deb/postinst" "$STAGING/DEBIAN/postinst"
chmod 0755 "$STAGING/DEBIAN/postinst"

cp "$REPO/packaging/deb/prerm" "$STAGING/DEBIAN/prerm"
chmod 0755 "$STAGING/DEBIAN/prerm"

cp "$REPO/packaging/deb/postrm" "$STAGING/DEBIAN/postrm"
chmod 0755 "$STAGING/DEBIAN/postrm"

# Asegurar permisos canónicos en todos los directorios
find "$STAGING" -type d -exec chmod 0755 {} +

# 6. Construcción del archivo .deb
echo "--> [6/6] Ensamblando paquete con dpkg-deb..."
DEB_FILE="$REPO/target/mx-options_${FULL_VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$STAGING" "$DEB_FILE"

echo ""
echo "=============================================================================="
echo " PAQUETE GENERADO EXITOSAMENTE"
echo " Ubicación: $DEB_FILE"
ls -lh "$DEB_FILE"
echo " SHA-256:  $(sha256sum "$DEB_FILE" | cut -d' ' -f1)"
echo "=============================================================================="
