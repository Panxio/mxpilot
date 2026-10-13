#!/usr/bin/env bash
# scripts/install-systemd-service.sh
# Instala el binario mx-daemon en ~/.local/bin/ y la unidad systemd en ~/.config/systemd/user/

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BIN_SRC="$REPO/target/release/mx-daemon"
BIN_DEST="$HOME/.local/bin/mx-daemon"
SERVICE_SRC="$REPO/systemd/user/mx-options.service"
SERVICE_DEST="$HOME/.config/systemd/user/mx-options.service"

echo "=== Instalación de Servicio systemd --user para mx-daemon ==="

mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.config/systemd/user"

echo "1. Compilando binario mx-daemon en release..."
cargo build --release -p mx-daemon --manifest-path "$REPO/Cargo.toml"

echo "2. Copiando binario a $BIN_DEST..."
cp "$BIN_SRC" "$BIN_DEST"
chmod +x "$BIN_DEST"

echo "3. Copiando unidad a $SERVICE_DEST..."
cp "$SERVICE_SRC" "$SERVICE_DEST"

echo "4. Recargando systemd --user daemon..."
systemctl --user daemon-reload

echo ""
echo "¡Instalación completada exitosamente!"
echo ""
echo "Para iniciar el servicio inmediatamente:"
echo "  systemctl --user start mx-options.service"
echo ""
echo "Para habilitar inicio automático al iniciar sesión:"
echo "  systemctl --user enable --now mx-options.service"
echo ""
echo "Para ver logs en vivo:"
echo "  journalctl --user -u mx-options.service -f"
