#!/usr/bin/env bash
# ==============================================================================
# run-f4.5-security-tanda.sh — Tanda Colegiada de Endurecimiento y Ethical Hacking Web
# ==============================================================================
# Propósito:
#   Despachar a los 4 subagentes especializados para diseñar la estrategia,
#   criterios de aceptación y matriz de pruebas de Ethical Hacking y Endurecimiento
#   para Tauri v2 (WebKitGTK, CSP, IPC) y el Daemon en Linux.
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

NOMBRE_TANDA="security_f4_5"
SALIDA_DIR="$REPO/.superpowers/$NOMBRE_TANDA"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Seguridad Linux, IPC y Kernel"
  "haiku|opencode-go/claude-haiku-5-5|Matriz OWASP y Criterios de Aceptación"
  "muse|opencode-go/muse-spark-1.3-contributor|Fuzzing y Resiliencia Adversarial"
  "luna|opencode-go/gpt-6-luna|Seguridad Webview Tauri v2 y CSP"
)

echo "=============================================================================="
echo " INICIANDO TANDA MULTI-AGENTE: $NOMBRE_TANDA (Ethical Hacking & Hardening)"
echo " Directorio: $SALIDA_DIR"
echo "=============================================================================="

# 1. Prompts altamente especializados
cat > "$PROMPTS_DIR/deepseek.md" << 'EOF'
Eres el especialista en Seguridad de Sistemas Linux, IPC y Kernel (deepseek).
El proyecto mx-options-linux utiliza un daemon Rust (`mx-daemon`) con socket Unix (`$XDG_RUNTIME_DIR/mx-options.sock`, permisos 0600, SO_PEERCRED) y un cliente GUI Tauri v2 (`mx-gui`) en Linux.
Antes de iniciar el empaquetado (Fase 5), el usuario solicitó una fase de Endurecimiento y Ethical Hacking (Fase 4.5).

Tu misión técnica:
1. Analizar la superficie de ataque del IPC Unix Domain Socket y el daemon:
   - ¿Qué riesgos existen si un atacante local con el mismo UID o con acceso a sockets intenta explotar el daemon?
   - Evaluación de `SO_PEERCRED`, evasión de permisos, secuestro de descriptores (file descriptor leaking).
   - Invocación de procesos externos: `commands.rs` invoca `/usr/bin/systemctl` sin shell. ¿Es 100% inmune a inyección de argumentos, PATH traversal o manipulación de variables de entorno?
   - Aislamiento de permisos de Linux: `systemd/user/mx-options.service` tiene `NoNewPrivileges=yes` y `RestrictAddressFamilies=AF_UNIX`. ¿Qué directivas adicionales de sandboxing de systemd deberían sumarse (`ProtectSystem`, `ProtectHome`, `PrivateTmp`, `MemoryDenyWriteExecute`)?
2. Proponer un plan de pruebas de Ethical Hacking local (Linux vectors) con comandos reproducibles y criterios de aceptación Pass/Fail estrictos.
EOF

cat > "$PROMPTS_DIR/haiku.md" << 'EOF'
Eres el especialista en Criterios de Aceptación, Métricas y Matriz OWASP (haiku).
El proyecto mx-options-linux utiliza Tauri v2 con Svelte 5 + Tailwind en WebKitGTK, comunicándose vía IPC nativo con un backend en Rust y un daemon headless.
El usuario solicitó una fase previa de Endurecimiento y Ethical Hacking Web (Fase 4.5).

Tu misión técnica:
1. Diseñar una Matriz de Pruebas de Seguridad basada en OWASP Top 10 Desktop / Tauri Security Best Practices:
   - Inyección en IPC: Payloads maliciosos en `set_action` (CIDs fuera de rango, teclas no autorizadas, buffers gigantescos, Unicode malformado, null bytes).
   - Comportamiento de errores y Falso Verde: Asegurar que peticiones inválidas no alteren el estado en memoria (`DaemonState.mapping`) ni cuelguen el webview.
   - Evaluación de dependencias: Criterios para auditorías de CVEs en crates (`cargo audit`) y paquetes npm (`npm audit`).
   - Criterios de aceptación objetivos (Pass / Fail) con métricas numéricas concretas (Pilar 7).
2. Formular un checklist estricto de compuerta de seguridad que deba cumplirse al 100% antes de autorizar la Fase 5 de empaquetado.
EOF

