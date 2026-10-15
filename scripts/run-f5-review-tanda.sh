#!/usr/bin/env bash
# ==============================================================================
# run-f5-review-tanda.sh — Tanda Colegiada de Revisión de Fase 5 (Empaquetado .deb)
# ==============================================================================
# Subagentes:
#   1. deepseek (deepseek-v4.1-flash): Arquitectura FHS, Linux Packaging y udev/systemd
#   2. haiku (claude-haiku-5-5): Control Debian, dpkg-shlibdeps y Criterios de Aceptación
#   3. muse (muse-spark-1.3-contributor): Zero Dead Buttons en remoción y Convivencia
#   4. luna (gpt-6-luna): FreeDesktop / XDG (.desktop, Hicolor) e Inspección Empírica
# ==============================================================================

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

NOMBRE_TANDA="review_f5_packaging"
SALIDA_DIR="$REPO/.superpowers/$NOMBRE_TANDA"
LOGS="$SALIDA_DIR/logs"
PROMPTS_DIR="$SALIDA_DIR/prompts"
REPORTS_DIR="$SALIDA_DIR/reports"

mkdir -p "$LOGS" "$PROMPTS_DIR" "$REPORTS_DIR"

AGENTES=(
  "deepseek|opencode-go/deepseek-v4.1-flash|Arquitectura FHS y Packaging Linux"
  "haiku|opencode-go/claude-haiku-5-5|Metadatos Control y Dependencias shlibdeps"
  "muse|opencode-go/muse-spark-1.3-contributor|Zero Dead Buttons y Convivencia Solaar/Logiops"
  "luna|opencode-go/gpt-6-luna|XDG FreeDesktop e Inspección Empírica"
)

echo "=============================================================================="
echo " INICIANDO TANDA COLEGIADA DE REVISIÓN FASE 5: EMPAQUETADO (.DEB)"
echo " Salida: $SALIDA_DIR"
echo " Modo: Solo Lectura (ReadOnlyPaths) sobre el código fuente"
echo "=============================================================================="

# 1. Generar prompts especializados para cada auditor
cat > "$PROMPTS_DIR/deepseek.md" << 'EOF'
Eres DeepSeek (deepseek-v4.1-flash), especialista senior en Arquitectura del Sistema Linux, Packaging Debian/RPM y Servicios de Sistema.
Audita la especificación técnica en `specs/F5-packaging-deb.md` para el empaquetado multi-distro (.deb y .rpm) y distribución de la suite `mx-options-linux`.

Analiza en profundidad y responde con propuestas técnicas concretas:
1. Jerarquía FHS y Reglas udev: ¿Es correcto ubicar las reglas udev en `/usr/lib/udev/rules.d/70-mx-options.rules` tanto para Debian/Ubuntu como para Fedora (UsrMerge / RPM packaging guidelines)?
2. Unidad systemd: La unidad `mx-options.service` va en `/usr/lib/systemd/user/`. ¿Cómo debe gestionarse su activación en `postinst` / `%post` y desactivación en `prerm` / `%preun`? ¿`systemctl --global enable` o dejarlo para que la GUI lo inicie bajo demanda? ¿Qué macros de RPM (`%systemd_user_post`, etc.) o comandos genéricos aplican?
3. Scripts del mantenedor en Debian (`postinst`, `prerm`, `postrm`) y RPM (`%post`, `%preun`, `%postun`): ¿Están cubiertos todos los casos de falla de `udevadm`, `systemctl` y chroots mínimos de instalación?
4. Idempotencia y permisos: Revisa los modos y propietarios de cada archivo en ambos tipos de paquete.

Formato: Reporte técnico en Markdown con hallazgos (clasificados en Crítico, Alto, Medio, Bajo), justificación técnica y código/scripts recomendados para Debian y Fedora.
EOF

cat > "$PROMPTS_DIR/haiku.md" << 'EOF'
Eres Claude Haiku (claude-haiku-5-5), especialista en Metadatos de Empaquetado Linux, Gestión de Dependencias y CI/CD en GitHub Actions.
Audita la especificación técnica en `specs/F5-packaging-deb.md` para el empaquetado (.deb / .rpm) y publicación en GitHub de la suite `mx-options-linux`.

Analiza en profundidad y responde con propuestas técnicas concretas:
1. Metadatos de Paquete:
   - `DEBIAN/control`: Revisa los campos obligatorios y recomendados (Package, Version, Section, Priority, Architecture, Depends, Maintainer, Description).
   - `packaging/rpm/mx-options.spec`: Estructura para Fedora (Name, Version, Release, Requires, Summary, License).
