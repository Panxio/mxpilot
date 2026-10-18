#!/usr/bin/env bash
# ==============================================================================
# audit-packages.sh — Arnés de Verificación Objetiva de Paquetes (.deb y .rpm)
# ==============================================================================
# Evalúa para cada artefacto (.deb o .rpm):
# 1. Tamaño del paquete (< 15 MB)
# 2. Permisos y propietarios (root/root, 0755/0644)
# 3. Ausencia total de rutas de desarrollo (/home/panxio) en archivos y strings
# 4. Presencia de los 5 binarios, udev, systemd, desktop, licencia e iconos
# 5. Garantía Zero Dead Buttons en scripts de desinstalación (prerm / %preun)
# 6. Integridad de metadatos y dependencias críticas
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

DEV_PATHS=("/home/panxio" "/home/runner")
MAX_BYTES=15728640  # 15 MiB

fail() {
    echo -e "\n[\033[1;31mFALLO\033[0m] $*" >&2
    exit 1
}

pass() {
    echo -e "[\033[1;32mPASS\033[0m] $*"
}

audit_deb() {
    local DEB
    DEB="$(realpath "$1")"
    echo "=============================================================================="
    echo " INICIANDO AUDITORÍA OBJETIVA DEL PAQUETE DEB: $(basename "$DEB")"
    echo "=============================================================================="

    if [[ ! -f "$DEB" ]]; then
        fail "No se encontró el paquete DEB: $DEB"
    fi

    local TMP_DIR
    TMP_DIR=$(mktemp -d)
    trap "rm -rf '$TMP_DIR'" RETURN

    # 1. Verificación de Tamaño (< 15 MB)
    local BYTES MB
    BYTES=$(stat -c '%s' "$DEB")
    MB=$(awk "BEGIN {printf \"%.2f\", $BYTES/1048576}")
    if (( BYTES < MAX_BYTES )); then
        pass "Tamaño del paquete DEB: $MB MB ($BYTES bytes) < 15 MB"
    else
        fail "Tamaño del paquete DEB excede el límite: $MB MB ($BYTES bytes) >= $MAX_BYTES"
    fi

    # 2. Verificación de Propietarios y Permisos del Contenido
    echo "--> Verificando propietarios y modos de archivos (.deb)..."
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
    ' "$TMP_DIR/deb.contents" || fail "Se detectaron permisos o propietarios disidentes en el payload DEB"

    pass "Propietarios (root:root) y modos (0755/0644) 100% canónicos en el payload DEB"

    # 3. Extracción e Inspección de Rutas Residuales (/home/panxio)
    echo "--> Extrayendo payload y control para búsqueda de rutas de desarrollo..."
    mkdir -p "$TMP_DIR/root" "$TMP_DIR/control"
    dpkg-deb -x "$DEB" "$TMP_DIR/root"
    dpkg-deb -e "$DEB" "$TMP_DIR/control"

    [[ -f "$TMP_DIR/root/usr/bin/mx-daemon" ]] || fail "Extracción DEB falló o no produjo el payload esperado"

    for dev_p in "${DEV_PATHS[@]}"; do
        if grep -R -a -F -l -- "$dev_p" "$TMP_DIR/root" "$TMP_DIR/control" 2>/dev/null; then
            fail "Ruta de desarrollo '$dev_p' encontrada en archivos del paquete DEB"
        fi

        while IFS= read -r -d '' bin_file; do
            if grep -a -F -q -- "$dev_p" "$bin_file"; then
                fail "Ruta de desarrollo '$dev_p' encontrada en binario compilado: $(basename "$bin_file")"
            fi
        done < <(find "$TMP_DIR/root/usr/bin" -type f -perm /111 -print0)
    done

    pass "Cero rutas residuales de desarrollo en archivos ni en binarios DEB"

    # 4. Verificación de Integridad del Payload
    echo "--> Verificando presencia de componentes clave en el payload DEB..."
    local REQUIRED_FILES=(
        "usr/bin/mx-daemon"
        "usr/bin/mx-gui"
        "usr/bin/mx-diag"
        "usr/bin/mx-cli"
        "usr/bin/mx-probe"
        "usr/lib/systemd/user/mx-options.service"
        "usr/lib/systemd/user-preset/50-mx-options.preset"
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
            fail "Falta archivo requerido en payload DEB: $f"
        fi
    done

    # 5. Verificación de Scripts de Mantenedor y Control
    echo "--> Verificando scripts de control y metadatos DEB..."
    for script in postinst prerm postrm; do
        if [[ -f "$TMP_DIR/control/$script" ]]; then
            local mode
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

    # Verificación de lógica de prerm (Invariante Zero Dead Buttons)
    echo "--> Verificando lógica de prerm (Zero Dead Buttons)..."
    local PRERM="$TMP_DIR/control/prerm"
    grep -q "pkill -x -TERM mx-daemon" "$PRERM" || fail "prerm no contiene señal limpia 'pkill -x -TERM mx-daemon'"
    grep -q "sleep 0.1" "$PRERM" || fail "prerm no contiene espera de drenado 'sleep 0.1'"
    grep -q "pkill -x -KILL mx-daemon" "$PRERM" || fail "prerm no contiene fallback forzoso 'pkill -x -KILL mx-daemon'"
    grep -q "remove|purge" "$PRERM" || fail "prerm no condiciona la parada a remove|purge"
    grep -q "upgrade" "$PRERM" || fail "prerm no contempla la cláusula de upgrade"
    pass "prerm cumple al 100% las garantías de Zero Dead Buttons y no interrumpe en upgrade"

    # Verificación de postinst y postrm (udev / systemd)
    local POSTINST="$TMP_DIR/control/postinst"
    grep -q "udevadm control --reload-rules" "$POSTINST" || fail "postinst no recarga las reglas de udev"
    if grep -q "systemctl --global daemon-reload" "$POSTINST"; then
        fail "postinst invoca comando inexistente 'systemctl --global daemon-reload'"
    fi
    pass "postinst recarga udev y respeta el ciclo de vida sin comandos inválidos"

    # 6. Inspección de campos obligatorios en DEBIAN/control
    echo "--> Verificando campos de DEBIAN/control..."
    local CONTROL_INFO
    CONTROL_INFO=$(dpkg-deb -I "$DEB")
    for field in "Package: mx-options" "Architecture: amd64" "Maintainer:" "Depends:" "Description:" "Homepage: https://github.com/Panxio/mxpilot"; do
        if echo "$CONTROL_INFO" | grep -q "$field"; then
            pass "Campo obligatorio presente: $field"
        else
            fail "Falta campo obligatorio en DEBIAN/control: $field"
        fi
    done

    local DEPENDS_LINE
    DEPENDS_LINE=$(echo "$CONTROL_INFO" | grep "^ Depends:" || true)
    for dep in "procps" "libayatana-appindicator3-1" "udev" "systemd"; do
        if echo "$DEPENDS_LINE" | grep -q "$dep"; then
            pass "Dependencia crítica declarada en Depends: $dep"
        else
            fail "Falta dependencia crítica '$dep' en Depends: $DEPENDS_LINE"
        fi
    done
}

audit_rpm() {
    local RPM
    RPM="$(realpath "$1")"
    echo "=============================================================================="
    echo " INICIANDO AUDITORÍA OBJETIVA DEL PAQUETE RPM: $(basename "$RPM")"
    echo "=============================================================================="

    if [[ ! -f "$RPM" ]]; then
        fail "No se encontró el paquete RPM: $RPM"
    fi

    for cmd in rpm rpm2cpio cpio; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            fail "Falta herramienta '$cmd' para auditar el paquete RPM"
        fi
    done

    local TMP_DIR
    TMP_DIR=$(mktemp -d)
    trap "rm -rf '$TMP_DIR'" RETURN

    # 1. Verificación de Tamaño (< 15 MB)
    local BYTES MB
    BYTES=$(stat -c '%s' "$RPM")
    MB=$(awk "BEGIN {printf \"%.2f\", $BYTES/1048576}")
    if (( BYTES < MAX_BYTES )); then
        pass "Tamaño del paquete RPM: $MB MB ($BYTES bytes) < 15 MB"
    else
        fail "Tamaño del paquete RPM excede el límite: $MB MB ($BYTES bytes) >= $MAX_BYTES"
    fi

    # 2. Verificación de Propietarios y Permisos del Contenido
    echo "--> Verificando propietarios y modos de archivos (.rpm)..."
    rpm -qplv "$RPM" > "$TMP_DIR/rpm.contents"

    awk '
    {
        mode = $1
        owner = $3
        group = $4
        path = $NF

        is_dir = (substr(mode, 1, 1) == "d")
        is_bin = (path ~ /^\/usr\/bin\/mx-[^/]+$/)

        if (is_dir) {
            expected_mode = "drwxr-xr-x"
        } else if (is_bin) {
            expected_mode = "-rwxr-xr-x"
        } else {
            expected_mode = "-rw-r--r--"
        }

        if (owner != "root" || group != "root" || mode != expected_mode) {
            printf "[ERROR] Modo o propietario invalido en RPM: mode=%s owner=%s:%s path=%s (esperado: %s root:root)\n",
                   mode, owner, group, path, expected_mode > "/dev/stderr"
            bad = 1
        }
    }
    END { exit bad }
    ' "$TMP_DIR/rpm.contents" || fail "Se detectaron permisos o propietarios disidentes en el payload RPM"

    pass "Propietarios (root:root) y modos (0755/0644) 100% canónicos en el payload RPM"

    # 3. Extracción e Inspección de Rutas Residuales (/home/panxio)
    echo "--> Extrayendo payload RPM para búsqueda de rutas de desarrollo..."
    mkdir -p "$TMP_DIR/root"
    (cd "$TMP_DIR/root" && rpm2cpio "$RPM" | cpio -idm 2>/dev/null)

    [[ -f "$TMP_DIR/root/usr/bin/mx-daemon" ]] || fail "Extracción RPM falló o no produjo el payload esperado"

    for dev_p in "${DEV_PATHS[@]}"; do
        if grep -R -a -F -l -- "$dev_p" "$TMP_DIR/root" 2>/dev/null; then
            fail "Ruta de desarrollo '$dev_p' encontrada en archivos del paquete RPM"
        fi

        while IFS= read -r -d '' bin_file; do
            if grep -a -F -q -- "$dev_p" "$bin_file"; then
                fail "Ruta de desarrollo '$dev_p' encontrada en binario compilado RPM: $(basename "$bin_file")"
            fi
        done < <(find "$TMP_DIR/root/usr/bin" -type f -perm /111 -print0)
    done

    pass "Cero rutas residuales de desarrollo en archivos ni en binarios RPM"

    # 4. Verificación de Integridad del Payload RPM
    echo "--> Verificando presencia de componentes clave en el payload RPM..."
    local REQUIRED_FILES=(
        "usr/bin/mx-daemon"
        "usr/bin/mx-gui"
        "usr/bin/mx-diag"
        "usr/bin/mx-cli"
        "usr/bin/mx-probe"
        "usr/lib/systemd/user/mx-options.service"
        "usr/lib/systemd/user-preset/50-mx-options.preset"
        "usr/lib/udev/rules.d/70-mx-options.rules"
        "usr/share/applications/mx-options.desktop"
        "usr/share/doc/mx-options/copyright"
        "usr/share/doc/mx-options/LICENSE"
        "usr/share/icons/hicolor/scalable/apps/mx-options.svg"
        "usr/share/icons/hicolor/128x128/apps/mx-options.png"
        "usr/share/icons/hicolor/512x512/apps/mx-options.png"
    )

    for f in "${REQUIRED_FILES[@]}"; do
        if [[ -f "$TMP_DIR/root/$f" ]]; then
            pass "Archivo clave en payload RPM presente: $f"
        else
            fail "Falta archivo requerido en payload RPM: $f"
        fi
    done

    # 5. Verificación de Scriptlets RPM (%post, %preun, %postun y Zero Dead Buttons)
    echo "--> Verificando scriptlets RPM (%preun Zero Dead Buttons)..."
    local SCRIPTS
    SCRIPTS=$(rpm -qp --scripts "$RPM")
    echo "$SCRIPTS" | grep -q "pkill -x -TERM mx-daemon" || fail "%preun en RPM no contiene 'pkill -x -TERM mx-daemon'"
    echo "$SCRIPTS" | grep -q "sleep 0.1" || fail "%preun en RPM no contiene 'sleep 0.1'"
    echo "$SCRIPTS" | grep -q "pkill -x -KILL mx-daemon" || fail "%preun en RPM no contiene fallback 'pkill -x -KILL mx-daemon'"
    echo "$SCRIPTS" | grep -q '\[ "\$1" = "0" \]' || fail "%preun en RPM no condiciona la parada a desinstalación total (\$1 = 0)"
    echo "$SCRIPTS" | grep -q "udevadm control --reload-rules" || fail "%post en RPM no recarga las reglas udev"
    pass "Scriptlets RPM (%post, %preun, %postun) cumplen Zero Dead Buttons y protección en upgrade"

    # 6. Verificación de Metadatos y Dependencias RPM
    echo "--> Verificando metadatos y Requires del paquete RPM..."
    local RPM_INFO RPM_REQS
    RPM_INFO=$(rpm -qpi "$RPM")
    echo "$RPM_INFO" | grep -q "https://github.com/Panxio/mxpilot" || fail "URL oficial https://github.com/Panxio/mxpilot ausente en RPM"
    pass "URL oficial presente en metadatos RPM: https://github.com/Panxio/mxpilot"

    RPM_REQS=$(rpm -qpR "$RPM")
    for dep in "webkit2gtk4.1" "gtk3" "libayatana-appindicator-gtk3" "systemd" "systemd-udev" "procps-ng"; do
        if echo "$RPM_REQS" | grep -q "^${dep}"; then
            pass "Dependencia crítica declarada en Requires RPM: $dep"
        else
            fail "Falta dependencia crítica '$dep' en Requires RPM"
        fi
    done
}

# Procesamiento de argumentos (admite múltiples paquetes .deb y/o .rpm)
TARGETS=("$@")
if [[ ${#TARGETS[@]} -eq 0 ]]; then
    TARGETS=("$REPO/target/mx-options_0.1.0-1_amd64.deb")
    if [[ -f "$REPO/target/mx-options-0.1.0-1.x86_64.rpm" ]]; then
        TARGETS+=("$REPO/target/mx-options-0.1.0-1.x86_64.rpm")
    fi
fi

for pkg in "${TARGETS[@]}"; do
    case "$pkg" in
        *.deb)
            audit_deb "$pkg"
            ;;
        *.rpm)
            audit_rpm "$pkg"
            ;;
        *)
            fail "Formato de paquete no soportado para auditoría: $pkg"
            ;;
    esac
done

echo ""
echo "=============================================================================="
echo " TODAS LAS VERIFICACIONES DE PAQUETES PASARON EXITOSAMENTE (100% VERDE)"
echo "=============================================================================="