cat > "$PROMPTS_DIR/muse.md" << 'EOF'
Eres el especialista en Fuzzing, Resiliencia Adversarial y Pruebas de Estrés (muse).
En mx-options-linux, `crates/mx-daemon` y `crates/mx-gui` procesan mensajes IPC estructurados en JSON sobre sockets Unix con framing de salto de línea (`\n`).
El usuario solicitó una fase de Endurecimiento y Ethical Hacking (Fase 4.5).

Tu misión técnica:
1. Proponer una suite concreta de Fuzzing y pruebas de estrés adversarial:
   - Fuzzing de parsing JSON en `protocol.rs` (`parse_request`, `ActionPayload`, `parse_key_code`).
   - Test de framing corrupto: envío de datos sin salto de línea, datos truncados, inyección de secuencias `\r\n` o `\0`, ráfagas de 10.000 mensajes concurrentes.
   - Prueba de límite de payload: enviar mensajes de >64 KB, exactamente 64 KB y fragmentación byte por byte.
   - Mitigación de fugas de memoria o degradación de CPU (Slow-Loris, retención de sockets abiertos, desconexión forzada en pleno intercambio).
2. Proporcionar código de prueba Rust / tests unitarios de integración para incorporar al arnés de pruebas automatizado de `cargo test`.
EOF

cat > "$PROMPTS_DIR/luna.md" << 'EOF'
Eres la especialista en Seguridad Webview (Tauri v2 + WebKitGTK) y Aislamiento de Frontend (luna).
En mx-options-linux, `crates/mx-gui` ejecuta Tauri v2 con frontend Svelte 5 en Linux (WebKitGTK). Actualmente `tauri.conf.json` tiene `"csp": null` y `capabilities/default.json` tiene `"core:default"`.
El usuario solicitó una fase de Endurecimiento y Ethical Hacking Web (Fase 4.5).

Tu misión técnica:
1. Diagnosticar los riesgos específicos de tecnología web en Tauri v2 sobre Linux:
   - Configuración óptima de Content Security Policy (CSP): ¿Cuál es la directiva CSP más restrictiva posible que no rompa Svelte 5 ni Tailwind CSS v4, eliminando completamente `unsafe-eval` y restringiendo orígenes?
   - Análisis de Capabilities y Permisos de Tauri: ¿El archivo `capabilities/default.json` expone alguna API innecesaria o peligrosa? ¿Se requiere aislar ventanas o eventos?
   - Interacción con el entorno de escritorio: Prevención de apertura de enlaces no deseados (`target="_blank"`, drag and drop de archivos sobre la ventana web, atajos de desarrollador como F12/Ctrl+Shift+I en builds release).
   - Sanitización del DOM: Verificación de que ningún componente en Svelte (`MouseViewer`, `ActionPicker`, `HealthModal`) use `@html` sin sanitizar o manipulación insegura del DOM.
2. Definir los cambios exactos de configuración y código para blindar el Webview antes de Fase 5.
EOF