2. Cálculo de dependencias: La especificación propone usar `dpkg-shlibdeps` sobre los binarios release stripped para .deb. ¿Cómo orquestar `dpkg-shlibdeps` dentro de `scripts/package-deb.sh` de forma limpia y reproducible? ¿Qué dependencias estáticas no detectadas por `shlibdeps` deben agregarse manualmente (ej. `libayatana-appindicator3-1`, `udev`, `systemd`)? ¿Cómo se resuelven en Fedora (`webkit2gtk4.1`, `gtk3`, `libayatana-appindicator3`)?
3. Automatización en GitHub Actions (`.github/workflows/release.yml`):
   - Diseño del workflow para dispararse ante tags de release (`v*`).
   - Matriz de compilación y empaquetado para generar `.deb` y `.rpm`.
   - Adjuntar artefactos y sumas SHA256 al GitHub Release automáticamente.
4. Criterios de Aceptación y Checklist: Elabora el checklist numérico de validación previa al release.

Formato: Reporte técnico en Markdown con hallazgos (Crítico, Alto, Medio, Bajo), estructura exacta del archivo `control`, `mx-options.spec` y snippet YAML para GitHub Actions.
EOF

cat > "$PROMPTS_DIR/muse.md" << 'EOF'
Eres Muse (muse-spark-1.3-contributor), especialista en Kill-Gates, Telemetría, Recuperación ante Fallos y Convivencia de Hardware.
Audita la especificación técnica en `specs/F5-packaging-deb.md` para el empaquetado (.deb y .rpm) de la suite `mx-options-linux`.

Analiza en profundidad y responde con propuestas técnicas concretas:
1. Kill-Gate "Zero Dead Buttons" en desinstalación y upgrade:
   Si el usuario desinstala el paquete (`apt remove mx-options`, `dnf remove mx-options`) mientras `mx-daemon` está corriendo y tiene el botón superior (CID 0x00FD) desviado (`Divert`), ¿cómo garantizan `prerm` y `%preun` que el firmware del ratón vuelva a su función de fábrica antes de que los archivos sean borrados?
2. Distinción de ciclo de vida (Upgrade vs Uninstall):
   ¿Cómo diferenciar en `prerm` ($1 = "remove" vs "upgrade") y en RPM `%preun` ($1 = 0 vs $1 = 1) para NO desviar innecesariamente el ratón durante una actualización de paquete?
3. Señalización limpia: ¿Cómo enviar `SIGTERM` desde los scripts de root a los procesos `mx-daemon` de los usuarios normales, respetando el presupuesto de parada limpia y permitiendo que `DivertGuard` ejecute `impl Drop`?
4. Convivencia con Solaar y logiops (`logid`):
   ¿Qué comprobaciones debe hacer `mx-diag` para advertir al usuario si `logid` o `solaar` están compitiendo por los eventos HIDRAW del ratón? ¿Debe `postinst` / `%post` alertar o recomendar algo?

Formato: Reporte técnico en Markdown con análisis de escenarios de riesgo de hardware, scripts de desinstalación seguros y comprobaciones de convivencia para `mx-diag`.
EOF

cat > "$PROMPTS_DIR/luna.md" << 'EOF'
Eres Luna (gpt-6-luna), especialista en Estándares FreeDesktop / XDG, Integración con el Escritorio Linux y Verificación Empírica de Artefactos.
Audita la especificación técnica en `specs/F5-packaging-deb.md` para el empaquetado (.deb / .rpm) y distribución de la suite `mx-options-linux`.

Analiza en profundidad y responde con propuestas técnicas concretas:
1. Integración XDG FreeDesktop:
   - Revisa `assets/desktop/mx-options.desktop`: categorías, `StartupWMClass=mx-gui`, `Exec=/usr/bin/mx-gui`, `Icon=mx-options`.
   - Revisa la jerarquía de iconos Hicolor en `/usr/share/icons/hicolor/`. ¿Están cubiertos todos los tamaños requeridos (16, 24, 32, 48, 64, 128, 256, 512 y SVG scalable)?
2. Limpieza y Cero Rutas Residuales:
   - Diseña un arnés de verificación que inspeccione los paquetes `.deb` y `.rpm` generados (`dpkg-deb -c`, `dpkg-deb -I`, `rpm -qpl`, `strings` sobre binarios) para asegurar 0 referencias a rutas de desarrollo (`/home/panxio`) y 0 permisos incorrectos.
3. Actualización de caches de escritorio:
   - Invocación de `gtk-update-icon-cache` y `update-desktop-database` en scripts postinst/postrm y %post/%postun.
4. Metas Numéricas (Pilar 7):
   - Criterio de verificación para el peso de los paquetes (< 15 MB) y tiempo de instalación (< 5 s).

Formato: Reporte técnico en Markdown con validación de estándares FreeDesktop, script de auditoría de paquetes y recomendaciones de integración.
EOF

# 2. Función de despacho desacoplada con aislamiento systemd-run
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
export LOGS PROMPTS_DIR REPO

# 3. Despacho con concurrencia máxima = 2
MAX_JOBS=2
echo "Despachando agentes de revisión con concurrencia máxima = $MAX_JOBS..."

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
echo " RESUMEN FINAL CONSOLIDADO"
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
