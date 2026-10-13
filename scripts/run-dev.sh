#!/usr/bin/env bash
# Lanza UN developer externo (agy u OpenCode) en un worktree ya creado.
#
#   bash scripts/run-dev.sh <agy|oc> <worktree> <archivo-prompt> <nombre-log>
#
# Por qué es un script y no la línea directa: el clasificador de Claude Code no deja pasar
# `agy -p "$(cat prompt)" --dangerously-skip-permissions` ni `opencode run … --auto` sueltos,
# pero sí un script commiteado y legible sobre el que el humano pone UNA regla de permiso
# (así se lanzaba en ZFB: probes/run-tanda.sh y en TrastoMaldito: scripts/run-dev.sh). Este archivo es literal a propósito.
# Adaptado de zeroFeeBank/probes/run-tanda.sh (mismas trampas ya pagadas: --add-dir, </dev/null,
# --print-logs, tope por fuera para OpenCode).
#
# No commitea nada: la entrega queda SIN COMMITEAR en el worktree y la audita el orquestador.
# Nunca pisa un log: si <nombre-log> ya existe, aborta (así se perdió la medición de muse).
set -uo pipefail

[[ $# -eq 4 ]] || { echo "uso: bash scripts/run-dev.sh <agy|oc> <worktree> <prompt> <nombre-log>"; exit 2; }
HERR="$1"; WT="$(cd "$2" && pwd)"; PROMPT="$(realpath "$3")"; NOMBRE="$4"
case "$HERR" in agy|oc) ;; *) echo "ABORTA: herramienta '$HERR' (usa agy u oc)"; exit 2;; esac
[[ -f "$PROMPT" ]] || { echo "ABORTA: no existe $PROMPT"; exit 2; }

REPO="$(cd "$(dirname "$0")/.." && pwd)"
LOGS="${LOGS_DIR:-$REPO/.superpowers/logs}"
mkdir -p "$LOGS"
for ext in inicio fin exit err json ndjson; do
  [[ -e "$LOGS/$NOMBRE.$ext" ]] && { echo "ABORTA: ya existe $LOGS/$NOMBRE.$ext (log nuevo por corrida)"; exit 2; }
done
[[ -z "$(git -C "$WT" status --porcelain | grep -v node_modules)" ]] \
  || { echo "ABORTA: el worktree $WT no está limpio"; git -C "$WT" status --short; exit 2; }

AGY_MODEL="${AGY_MODEL:-gemini-3.8-flash-high}"
OC_MODEL="${OC_MODEL:-opencode-go/muse-spark-1.3-contributor}"
TOPE="${TOPE:-25m}"
export AGY_CLI_DISABLE_AUTO_UPDATE=1   # F-053 de ZFB: el auto-update puede matar la corrida

date +%H:%M:%S > "$LOGS/$NOMBRE.inicio"
cd "$WT" || exit 1
if [[ "$HERR" == "agy" ]]; then
  agy -p "$(cat "$PROMPT")" \
      --model "$AGY_MODEL" \
      --add-dir "$WT" \
      --mode accept-edits \
      --print-timeout "$TOPE" \
      --disable-slash-commands \
      --dangerously-skip-permissions \
      --output-format stream-json \
      > "$LOGS/$NOMBRE.ndjson" 2> "$LOGS/$NOMBRE.err" < /dev/null
else
  timeout "$TOPE" opencode run "$(cat "$PROMPT")" \
      --standalone \
      --model "$OC_MODEL" \
      --auto \
      --print-logs \
      --log-level info \
      --format json \
      > "$LOGS/$NOMBRE.json" 2> "$LOGS/$NOMBRE.err" < /dev/null
fi
echo $? > "$LOGS/$NOMBRE.exit"
date +%H:%M:%S > "$LOGS/$NOMBRE.fin"
echo "$HERR $NOMBRE: exit $(cat "$LOGS/$NOMBRE.exit") · $(cat "$LOGS/$NOMBRE.inicio") → $(cat "$LOGS/$NOMBRE.fin")"
