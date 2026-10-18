#!/usr/bin/env bash
# ==============================================================================
# package-rpm.sh — Generador Reproducible del Paquete RPM (.rpm)
# ==============================================================================
# Produce target/mx-options-0.1.0-1.x86_64.rpm (Fedora / RHEL / openSUSE) con:
# 1. Compilación release stripped de todo el workspace (5 binarios)
# 2. Reglas udev, unidad systemd user y lanzador XDG FreeDesktop
# 3. Tema completo de iconos Hicolor (16x16 a 512x512 y SVG escalable)
# 4. Sanitización determinista de rutas de compilación (Zero Local Paths)
# 5. Scripts de mantenedor (%post, %preun, %postun) con garantía Zero Dead Buttons
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

echo "=============================================================================="
echo " INICIANDO PIPELINE DE EMPAQUETADO RPM (.RPM) — MX PILOT (MX OPTIONS LINUX)"
echo "=============================================================================="

# Versión derivada dinámicamente de Cargo.toml o de argumentos de invocación
PKG_VERSION="${1:-$(grep '^version' "$REPO/Cargo.toml" | head -n1 | cut -d'"' -f2)}"
PKG_RPM_REV="${2:-1}"
FULL_VERSION="${PKG_VERSION}-${PKG_RPM_REV}"

# 0. Verificación de herramientas necesarias
for cmd in cargo npm strip rpmbuild; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "[!] Error: Falta la herramienta requerida: $cmd (en Debian/Ubuntu: sudo apt install rpm; en Fedora: sudo dnf install rpm-build)" >&2
        exit 1
    fi
done

# 1. Compilación del frontend Svelte
echo "--> [1/5] Compilando frontend Svelte..."
npm --prefix "$REPO/crates/mx-gui/ui" run build

# 2. Compilación de crates Rust en modo release con prefijos remapeados (Cero fugas de $HOME)
echo "--> [2/5] Compilando workspace Rust en modo release..."
RUSTFLAGS="--remap-path-prefix $HOME=/build" cargo build --release --locked --workspace

# 3. Preparación del árbol de Staging FHS
echo "--> [3/5] Preparando estructura FHS en staging..."
RPM_ROOT="$REPO/target/rpm"
STAGING="$RPM_ROOT/staging"
RPM_TOPDIR="$RPM_ROOT/topdir"
rm -rf "$RPM_ROOT"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/lib/udev/rules.d"
mkdir -p "$STAGING/usr/lib/systemd/user"
mkdir -p "$STAGING/usr/lib/systemd/user-preset"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/usr/share/doc/mx-options"
mkdir -p "$RPM_TOPDIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

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

# Asegurar permisos canónicos en todos los directorios
find "$STAGING" -type d -exec chmod 0755 {} +

# 4. Construcción del paquete RPM con rpmbuild
echo "--> [4/5] Ejecutando rpmbuild para versión $FULL_VERSION (x86_64)..."
cp "$REPO/packaging/rpm/mx-options.spec" "$RPM_TOPDIR/SPECS/mx-options.spec"

rpmbuild -bb \
    --target x86_64 \
    --define "_topdir $RPM_TOPDIR" \
    --define "staging_dir $STAGING" \
    --define "pkg_version $PKG_VERSION" \
    --define "pkg_release $PKG_RPM_REV" \
    --define "dist %{nil}" \
    "$RPM_TOPDIR/SPECS/mx-options.spec"

# 5. Copia al directorio target/ raíz
echo "--> [5/5] Copiando artefacto RPM final..."
BUILT_RPM=$(find "$RPM_TOPDIR/RPMS" -name "mx-options-*.rpm" | head -n1)
if [[ -z "$BUILT_RPM" || ! -f "$BUILT_RPM" ]]; then
    echo "[!] Error: rpmbuild no produjo el archivo .rpm esperado" >&2
    exit 1
fi

RPM_FILE="$REPO/target/mx-options-${FULL_VERSION}.x86_64.rpm"
cp "$BUILT_RPM" "$RPM_FILE"

echo ""
echo "=============================================================================="
echo " PAQUETE RPM GENERADO EXITOSAMENTE"
echo " Ubicación: $RPM_FILE"
ls -lh "$RPM_FILE"
echo " SHA-256:  $(sha256sum "$RPM_FILE" | cut -d' ' -f1)"
echo "=============================================================================="
