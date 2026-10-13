# Especificación Técnica (SDD) — F0: Protocolo HID++ y Sonda de Hardware (mx-probe)

## 1. Propósito y Alcance
Proveer la biblioteca base `mx-core` para la codificación, decodificación y gestión del protocolo Logitech HID++ (versiones 2.0 y 4.x), y una herramienta de diagnóstico por línea de comandos `mx-probe` (y `mx-diag`) que verifique el **Kill-gate crítico** de hardware:
- Descubrimiento de características (*Feature Discovery*) en el ratón Logitech MX Vertical (`046d:b020`).
- Comprobación de existencia de la característica `0x1B04` (*Reprogrammable Keys / Button Diversion*).
- Lectura de nivel y estado de batería (características `0x1000` o `0x1004`).
- Comprobación de si el firmware expone la característica `0x2201` (*Adjustable DPI*) o si es fija en firmware.

---

## 2. Definición del Protocolo HID++ 2.0+

### 2.1 Estructura de Tramas (Reportes HID)
El protocolo HID++ 2.0 utiliza dos tamaños principales de reporte sobre `/dev/hidraw`:

#### Reporte Corto (Short Report — 7 bytes):
- Byte 0: `Report ID` (`0x10`)
- Byte 1: `Device Index` (`0xFF` para el propio dispositivo o `0x01`..`0x06` en receptor Unifying/Bolt)
- Byte 2: `Feature Index` (`0x00` para Root/Feature Set)
- Byte 3: `Function ID & Software ID` (bits 7..4 = Function ID, bits 3..0 = Software ID `0x01`..`0x0F`)
- Bytes 4..6: `Parameters` (3 bytes de payload)

#### Reporte Largo (Long Report — 20 bytes):
- Byte 0: `Report ID` (`0x11`)
- Byte 1: `Device Index` (`0xFF` para conexión directa Bluetooth/USB)
- Byte 2: `Feature Index`
- Byte 3: `Function ID & Software ID`
- Bytes 4..19: `Parameters` (16 bytes de payload)

### 2.2 Características Críticas (Feature IDs)
| Feature ID | Nombre | Función Clave |
|---|---|---|
| `0x0000` | `IRoot` | `getFeature(feature_id)` $\to$ devuelve `feature_index` y `feature_type`. |
| `0x0001` | `IFeatureSet` | `getCount()` y `getFeatureId(index)` $\to$ permite listar todas las features del mouse. |
| `0x0003` | `IDeviceInfo` | Nombre del dispositivo, número de serie, tipo de transporte. |
| `0x1000` | `BatteryStatus` | Nivel de batería porcentual (`0..100%`) y estado de carga (`discharging`, `charging`, `full`). |
| `0x1004` | `UnifiedBattery` | Protocolo moderno de batería con niveles de milivoltios y flags de carga. |
| `0x1B04` | `ReprogrammableKeys` | **KILL-GATE:** Lectura de botones reprogramables y activación del bit `divert`. |
| `0x2201` | `AdjustableDPI` | Lectura de lista de DPIs soportados y configuración del sensor. |

### 2.3 Mensajes de Error (Error Responses)
- Reporte `0x8F` (error HID++ 1.0) o respuesta de feature con código de error (típicamente `0xFF` en function ID):
  - `0x01`: Unknown feature / invalid argument.
  - `0x02`: Invalid address / function.
  - `0x05`: Device not present / timeout.

---

## 3. Invariantes del Sistema (Pilar 2)
1. **Sin Panics:** El parser nunca entra en `panic!` ante bytes corruptos, tramas truncadas o respuestas inesperadas del dispositivo. Todo error se propaga como `Result<T, HidppError>`.
2. **Buffer Bounded:** El parser opera sobre tramas fijas de 7 o 20 bytes sin alojamiento dinámico innecesario.
3. **Software ID Determinado:** Cada petición asigna un `software_id` monótono o constante (`0x01`) para emparejar peticiones y respuestas en el bus.
4. **Timeouts Acotados:** Ninguna llamada de I/O en hardware puede bloquear el hilo indefinidamente; timeout máximo de $1000 \text{ ms}$ por lectura.
5. **Aislamiento en Tests:** El core dispone de un trait `Transport` que permite simular respuestas HID++ en memoria para correr TDD sin necesidad de hardware conectado.

---

## 4. Casos Borde y Contingencias
1. **Bluetooth BLE vs Receptor Unifying:** En Bluetooth directo, el `device_index` es siempre `0xFF`. En receptor, el índice suele ser `0x01` o `0x02`. El handshake debe probar `0xFF` primero si el bus es Bluetooth.
2. **Feature Index Dinámico:** Los feature indexes NO son fijos entre modelos de firmware; siempre deben consultarse primero a la feature `0x0000` (IRoot) para resolver el `feature_index` en tiempo de ejecución.
3. **Permisos de lectura `/dev/hidraw*`:** Si el descriptor devuelve `PermissionDenied`, la herramienta debe mostrar un mensaje amigable indicando la necesidad de permisos o de aplicar la regla udev.

---

## 5. Árbitros Objetivos (Criterios de Cierre de F0)
1. **Arnés TDD:** `cargo test -p mx-core` pasa al 100% en menos de 1 segundo (cubriendo tramas cortas/largas, resolución de features y errores).
2. **Kill-gate Hardware:** `cargo run -p mx-probe` identifica el Logitech MX Vertical, extrae el feature count, reporta el nivel de batería y **confirma la presencia de `0x1B04`**.
3. **Clippy & Build:** `cargo clippy --workspace -- -D warnings` pasa con 0 advertencias.
