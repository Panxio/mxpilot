/**
 * actions.js — Comparador canónico de acciones y normalización de teclas.
 * Inmune al orden de serialización de claves JSON, insensible a alias y robusto ante conjuntos de modificadores.
 */

/**
 * Normaliza nombres de teclas a su identificador canónico evdev (KEY_*).
 */
export function normalizeKey(key) {
  if (!key || typeof key !== 'string') return '';
  const upper = key.trim().toUpperCase();
  const raw = upper.startsWith('KEY_') ? upper.slice(4) : upper;

  switch (raw) {
    case 'SUPER':
    case 'META':
    case 'LEFTMETA':
      return 'KEY_LEFTMETA';
    case 'RIGHTMETA':
      return 'KEY_RIGHTMETA';
    case 'CTRL':
    case 'LEFTCTRL':
      return 'KEY_LEFTCTRL';
    case 'RIGHTCTRL':
      return 'KEY_RIGHTCTRL';
    case 'ALT':
    case 'LEFTALT':
      return 'KEY_LEFTALT';
    case 'RIGHTALT':
      return 'KEY_RIGHTALT';
    case 'SHIFT':
    case 'LEFTSHIFT':
      return 'KEY_LEFTSHIFT';
    case 'RIGHTSHIFT':
      return 'KEY_RIGHTSHIFT';
    default:
      return upper.startsWith('KEY_') ? upper : `KEY_${upper}`;
  }
}

/**
 * Compara dos arrays de modificadores como conjuntos sin importar el orden ni duplicados.
 */
export function areModifiersEqual(modsA, modsB) {
  const setA = new Set((modsA || []).map(normalizeKey));
  const setB = new Set((modsB || []).map(normalizeKey));
  if (setA.size !== setB.size) return false;
  for (const k of setA) {
    if (!setB.has(k)) return false;
  }
  return true;
}

/**
 * Comparador semántico canónico entre dos ActionPayloads.
 */
export function isActionEqual(a, b) {
  if (!a || !b) return false;
  if (typeof a !== 'object' || typeof b !== 'object') return false;
  if (a.type !== b.type) return false;

  switch (a.type) {
    case 'SingleKey':
      return normalizeKey(a.key) === normalizeKey(b.key);
    case 'KeyCombo':
      return normalizeKey(a.key) === normalizeKey(b.key) && areModifiersEqual(a.modifiers, b.modifiers);
    case 'MouseButton':
      return String(a.button).trim() === String(b.button).trim();
    case 'None':
      return true;
    default:
      return false;
  }
}

/**
 * Obtiene la acción asignada para un CID en el mapping, admitiendo clave numérica o string.
 */
export function getActionForCid(mapping, cid) {
  if (!mapping) return null;
  if (mapping[cid] !== undefined) return mapping[cid];
  if (mapping[String(cid)] !== undefined) return mapping[String(cid)];
  return null;
}
