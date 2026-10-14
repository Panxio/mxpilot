#!/usr/bin/env bash
set -euo pipefail

# scripts/measure-gui.sh — Medidor de Memoria Total (Pilar 7)
# Suma el árbol completo de procesos (mx-daemon + mx-gui + WebKitWebProcess + WebKitNetworkProcess)

echo "=== Medición de Memoria de mx-options-linux (Pilar 7) ==="

DAEMON_PID=$(pgrep -x "mx-daemon" | head -n 1 || true)
GUI_PID=$(pgrep -x "mx-gui" | head -n 1 || true)

get_rss_kb() {
    local pid=$1
    if [ -f "/proc/$pid/status" ]; then
        grep "VmRSS:" "/proc/$pid/status" | awk '{print $2}'
    else
        echo "0"
    fi
}

get_pss_kb() {
    local pid=$1
    if [ -f "/proc/$pid/smaps_rollup" ]; then
        grep -E "^Pss:" "/proc/$pid/smaps_rollup" | awk '{print $2}'
    else
        echo "0"
    fi
}

TOTAL_RSS_KB=0
TOTAL_PSS_KB=0

# 1. Medir mx-daemon
if [ -n "$DAEMON_PID" ]; then
    DAEMON_RSS=$(get_rss_kb "$DAEMON_PID")
    DAEMON_PSS=$(get_pss_kb "$DAEMON_PID")
    TOTAL_RSS_KB=$((TOTAL_RSS_KB + DAEMON_RSS))
    TOTAL_PSS_KB=$((TOTAL_PSS_KB + DAEMON_PSS))
    echo "• mx-daemon (PID $DAEMON_PID): RSS $((DAEMON_RSS / 1024)) MB | PSS $((DAEMON_PSS / 1024)) MB"
else
    echo "• mx-daemon: No detectado en ejecución"
fi

# 2. Medir mx-gui y su árbol completo de procesos WebKit
if [ -n "$GUI_PID" ]; then
    GUI_MAIN_RSS=$(get_rss_kb "$GUI_PID")
    GUI_MAIN_PSS=$(get_pss_kb "$GUI_PID")
    echo "• mx-gui principal (PID $GUI_PID): RSS $((GUI_MAIN_RSS / 1024)) MB | PSS $((GUI_MAIN_PSS / 1024)) MB"
    TOTAL_RSS_KB=$((TOTAL_RSS_KB + GUI_MAIN_RSS))
    TOTAL_PSS_KB=$((TOTAL_PSS_KB + GUI_MAIN_PSS))

    # Buscar hijos de WebKit
    CHILD_PIDS=$(pgrep -P "$GUI_PID" || true)
    if [ -n "$CHILD_PIDS" ]; then
        for cpid in $CHILD_PIDS; do
            crss=$(get_rss_kb "$cpid")
            cpss=$(get_pss_kb "$cpid")
            cname=$(grep "Name:" "/proc/$cpid/status" | awk '{print $2}' || echo "child")
            TOTAL_RSS_KB=$((TOTAL_RSS_KB + crss))
            TOTAL_PSS_KB=$((TOTAL_PSS_KB + cpss))
            echo "  └─ $cname (PID $cpid): RSS $((crss / 1024)) MB | PSS $((cpss / 1024)) MB"
        done
    fi
else
    echo "• mx-gui: No detectado en ejecución"
fi

TOTAL_RSS_MB=$((TOTAL_RSS_KB / 1024))
TOTAL_PSS_MB=$((TOTAL_PSS_KB / 1024))
echo "--------------------------------------------------------"
echo "TOTAL RSS (Daemon + GUI + WebKit): $TOTAL_RSS_MB MB ($TOTAL_RSS_KB KB)"
echo "TOTAL PSS (Memoria Proporcional Real): $TOTAL_PSS_MB MB ($TOTAL_PSS_KB KB)"
echo "Presupuesto Inicial: < 80 MB"

if [ "$TOTAL_PSS_MB" -lt 80 ]; then
    echo "RESULTADO PSS: CUMPLE OBJETIVO (< 80 MB) ✅"
else
    echo "RESULTADO PSS: EXCEDE OBJETIVO (> 80 MB) (WebKitGTK Runtime Overhead) ⚠"
fi

