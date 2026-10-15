#!/usr/bin/env bash
# ==============================================================================
# run-f5-audit-cierre-tanda.sh — Auditoría de Cierre de Fase 5 (Empaquetado y Release)
# ==============================================================================
# Subagentes:
#   1. deepseek (deepseek-v4.1-flash): Auditoría de empaquetado FHS, scripts e instalación
#   2. haiku (claude-haiku-5-5): Metadatos control, dependencias shlibdeps y GitHub Actions
#   3. muse (muse-spark-1.3-contributor): Verificación de Zero Dead Buttons y parada limpia
#   4. luna (gpt-6-luna): Verificación empírica del archivo .deb y estándares FreeDesktop
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

NOMBRE_TANDA="audit_cierre_f5"
SALIDA_DIR="$REPO/.superpowers/$NOMBRE_TANDA"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Auditoría FHS y Scripts de Mantenedor"
  "haiku|opencode-go/claude-haiku-5-5|Metadatos Control y CI/CD GitHub Actions"
  "muse|opencode-go/muse-spark-1.3-contributor|Kill-Gate Zero Dead Buttons y Ciclo de Vida"
  "luna|opencode-go/gpt-6-luna|Inspección Empírica del .deb y FreeDesktop"
)

echo "=============================================================================="
echo " INICIANDO TANDA DE AUDITORÍA DE CIERRE FASE 5: EMPAQUETADO Y RELEASE"
echo " Salida: $SALIDA_DIR"
echo " Paquete a evaluar: target/mx-options_0.1.0-1_amd64.deb"
echo "=============================================================================="

# 1. Generar prompts especializados para cada auditor
cat > "$PROMPTS_DIR/deepseek.md" << 'EOF'
Eres DeepSeek (deepseek-v4.1-flash), especialista senior en Arquitectura del Sistema Linux y Packaging Debian/RPM.
Audita la implementación final de empaquetado de Fase 5 en `scripts/package-deb.sh`, `scripts/audit-packages.sh`, y los scripts en `packaging/deb/{postinst,prerm,postrm}` y `packaging/common/`.

Verifica:
1. ¿Los scripts de mantenedor son 100% seguros, idempotentes y sin posibilidad de bloquear dpkg?
2. ¿La migración de udev en `postinst` elimina limpiamente cualquier copia previa de `/etc/udev/rules.d/70-mx-options.rules`?
3. ¿La unidad systemd en `/usr/lib/systemd/user/mx-options.service` arranca limpia y segura con `ExecStart=/usr/bin/mx-daemon`?
4. ¿Existe algún riesgo de permisos o rutas residuales?

Emite tu dictamen técnico con hallazgos y visto bueno o recomendaciones finales.
EOF

cat > "$PROMPTS_DIR/haiku.md" << 'EOF'
Eres Claude Haiku (claude-haiku-5-5), especialista en Metadatos Debian, Gestión de Dependencias y CI/CD en GitHub Actions.
Audita la implementación final de Fase 5:
- `scripts/package-deb.sh` (cálculo de dependencias con `dpkg-shlibdeps`, compatibilidad t64 con `libgtk-3-0t64 | libgtk-3-0`).
- `.github/workflows/release.yml` (disparo por tag `v*`, permisos, artefactos y SHA256).
- `packaging/rpm/mx-options.spec` (estructura para Fedora).
- `LICENSE` (GPL-2.0).

Verifica:
1. ¿El archivo DEBIAN/control generado en `target/mx-options_0.1.0-1_amd64.deb` cumple todas las directivas de Debian?
2. ¿El workflow de GitHub Actions está listo para producción sin cuellos de botella?
3. Revisa métricas y checklist numérico.

Emite tu dictamen técnico con hallazgos y visto bueno o recomendaciones finales.
EOF

cat > "$PROMPTS_DIR/muse.md" << 'EOF'
Eres Muse (muse-spark-1.3-contributor), especialista en Kill-Gates, Telemetría y Ciclo de Vida de Hardware.
Audita la implementación final de Fase 5 en:
- `packaging/deb/prerm` (semántica de desinstalación vs actualización).
- `scripts/audit-packages.sh` (arnés de verificación objetiva).
- Invariante Zero Dead Buttons: ¿Garantiza `prerm` que ante `apt remove` o `dpkg -r` el mouse Logitech MX Vertical recupera su función de hardware nativa (DPI switch) y no queda con botones muertos?

Verifica los tiempos de espera (2.5 s), la señalización `SIGTERM` y el fallback `SIGKILL`.
Emite tu dictamen técnico con hallazgos y visto bueno o recomendaciones finales.
EOF

cat > "$PROMPTS_DIR/luna.md" << 'EOF'
Eres Luna (gpt-6-luna), especialista en Estándares FreeDesktop / XDG y Verificación Empírica de Artefactos.
Audita el artefacto binario generado `target/mx-options_0.1.0-1_amd64.deb` y el arnés `scripts/audit-packages.sh`.

Verifica:
1. Inspecciona el paquete `.deb`:
   - Tamaño (4.15 MB vs meta < 15 MB).
   - Propietarios (`root/root`) y permisos (`0755`/`0644`).
   - Lanzador `.desktop` en `/usr/share/applications/mx-options.desktop` (categorías `Settings;HardwareSettings;`, `Exec=/usr/bin/mx-gui`).
   - Iconos Hicolor de 16x16 a 512x512 y SVG escalable.
   - Cero presencia de `/home/panxio` en binarios o archivos de texto.
2. ¿Cumple el paquete al 100% con los estándares de FreeDesktop?

Emite tu dictamen técnico con hallazgos y visto bueno o recomendaciones finales.
EOF

# 2. Función de despacho desacoplada
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

# 3. Despacho con límite de concurrencia
MAX_JOBS=2
echo "Despachando agentes de auditoría de cierre con concurrencia máxima = $MAX_JOBS..."

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
echo "[$(date +%H:%M:%S)] Monitor iniciado. Escuchando eventos de estado..."

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

    if [[ "${prev_status[$alias]}" != "$status" ]]; then
      echo "[$(date +%H:%M:%S)] [EVENTO] Agente '$alias': ${prev_status[$alias]} -> $status (${elapsed}, salida: ${size})"
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
echo " RESUMEN FINAL CONSOLIDADO DE AUDITORÍA DE CIERRE"
echo "=============================================================================="
printf "%-10s %-25s %-15s %-10s %-12s\n" "AGENTE" "MODELO" "ESTADO" "TIEMPO" "SALIDA"
echo "------------------------------------------------------------------------------"
for item in "${AGENTES[@]}"; do
  IFS="|" read -r alias model role <<< "$item"
  status=$(cat "$LOGS/audit-$alias.status" 2>/dev/null || echo "DESCONOCIDO")
  start_ts=$(cat "$LOGS/audit-$alias.start_ts" 2>/dev/null || echo 0)
  end_ts=$(cat "$LOGS/audit-$alias.end_ts" 2>/dev/null || echo "$start_ts")
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

print("Extracción completada exitosamente.")
PYEOF
