# Documento de Brainstorming y Plan de Proyecto: Reemplazo Moderno de Logitech Options para Linux (Rust + Tauri)

**Proyecto:** Driver & Suite de Configuración Moderna para Logitech MX Vertical (y familia MX) en Linux  
**Stack Principal:** Rust (Core Engine / Protocolo HID++ / uinput) + Tauri v2 (Frontend UI liviano)  
**Fecha:** 13 de Octubre de 2026  
**Ubicación:** `/home/panxio/agy-env`

---

## 1. Resumen Ejecutivo y Objetivos

### 1.1 El Problema
En Linux, los dispositivos de la gama Logitech MX (en particular el **Logitech MX Vertical**) presentan limitaciones importantes "out-of-the-box":
* Los clics convencionales funcionan bajo estándares HID, pero el **botón superior (DPI Switch)** y ciertas funciones avanzadas no emiten eventos estándar de entrada a nivel del sistema operativo.
* El software oficial (**Logitech Options / Options+**) solo está disponible para Windows y macOS, es un software pesado (>300 MB de RAM, basado en Electron y múltiples servicios residentes), y no tiene soporte nativo para Linux.
* Las soluciones existentes en Linux (`Solaar`, `logiops`) son efectivas pero tienen limitaciones:
  * `Solaar`: Interfaz gráfica basada en GTK/Python, puede sentirse anticuada y a veces compleja de configurar para usuarios no técnicos.
  * `logiops`: Es un daemon C++ sin interfaz gráfica nativa; toda la configuración requiere editar manualmente archivos `/etc/logid.cfg`.

### 1.2 La Propuesta
Construir una **aplicación nativa para Linux en Rust + Tauri** que:
1. Sea **ultraliviana** (<30-50 MB de RAM en ejecución).
2. Tenga una **interfaz gráfica moderna, intuitiva y visual** (representación gráfica del mouse con puntos interactivos para reasignar botones).
3. Resuelva el soporte total de los botones del MX Vertical (especialmente el botón superior y botones laterales) mediante emulación de entradas a nivel de kernel (`uinput`).
4. Soporte control de DPI dinámico, monitorización de batería en la bandeja del sistema (*System Tray*), y perfiles de atajos de teclado.

---

## 2. Diagnóstico Técnico: Protocolo HID++ y "Button Diversion"

### 2.1 ¿Por qué el botón superior no responde de forma estándar?
Los dispositivos Logitech utilizan el protocolo propietario **HID++** (versiones 1.0, 2.0 y 4.x) encapsulado dentro de reportes USB/Bluetooth:
* El botón superior (DPI Switch) tiene asignado internamente un Identificador de Control (**CID** típicamente `0x00C3` o `0x00FD` según la tabla de características HID++).
* Por defecto, el microcontrolador del ratón intercepta este botón de forma interna (para alternar entre dos velocidades de DPI pregrabadas en firmware) o envía un reporte HID++ exclusivo en lugar de un código de tecla estándar.
* **Solución técnica (*Button Diversion*):** El software host debe enviar un paquete HID++ a la característica `REPROPROGRAMMABLE_KEYS` (Feature `0x1B04` o similar) con la bandera de **"Divert"** activada. Esto le ordena al mouse: *"No proceses este botón internamente; envíame el evento crudo al sistema operativo"*.