# 2. Función de despacho
lanzar_agente() {
  local alias="$1"
  local model="$2"
  local prompt_file="$PROMPTS_DIR/$alias.md"
  local log_prefix="audit-$alias"

  date +%s > "$LOGS/$log_prefix.start_ts"
  date +%H:%M:%S > "$LOGS/$log_prefix.inicio"
  echo "EJECUTANDO" > "$LOGS/$log_prefix.status"

  local code=0
  systemd-run --user --scope -p MemoryMax=2G \
    timeout 10m opencode run "$(cat "$prompt_file")" \
      --standalone \
      --model "$model" \
      --auto \
      --print-logs \
      --log-level info \
      --format json \
      > "$LOGS/$log_prefix.json" 2> "$LOGS/$log_prefix.err" < /dev/null || code=$?

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

# 3. Despacho concurrente (máximo 2 simultáneos)
MAX_JOBS=2
echo "Despachando 4 subagentes de seguridad (máximo $MAX_JOBS en paralelo)..."

for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  while [[ $(jobs -r -p | wc -l) -ge $MAX_JOBS ]]; do
    sleep 2
  done
  echo "--> Despachando: $alias ($model)..."
  lanzar_agente "$alias" "$model" &
done

# 4. Monitor no-invasivo basado en eventos
declare -A prev_status
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  prev_status["$alias"]="INICIANDO"
done

todos_terminados=false
last_heartbeat=0
echo "[$(date +%H:%M:%S)] Monitor iniciado. Supervisando ejecución..."

while [[ "$todos_terminados" == "false" ]]; do
  todos_terminados=true
  now=$(date +%s)
  running_count=0
  running_list=()

  for item in "${AGENTES[@]}"; do
    IFS="|" read -r alias model role <<< "$item"
    status_file="$LOGS/audit-$alias.status"
    start_ts_file="$LOGS/audit-$alias.start_ts"
    json_file="$LOGS/audit-$alias.json"

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
        running_count=$((running_count + 1))
        running_list+=("$alias")
      elif [[ -f "$LOGS/audit-$alias.end_ts" ]]; then
        end_ts=$(cat "$LOGS/audit-$alias.end_ts")
        elapsed="$(( end_ts - start_ts ))s"
      fi
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

    if [[ "${prev_status[$alias]}" != "$status" ]]; then
      echo "[$(date +%H:%M:%S)] [CAMBIO ESTADO] $alias -> $status (tiempo: $elapsed, json: $size)"
      prev_status["$alias"]="$status"
    fi
  done

  if (( now - last_heartbeat >= 30 )) && [[ "$todos_terminados" == "false" ]]; then
    running_str=$(IFS=", "; echo "${running_list[*]}")
    echo "[$(date +%H:%M:%S)] [LATIDO] $running_count/${#AGENTES[@]} en ejecución ($running_str)..."
    last_heartbeat=$now
  fi

  if [[ "$todos_terminados" == "false" ]]; then
    sleep 5
  fi
done

echo ""
echo "=============================================================================="
echo " RESUMEN FINAL CONSOLIDADO — SEGURIDAD & ETHICAL HACKING"
echo "=============================================================================="
printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
echo "------------------------------------------------------------------------------"
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  status=$(cat "$LOGS/audit-$alias.status" 2>/dev/null || echo "DESCONOCIDO")
  start_ts=$(cat "$LOGS/audit-$alias.start_ts" 2>/dev/null || echo 0)
  end_ts=$(cat "$LOGS/audit-$alias.end_ts" 2>/dev/null || echo 0)
  elapsed="$(( end_ts - start_ts ))s"
  json_file="$LOGS/audit-$alias.json"
  size="0 B"
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
echo "=============================================================================="

echo ""
echo "Extrayendo reportes individuales en $REPORTS_DIR..."
python3 - "$REPO" "$SALIDA_DIR" << 'PYEOF'
import sys, os, json

repo = sys.argv[1]
audit_dir = sys.argv[2]
logs_dir = os.path.join(audit_dir, "logs")
reports_dir = os.path.join(audit_dir, "reports")

agentes = ["deepseek", "haiku", "muse", "luna"]

for ag in agentes:
    json_path = os.path.join(logs_dir, f"audit-{ag}.json")
    report_path = os.path.join(reports_dir, f"{ag}.md")
    
    if not os.path.exists(json_path):
        continue
        
    extracted_text = []
    try:
        with open(json_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                    if isinstance(obj, dict):
                        if "part" in obj and isinstance(obj["part"], dict) and "text" in obj["part"]:
                            extracted_text.append(obj["part"]["text"])
                        elif "type" in obj and obj.get("type") == "text" and "content" in obj:
                            extracted_text.append(obj["content"])
                        elif "content" in obj and isinstance(obj["content"], str):
                            extracted_text.append(obj["content"])
                        elif "text" in obj:
                            extracted_text.append(obj["text"])
                except Exception:
                    continue
    except Exception as e:
        print(f"[!] Error leyendo {json_path}: {e}")
        
    full_content = "\n\n".join(extracted_text).strip()
    if full_content:
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write(full_content)
        print(f"[+] Reporte extraído para {ag}: {len(full_content)} caracteres -> {report_path}")
    else:
        err_path = os.path.join(logs_dir, f"audit-{ag}.err")
        if os.path.exists(err_path) and os.path.getsize(err_path) > 0:
            with open(err_path, 'r', encoding='utf-8') as ef, open(report_path, 'w', encoding='utf-8') as rf:
                rf.write(f"# Error en ejecución de {ag}\n\n```\n" + ef.read() + "\n```")
            print(f"[!] Reportado error para {ag} desde .err")

print("Extracción completada.")
PYEOF
