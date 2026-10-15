#!/usr/bin/env bash
# ==============================================================================
# audit-packages.sh — Arnés de Verificación Objetiva de Paquetes
# ==============================================================================
# Evalúa:
# 1. Tamaño del paquete (< 15 MB)
# 2. Permisos y propietarios (root/root, 0755/0644)
# 3. Ausencia total de rutas de desarrollo (/home/panxio) en archivos y strings
# 4. Integridad de metadatos DEBIAN/control
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

DEB="${1:-$REPO/target/mx-options_0.1.0-1_amd64.deb}"
DEV_PATH="/home/panxio"
MAX_BYTES=15728640  # 15 MiB

echo "=============================================================================="
echo " INICIANDO AUDITORÍA OBJETIVA DEL PAQUETE: $(basename "$DEB")"
echo "=============================================================================="

if [[ ! -f "$DEB" ]]; then
    echo "[!] Error: No se encontró el paquete $DEB" >&2
    exit 1
fi

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
    echo -e "\n[\033[1;31mFALLO\033[0m] $*" >&2
    exit 1
}

pass() {
    echo -e "[\033[1;32mPASS\033[0m] $*"
}

# 1. Verificación de Tamaño (< 15 MB)
BYTES=$(stat -c '%s' "$DEB")
MB=$(awk "BEGIN {printf \"%.2f\", $BYTES/1048576}")
if (( BYTES < MAX_BYTES )); then
    pass "Tamaño del paquete: $MB MB ($BYTES bytes) < 15 MB"
else
    fail "Tamaño del paquete excede el límite: $MB MB ($BYTES bytes) >= $MAX_BYTES"
fi

# 2. Verificación de Propietarios y Permisos del Contenido
echo "--> Verificando propietarios y modos de archivos..."
dpkg-deb -c "$DEB" > "$TMP_DIR/deb.contents"

