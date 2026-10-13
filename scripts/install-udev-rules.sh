#!/usr/bin/env bash
# Instala las reglas udev para MX Options Linux y recarga udevadm
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
RULE_FILE="$REPO_DIR/rules/70-mx-options.rules"
TARGET_DIR="/etc/udev/rules.d"

if [[ ! -f "$RULE_FILE" ]]; then
    echo "Error: No se encontró $RULE_FILE"
    exit 1
fi

echo "Instalando $RULE_FILE en $TARGET_DIR/70-mx-options.rules..."
sudo rm -f "$TARGET_DIR/99-mx-options.rules"
sudo cp "$RULE_FILE" "$TARGET_DIR/70-mx-options.rules"
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw
sudo udevadm trigger --subsystem-match=misc

echo "Reglas udev instaladas y recargadas exitosamente."
echo "Permisos actualizados para dispositivos Logitech y uinput."
