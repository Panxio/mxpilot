# Lecciones de Gobernanza, Autonomía y Seguridad de Subagentes

> **Módulo:** Gobernanza Multi-Agente y Protocolo de Delegación (`docs/SUBAGENTES.md`)  
> **Fecha:** 2026-10-14  
> **Origen:** Hallazgos y observaciones de la Sesión 14 (Fase 4.5: Endurecimiento y Ethical Hacking)  
> **Estado:** Documento de trabajo activo para maduración en próximas sesiones.

---

## 1. El Doble Filo de la Autonomía de los Subagentes

Durante la Fase 4.5, se evidenció con claridad la tensión entre **iniciativa autónoma** y **seguridad del repositorio**:

### 1.1 El valor de la autonomía (Aspecto positivo)
* **Muse (`muse-spark-1.3-contributor`):** No se limitó a sugerir ideas en prosa; implementó directamente 21 pruebas de fuzzing y ráfagas en Rust (`crates/mx-daemon/tests/ipc_hardening.rs` y `crates/mx-gui/tests/client_hardening_test.rs`). Esto elevó la suite de tests a 58 pruebas automáticas pasando al 100%.
* **DeepSeek (`deepseek-v4.1-flash`):** Creó un script ejecutable completo en Python (`scripts/security/f45-ipc-attacks.py`) que probó 9 vectores de ataque reales contra el daemon en vivo.
* **Haiku (`claude-haiku-5-5`):** Escribió una especificación exhaustiva de 35 casos OWASP en `specs/F4.5-matriz-seguridad.md`.

*Conclusión:* La autonomía aporta un enorme valor de ingeniería y acelera drásticamente el desarrollo cuando los agentes producen código auxiliar (tests, arneses, especificaciones).

### 1.2 El riesgo de la autonomía sin contención (Llamado de atención)
* **Modificación no supervisada del árbol de trabajo:** Al ejecutarse con `opencode run ... --auto`, los subagentes tuvieron acceso directo de escritura y ejecución de comandos sobre el workspace principal.
* **Riesgo latente:** Si un modelo alucina, interpreta mal una orden o ejecuta comandos destructivos (`rm -rf`, `git reset --hard`), podría dañar o borrar código de producción en la rama principal.

---

## 2. Protocolo Preventivo de Contención de Git (Worktrees y Ramas)

Para preservar la iniciativa de los agentes sin poner en riesgo la estabilidad del proyecto, se establece la siguiente doctrina:

1. **La rama principal (`master` / `main`) es sagrada:**
   * Ningún subagente debe operar directamente sobre el directorio de trabajo de la rama principal.
2. **Uso de Ramas Feature Dedicadas:**
   * Cada fase o unidad de trabajo no trivial debe residir en una rama (`feature/<nombre-fase>`).
   * Al concluir y verificar todos los árbitros objetivos (tests, linter, compilación), se realiza el merge controlado (`git merge --no-ff`) hacia `master`.
3. **Uso de Git Worktrees Aislados para Tandas Multi-Agente:**
   * Cuando se despachen agentes con permiso de generar código o tests, cada uno debe operar en su propio directorio de trabajo aislado:
     ```bash
     git worktree add ../mx-wt-<alias> -b feat/<alias>-<tarea>
     ```
   * El orquestador es quien inspecciona los diffs en el worktree, extrae lo que sirve y descarta lo riesgoso antes de fusionar.

---

## 3. Límites en el Sistema Operativo (Sandboxing)

Más allá de Git, el sistema operativo provee barreras duras contra comandos destructivos:

* **Montajes de Solo Lectura con `systemd-run`:**
  Para tandas puramente de auditoría o revisión donde los agentes no deban tocar código:
  ```bash
  systemd-run --user --scope -p MemoryMax=2G \
    --property=ReadOnlyPaths=/home/panxio/Projects/mx-options-linux \
    --property=ReadWritePaths=/home/panxio/Projects/mx-options-linux/.superpowers \
    opencode run ...
  ```
  Esto garantiza que cualquier intento de `write_file` o `rm` en el código fuente sea bloqueado directamente por el kernel de Linux (`Read-only file system`).

---

## 4. Tarea Pendiente: Maduración con la Documentación Oficial de OpenCode

Para la próxima sesión o en un momento natural de refinamiento del tooling:

1. **Consultar la documentación oficial de OpenCode:**
   * Investigar si existen flags nativos de CLI como `--read-only`, `--tools-exclude`, o modos de ejecución sin herramientas de shell/escritura.
   * Investigar el sistema de plugins y hooks de OpenCode para interceptar llamadas destructivas antes de su ejecución.
2. **Definir políticas de roles por agente:**
   * Rol *Auditor / Revisor*: Sin herramientas de escritura (`read-only`).
   * Rol *Developer / Tester*: Escritura permitida exclusivamente dentro de su branch/worktree asignado.
3. **Actualizar `scripts/run-dev.sh` y `scripts/templates/template-tanda.sh`** con estas opciones validadas.