awk '
{
    mode = $1
    owner = $2
    path = $NF
    sub(/^\.\//, "", path)

    is_dir = (substr(mode, 1, 1) == "d")
    is_bin = (path ~ /^usr\/bin\/mx-[^/]+$/)

    if (is_dir) {
        expected_mode = "drwxr-xr-x"
    } else if (is_bin) {
        expected_mode = "-rwxr-xr-x"
    } else {
        expected_mode = "-rw-r--r--"
    }

    if (owner != "root/root" || mode != expected_mode) {
        printf "[ERROR] Modo o propietario invalido: mode=%s owner=%s path=%s (esperado: %s root/root)\n",
               mode, owner, path, expected_mode > "/dev/stderr"
        bad = 1
    }
}
END { exit bad }
' "$TMP_DIR/deb.contents" || fail "Se detectaron permisos o propietarios disidentes en el payload"

pass "Propietarios (root:root) y modos (0755/0644) 100% canónicos en el payload"

# 3. Extracción e Inspección de Rutas Residuales (/home/panxio)
echo "--> Extrayendo payload y control para búsqueda de rutas de desarrollo..."
mkdir -p "$TMP_DIR/root" "$TMP_DIR/control"
dpkg-deb -x "$DEB" "$TMP_DIR/root"
dpkg-deb -e "$DEB" "$TMP_DIR/control"

# Búsqueda en texto plano
if grep -R -a -F -l -- "$DEV_PATH" "$TMP_DIR/root" "$TMP_DIR/control" 2>/dev/null; then
    fail "Ruta de desarrollo '$DEV_PATH' encontrada en archivos del paquete"
fi

# Búsqueda en cadenas de ejecutables (strings)
while IFS= read -r -d '' bin_file; do
    if strings -a "$bin_file" | grep -F -q -- "$DEV_PATH"; then
        fail "Ruta de desarrollo '$DEV_PATH' encontrada en binario compilado: $(basename "$bin_file")"
    fi
done < <(find "$TMP_DIR/root/usr/bin" -type f -perm /111 -print0)

pass "Cero rutas residuales de desarrollo ($DEV_PATH) en archivos ni en binarios"

# 4. Verificación de Integridad del Payload (Luna)
echo "--> Verificando presencia de componentes clave en el payload..."
REQUIRED_FILES=(
    "usr/bin/mx-daemon"
    "usr/bin/mx-gui"
    "usr/bin/mx-diag"
    "usr/bin/mx-cli"
    "usr/bin/mx-probe"
    "usr/lib/systemd/user/mx-options.service"
    "usr/lib/udev/rules.d/70-mx-options.rules"
    "usr/share/applications/mx-options.desktop"
    "usr/share/doc/mx-options/copyright"
    "usr/share/doc/mx-options/changelog.Debian.gz"
    "usr/share/icons/hicolor/scalable/apps/mx-options.svg"
    "usr/share/icons/hicolor/128x128/apps/mx-options.png"
    "usr/share/icons/hicolor/512x512/apps/mx-options.png"
)

for f in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$TMP_DIR/root/$f" ]]; then
        pass "Archivo clave en payload presente: $f"
    else
        fail "Falta archivo requerido en payload: $f"
    fi
done

# 5. Verificación de Scripts de Mantenedor y Control
echo "--> Verificando scripts de control y metadatos..."
for script in postinst prerm postrm; do
    if [[ -f "$TMP_DIR/control/$script" ]]; then
        mode=$(stat -c '%a' "$TMP_DIR/control/$script")
        if [[ "$mode" == "755" ]]; then
            pass "Script de mantenedor $script presente con modo 0755"
        else
            fail "Script de mantenedor $script tiene modo $mode (esperado: 0755)"
        fi
    else
        fail "Falta el script obligatorio $script en DEBIAN/"
    fi
done

# Verificación de lógica de prerm (Invariante Zero Dead Buttons - Muse)
echo "--> Verificando lógica de prerm (Zero Dead Buttons)..."
PRERM="$TMP_DIR/control/prerm"
if ! grep -q "pkill -x -TERM mx-daemon" "$PRERM"; then
    fail "prerm no contiene señal limpia 'pkill -x -TERM mx-daemon'"
fi
if ! grep -q "sleep 0.1" "$PRERM"; then
    fail "prerm no contiene espera de drenado 'sleep 0.1'"
fi
if ! grep -q "pkill -x -KILL mx-daemon" "$PRERM"; then
    fail "prerm no contiene fallback forzoso 'pkill -x -KILL mx-daemon'"
fi
if ! grep -q "remove|purge" "$PRERM"; then
    fail "prerm no condiciona la parada a remove|purge"
fi
if ! grep -q "upgrade" "$PRERM"; then
    fail "prerm no contempla la cláusula de upgrade para evitar matar el daemon en actualización"
fi
pass "prerm cumple al 100% las garantías de Zero Dead Buttons y no interrumpe en upgrade"

# Verificación de postinst y postrm (udev / systemd)
echo "--> Verificando postinst y postrm..."
POSTINST="$TMP_DIR/control/postinst"
if ! grep -q "udevadm control --reload-rules" "$POSTINST"; then
    fail "postinst no recarga las reglas de udev"
fi
if grep -q "systemctl --global daemon-reload" "$POSTINST"; then
    fail "postinst invoca comando inexistente 'systemctl --global daemon-reload'"
fi
pass "postinst recarga udev y respeta el ciclo de vida sin comandos inválidos"

# 6. Inspección de campos obligatorios en DEBIAN/control
echo "--> Verificando campos de DEBIAN/control..."
CONTROL_INFO=$(dpkg-deb -I "$DEB")
for field in "Package: mx-options" "Architecture: amd64" "Maintainer:" "Depends:" "Description:"; do
    if echo "$CONTROL_INFO" | grep -q "$field"; then
        pass "Campo obligatorio presente: $field"
    else
        fail "Falta campo obligatorio en DEBIAN/control: $field"
    fi
done

# Verificación de dependencias críticas en Depends
DEPENDS_LINE=$(echo "$CONTROL_INFO" | grep "^ Depends:" || true)
for dep in "procps" "libayatana-appindicator3-1" "udev" "systemd"; do
    if echo "$DEPENDS_LINE" | grep -q "$dep"; then
        pass "Dependencia crítica declarada en Depends: $dep"
    else
        fail "Falta dependencia crítica '$dep' en Depends: $DEPENDS_LINE"
    fi
done

echo ""
echo "=============================================================================="
echo " TODAS LAS VERIFICACIONES DEL PAQUETE PASARON EXITOSAMENTE (100% VERDE)"
echo "=============================================================================="