### 2.2 Estado del Arte e Ingeniería Inversa
* **¿Es necesario desensamblar el `.exe` de Logitech Options?**  
  **No es indispensable para la base funcional.** El protocolo HID++ 2.0 / 4.x ya ha sido extensamente analizado y documentado por proyectos de código abierto:
  * [PixlOne/logiops](https://github.com/PixlOne/logiops): Implementación de referencia en C++ para envío de comandos HID++ y mapeo de botones/gestos.
  * [pwr-Solaar/Solaar](https://github.com/pwr-Solaar/Solaar): Implementación exhaustiva en Python de todas las tablas de características HID++ (batería, DPI, diversion, unifying).
  * Módulo del kernel Linux: `drivers/hid/hid-logitech-hidpp.c`.
* **Rol de REA (`morluto/rea`) en el proyecto:**
  * Si deseamos inspeccionar el instalador de Logitech Options (`.exe`) para:
    * Extraer assets visuales vectoriales (SVGs oficiales del MX Vertical y esquemas de botones).
    * Descubrir configuraciones por defecto o características de firmware avanzadas que no estén completamente documentadas en Solaar.
    * Analizar cómo Options estructura internamente los perfiles de aplicación.

---

## 3. Arquitectura Propuesta del Sistema

```
+-----------------------------------------------------------------+
|                        CAPA DE USUARIO                          |
|  Tauri Frontend (UI Moderna: React / Svelte / Vue + Tailwind)   |
|  - Esquema interactivo del MX Vertical                          |
|  - Configuración de DPI en tiempo real                          |
|  - Mapeo de botones a atajos / macros                           |
|  - Indicador de batería en System Tray                          |
+-----------------------------------------------------------------+
                                | IPC (Tauri Commands & Events)
+-----------------------------------------------------------------+
|                       CAPA CORE (RUST)                          |
|                                                                 |
|  1. Device Manager (`hidapi` / `nix` sobre /dev/hidraw):        |
|     - Detección de dispositivos (Vendor ID 0x046d)              |
|     - Soporte para Unifying, Bolt, Bluetooth y Cable USB        |
|                                                                 |
|  2. HID++ Protocol Engine:                                      |
|     - Handshake HID++ 2.0 (Ping, Feature Discovery)             |
|     - Activación de Button Diversion                            |
|     - Lectura de batería (Feature 0x1000 / 0x1004)              |
|     - Control de DPI (Feature 0x2201)                           |
|                                                                 |
|  3. Virtual Input Subsystem (`evdev` / `uinput`):               |
|     - Emisión de pulsaciones de teclas virtuales a nivel kernel  |
|     - Compatible al 100% con Wayland y X11                      |
|                                                                 |
|  4. Settings & Profiles:                                        |
|     - Persistencia en ~/.config/mx-options/config.toml          |
+-----------------------------------------------------------------+
                                | Acceso a dispositivos
+-----------------------------------------------------------------+
|                       KERNEL DE LINUX                           |
|  /dev/hidraw* (con udev rules)       | /dev/uinput              |
|  Logitech MX Vertical (Hardware)     | Teclado/Mouse Virtual    |
+-----------------------------------------------------------------+
```

---

## 4. Preguntas Clave y Decisiones para el Brainstorming

Para debatir con el modelo y afinar el diseño, aquí están los puntos arquitectónicos críticos:

### Pregunta 1: ¿Arquitectura Monolítica (Tauri App) o Cliente-Servidor (Daemon + GUI)?
* **Opción A (Todo-en-uno en Tauri):**  
  La aplicación Tauri ejecuta el bucle de eventos del hardware en un thread de fondo de Rust mientras se minimiza al System Tray.
  * *Ventaja:* Mucho más simple de desarrollar, distribuir e instalar (un solo binario o AppImage).
  * *Desventaja:* Si el usuario cierra completamente la app (o no quiere tener interfaz gráfica cargada), los botones dejan de estar interceptados.
* **Opción B (Daemon Headless `systemd` en Rust + GUI de Configuración en Tauri):**  
  Un servicio de fondo ligero (`mx-daemon`) corre como `systemd --user` y maneja el hardware todo el tiempo. La aplicación Tauri solo se abre para enviar cambios de configuración vía socket IPC (UNIX Domain Socket).
  * *Ventaja:* Robusto, estándar en Linux (similar a cómo opera Bluetooth o audio).
  * *Desventaja:* Mayor complejidad inicial de setup y comunicación entre procesos.

### Pregunta 2: Compatibilidad Wayland vs X11
* Muchas herramientas antiguas usan `xdotool` para emitir atajos, pero `xdotool` falla en Wayland (GNOME/KDE modernos).
* **Solución propuesta:** Utilizar el crate `evdev` o `uinput` de Rust para crear un nodo virtual de teclado a nivel del kernel de Linux (`/dev/uinput`). Esto garantiza funcionamiento transparente tanto en X11 como en Wayland. ¿Conviene agregar soporte para emisión de comandos DBus directos (ej. controlar reproductor de medios MPRIS)?

### Pregunta 3: Gestión de Permisos y Reglas `udev`
* Acceder a `/dev/hidraw*` y `/dev/uinput` habitualmente requiere permisos de superusuario (`root`).
* ¿Cómo diseñamos la experiencia de instalación para que el usuario configure las reglas de `udev` automáticamente (por ejemplo, mediante un grupo `uinput` o regla basada en TAG `uaccess`) sin tener que ejecutar la app con `sudo`?

### Pregunta 4: Stack de Frontend para Tauri
* Opciones candidatas:
  * **Svelte + Tailwind:** Peso mínimo en el bundle, rendimiento instantáneo, curva de aprendizaje baja.
  * **React + Tailwind (Shadcn/UI o Lucide):** Ecosistema masivo de componentes prearmados.
  * **Vanilla TypeScript + HTML/CSS:** Máxima ligereza y cero dependencias complejas.

### Pregunta 5: Alcance Inicial (MVP) vs Características Avanzadas
* **MVP:**
  1. Detectar el MX Vertical conectado por Bluetooth, receptor Unifying o cable.
  2. Leer nivel de batería y mostrarlo en la UI.
  3. Desviar el botón superior y permitir remapearlo a:
     * Tecla Super / Meta (abrir launcher de aplicaciones).
     * Combinación de teclas personalizada (ej. Ctrl+C, Ctrl+V, Alt+Tab).
     * Clic de mouse especial.
  4. Selector de DPI fijo (ej. 400 a 4000 DPI).
* **Fases Futuras:**
  * Gestos de movimiento (mantener botón y deslizar el mouse arriba/abajo/izquierda/derecha).
  * Detección de ventana activa para perfiles por aplicación.
  * Soporte extendido para otros dispositivos (MX Master 3, MX Anywhere, teclados MX Keys).

---

## 5. Plan de Trabajo Inicial

1. **Paso 0:** Validar entorno de desarrollo local (Rust, Cargo, librerías de desarrollo de C/Gtk/Webkit para Tauri, permisos udev).
2. **Paso 1:** Prototipo CLI en Rust (`mx-cli`):
   * Conectar con `/dev/hidraw` usando `hidapi`.
   * Realizar el handshake HID++ 2.0 con el MX Vertical.
   * Interceptar la pulsación del botón superior e imprimirla en consola.
3. **Paso 2:** Módulo de entrada virtual:
   * Instanciar dispositivo `uinput` y emitir una pulsación al recibir el evento del botón.
4. **Paso 3:** Inicializar estructura del proyecto Tauri (`cargo tauri init`):
   * Trasladar el módulo de Rust a la capa backend de Tauri.
   * Crear la interfaz gráfica con soporte para System Tray.
5. **Paso 4:** Persistencia y empaquetado (.deb / AppImage).

---

## 6. Recursos y Enlaces de Referencia

* **Herramienta de análisis/reversing asistido por IA:**  
  * [morluto/rea (Reverse Engineer Anything)](https://github.com/morluto/rea)
* **Investigaciones y debates específicos sobre MX Vertical en Linux:**  
  * [Ubuntu Logitech MX Vertical - Rafal Zarajczyk](https://rzarajczyk.github.io/ubuntu-logitech-mx-vertical/)
  * [Reddit LinuxHardware - Logitech MX Vertical Mouse](https://www.reddit.com/r/linuxhardware/comments/e7m7s0/logitech_mx_vertical_mouse/)
  * [AskUbuntu - Logitech Options on Linux](https://askubuntu.com/questions/1206369/logitech-options-on-linux)
* **Motores y drivers de referencia:**  
  * [PixlOne/logiops](https://github.com/PixlOne/logiops) (C++ HID++ daemon)
  * [pwr-Solaar/Solaar](https://github.com/pwr-Solaar/Solaar) (Python HID++ manager)
  * [Logitech HID++ 2.0 Specification Drafts](https://lekensteyn.nl/files/logitech/hidpp20_features.pdf)
