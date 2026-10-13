#!/usr/bin/env bash
# scripts/run-f3-audit-cierre-tanda.sh
# Lanza la tanda de auditoría colegiada de CIERRE de Fase 3 con 4 subagentes:
#   - deepseek (deepseek-v4.1-flash)
#   - haiku (claude-haiku-5-5)
#   - muse (muse-spark-1.3-contributor)
#   - luna (gpt-6-luna)
#
# Cumple estrictamente con el Protocolo de Cierre de Fase (AGENTS.md y STATUS.md).

set -uo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
AUDIT_DIR="$REPO/.superpowers/audit_cierre_f3"
LOGS="$AUDIT_DIR/logs"
PROMPTS_DIR="$AUDIT_DIR/prompts"
REPORTS_DIR="$AUDIT_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura y Sistemas Linux (Unix sockets, SO_PEERCRED, actor hid_worker, flock, señales, leaks)"
  "haiku|opencode-go/claude-haiku-5-5|Métricas, Criterios de Aceptación y Rigor SDD/TDD (RAM 3.2MB vs 15MB, latencia p95, 26 tests pasando, cobertura)"
  "muse|opencode-go/muse-spark-1.3-contributor|Kill-Gates, Observabilidad y Telemetría (salud get_health, parada limpia 2s, cero botones muertos, rollback)"
  "luna|opencode-go/gpt-6-luna|Verificaciones Empíricas y Compatibilidad Linux (systemd --user, suspend/resume, XDG_RUNTIME_DIR, uinput)"
)

echo "=== INICIANDO TANDA DE AUDITORÍA DE CIERRE DE FASE 3 (4 SUBAGENTES) ==="
echo "Directorio de logs: $LOGS"
echo "Directorio de reportes: $REPORTS_DIR"
echo ""

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prompt_file="$PROMPTS_DIR/$alias.md"
  cat <<EOF > "$prompt_file"
# AUDITORÍA DE CIERRE Y ENTREGA DE FASE 3: mx-daemon & Socket IPC

Eres el subagente especialista: **$alias** ($model).
Tu rol en esta auditoría de entrega: **$role**.

Tu tarea es realizar una AUDITORÍA EN MODO LECTURA (READ-ONLY) estricta sobre la implementación final de la Fase 3 en \`mx-options-linux\`.
BAJO NINGUNA CIRCUNSTANCIA DEBES MODIFICAR O CREAR ARCHIVOS DE CÓDIGO EN EL REPOSITORIO.
Tu única salida debe ser tu informe técnico de auditoría estructurado en Markdown con veredicto claro y oportunidades de mejora continua.

## Resumen del Entregable
Se ha implementado por completo el servicio en segundo plano \`mx-daemon\`:
1. **Scaffolding y Dependencias:** \`crates/mx-daemon\` en el workspace Cargo con librerías ultralivianas (\`libc\`, \`serde\`, \`serde_json\`, \`log\`, \`evdev\`).
2. **Protocolo IPC Seguro:** \`crates/mx-daemon/src/ipc/protocol.rs\` con comandos (\`Ping\`, \`GetStatus\`, \`GetHealth\`, \`GetBattery\`, \`SetAction\`, \`ReloadProfile\`), validación de CIDs/teclas y respuestas tipadas.
3. **Servidor IPC Unix Socket:** \`crates/mx-daemon/src/ipc/server.rs\` en \`\$XDG_RUNTIME_DIR/mx-options.sock\` con permisos \`0600\`, validación de credenciales de kernel \`SO_PEERCRED\` (\`getsockopt\`), acumulador bounded (64 KB) y timeout de 2 s contra slow-loris.
4. **Exclusividad Mutua:** \`crates/mx-daemon/src/lock.rs\` con \`flock(LOCK_EX | LOCK_NB)\` sobre \`mx-options.lock\`.
5. **Actor de Dueño Único y Fail-Safe:**
   - \`hid_worker\` como dueño exclusivo de \`/dev/hidraw\`.
   - \`uinput_worker\` persistente con \`sync_channel(64)\`, tracking de \`HashSet<KeyCode>\` y emisión inmediata de ReleaseAll ante desconexión (cero teclas pegadas).
6. **Métricas Empíricas Reales:**
   - Compilación release: binario de solo 2.5 MB.
   - 26/26 tests unitarios y de integración pasando al 100% en menos de 0.5 s.
   - Clippy pasando con 0 advertencias.
   - Medición en ejecución real sobre hardware (/dev/hidraw3): **VmRSS = 3348 kB (~3.2 MB)** frente al objetivo de < 15 MB.
   - Parada limpia por \`SIGTERM\` en menos de 0.1 s restaurando el divert.
7. **Servicio systemd:** \`systemd/user/mx-options.service\` y script \`scripts/install-systemd-service.sh\`.

## Archivos a Examinar
- \`crates/mx-daemon/src/\` (\`main.rs\`, \`lib.rs\`, \`state.rs\`, \`workers.rs\`, \`lock.rs\`, \`ipc/protocol.rs\`, \`ipc/server.rs\`)
- \`crates/mx-daemon/tests/ipc_tests.rs\`
- \`specs/F3-daemon-ipc.md\`
- \`systemd/user/mx-options.service\`
- \`STATUS.md\`

## Tu Misión de Auditoría desde tu Especialidad (**$role**)
1. **Veredicto Técnico:** (APROBADO PARA CIERRE DE FASE 3 / APROBADO CON OBSERVACIONES / RIESGO DETECTADO).
2. **Puntos Fuertes:** Evaluación de la adherencia a los 8 consensos previos.
3. **Puntos Ciegos o Casos de Borde Remanentes:** Cualquier detalle sutil en concurrencia, descriptores, señales, socket cleanup o resiliencia.
4. **Propuestas de Mejora Continua para la Fase 4 (GUI Tauri v2):** Cómo la interfaz gráfica debe consumir este daemon vía socket de la forma más limpia.

Sé conciso, técnico y exigente. Entrega tu reporte en Markdown estructurado.
EOF
done

echo "Prompts generados para los 4 agentes en $PROMPTS_DIR."
echo ""

lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="audit-cierre-$alias"

  date +%s > "$LOGS/$log_prefix.start_ts"
  date +%H:%M:%S > "$LOGS/$log_prefix.inicio"
  echo "EJECUTANDO" > "$LOGS/$log_prefix.status"

  systemd-run --user --scope -p MemoryMax=2G \
    timeout 10m opencode run "$(cat "$prompt_file")" \
      --standalone \
      --model "$model" \
      --auto \
      --print-logs \
      --log-level info \
      --format json \
      > "$LOGS/$log_prefix.json" 2> "$LOGS/$log_prefix.err" < /dev/null

  local code=$?
  echo "$code" > "$LOGS/$log_prefix.exit"
  date +%H:%M:%S > "$LOGS/$log_prefix.fin"
  date +%s > "$LOGS/$log_prefix.end_ts"

  if [[ $code -eq 0 ]]; then
    echo "COMPLETADO" > "$LOGS/$log_prefix.status"
  else
    echo "FALLIDO (exit $code)" > "$LOGS/$log_prefix.status"
  fi
}

export -f lanzar_agente
export LOGS PROMPTS_DIR

MAX_JOBS=2
echo "Lanzando agentes con concurrencia máxima = $MAX_JOBS..."

pids=()
aliases=()

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"

  while [[ $(jobs -r -p | wc -l) -ge $MAX_JOBS ]]; do
    sleep 2
  done

  echo "--> Despachando agente: $alias ($model)..."
  lanzar_agente "$alias" "$model" &
  pids+=("$!")
  aliases+=("$alias")
done

echo ""
echo "Todos los agentes han sido despachados. Entrando al monitor en vivo..."
echo ""

todos_terminados=false
while [[ "$todos_terminados" == "false" ]]; do
  todos_terminados=true
  now=$(date +%s)

  echo "==================== ESTADO DE AUDITORÍA DE CIERRE F3 ===================="
  printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
  echo "--------------------------------------------------------------------------"

  for item in "${AGENTES[@]}"; do
    IFS="|" read -r alias model role <<< "$item"
    status_file="$LOGS/audit-cierre-$alias.status"
    start_ts_file="$LOGS/audit-cierre-$alias.start_ts"
    json_file="$LOGS/audit-cierre-$alias.json"

    status="PENDIENTE"
    elapsed="0s"
    size="0 B"

    if [[ -f "$status_file" ]]; then
      status=$(cat "$status_file")
    fi

    if [[ -f "$start_ts_file" ]]; then
      start_ts=$(cat "$start_ts_file")
      if [[ "$status" == "EJECUTANDO" ]]; then
        elapsed="$(( now - start_ts ))s"
        todos_terminados=false
      elif [[ -f "$LOGS/audit-cierre-$alias.end_ts" ]]; then
        end_ts=$(cat "$LOGS/audit-cierre-$alias.end_ts")
        elapsed="$(( end_ts - start_ts ))s"
      fi
    else
      todos_terminados=false
    fi

    if [[ -f "$json_file" ]]; then
      bytes=$(wc -c < "$json_file" 2>/dev/null || echo 0)
      if [[ $bytes -ge 1048576 ]]; then
        size="$(awk "BEGIN {printf \"%.1f MB\", $bytes/1048576}")"
      elif [[ $bytes -ge 1024 ]]; then
        size="$(awk "BEGIN {printf \"%.1f KB\", $bytes/1024}")"
      else
        size="${bytes} B"
      fi
    fi

    printf "%-10s %-25s %-15s %-10s %-12s\n" "$alias" "${model:0:24}" "$status" "$elapsed" "$size"
  done

  echo "=========================================================================="
  if [[ "$todos_terminados" == "false" ]]; then
    sleep 5
  fi
done

echo ""
echo "=== TODOS LOS AGENTES HAN FINALIZADO ==="
echo ""

echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 -c "
import os, json

logs_dir = '$LOGS'
reports_dir = '$REPORTS_DIR'
os.makedirs(reports_dir, exist_ok=True)

for alias in ['deepseek', 'haiku', 'muse', 'luna']:
    json_path = os.path.join(logs_dir, f'audit-cierre-{alias}.json')
    out_md = os.path.join(reports_dir, f'{alias}.md')
    if not os.path.exists(json_path):
        print(f'[{alias}] Log JSON no encontrado.')
        continue
    texts = []
    with open(json_path, 'r', encoding='utf-8', errors='replace') as f:
        for line in f:
            line = line.strip()
            if not line: continue
            try:
                data = json.loads(line)
                if data.get('type') == 'text' and 'part' in data:
                    t = data['part'].get('text', '')
                    if t: texts.append(t)
            except Exception: pass
    full_text = '\n'.join(texts).strip()
    with open(out_md, 'w', encoding='utf-8') as out_f:
        out_f.write(full_text)
    print(f'[{alias}] Reporte guardado: {out_md} ({len(full_text)} caracteres)')
"

echo ""
echo "Tanda de auditoría de cierre F3 completada exitosamente."
