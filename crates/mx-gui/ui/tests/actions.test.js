import assert from 'node:assert';
import { normalizeKey, areModifiersEqual, isActionEqual, getActionForCid } from '../src/lib/actions.js';

console.log('--- Iniciando Suite de Verificación Canónica de Acciones (Haiku Test Suite) ---');

// Test 1: SingleKey iguales
assert.strictEqual(
  isActionEqual({ type: 'SingleKey', key: 'KEY_LEFTMETA' }, { type: 'SingleKey', key: 'KEY_LEFTMETA' }),
  true,
  'Test 1 falló'
);
console.log('✓ Test 1: SingleKey idénticas -> true');

// Test 2: Mismas claves en distinto orden de serialización (el bug real del daemon)
assert.strictEqual(
  isActionEqual({ key: 'KEY_LEFTMETA', type: 'SingleKey' }, { type: 'SingleKey', key: 'KEY_LEFTMETA' }),
  true,
  'Test 2 falló'
);
console.log('✓ Test 2: {key, type} vs {type, key} (inmune al orden de claves JSON) -> true');

// Test 3: KeyCombo con modificadores en orden inverso
assert.strictEqual(
  isActionEqual(
    { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_TAB' },
    { type: 'KeyCombo', modifiers: ['KEY_LEFTALT', 'KEY_LEFTCTRL'], key: 'KEY_TAB' }
  ),
  true,
  'Test 3 falló'
);
console.log('✓ Test 3: KeyCombo con modificadores en orden inverso -> true');

// Test 4: KeyCombo con duplicados en modificadores
assert.strictEqual(
  isActionEqual(
    { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTCTRL'], key: 'KEY_C' },
    { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_C' }
  ),
  true,
  'Test 4 falló'
);
console.log('✓ Test 4: Modificadores deduplicados -> true');

// Test 5: KeyCombo sin modificadores vs SingleKey de la misma tecla
assert.strictEqual(
  isActionEqual(
    { type: 'KeyCombo', modifiers: [], key: 'KEY_C' },
    { type: 'SingleKey', key: 'KEY_C' }
  ),
  false,
  'Test 5 falló'
);
console.log('✓ Test 5: KeyCombo sin mods vs SingleKey -> false (tipos distintos)');

// Test 6: SingleKey KEY_C vs KeyCombo CTRL+C
assert.strictEqual(
  isActionEqual(
    { type: 'SingleKey', key: 'KEY_C' },
    { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL'], key: 'KEY_C' }
  ),
  false,
  'Test 6 falló'
);
console.log('✓ Test 6: SingleKey vs KeyCombo -> false');

// Test 7: None idéntico
assert.strictEqual(
  isActionEqual({ type: 'None' }, { type: 'None' }),
  true,
  'Test 7 falló'
);
console.log('✓ Test 7: None vs None -> true');

// Test 8: MouseButton vs SingleKey
assert.strictEqual(
  isActionEqual(
    { type: 'MouseButton', button: 'BTN_MIDDLE' },
    { type: 'SingleKey', key: 'KEY_MUTE' }
  ),
  false,
  'Test 8 falló'
);
console.log('✓ Test 8: MouseButton vs SingleKey -> false sin excepción');

// Test 9: Alias normalizados (SUPER, META, KEY_LEFTMETA)
assert.strictEqual(
  isActionEqual(
    { type: 'SingleKey', key: 'SUPER' },
    { type: 'SingleKey', key: 'KEY_LEFTMETA' }
  ),
  true,
  'Test 9 falló'
);
assert.strictEqual(
  isActionEqual(
    { type: 'SingleKey', key: 'meta' },
    { type: 'SingleKey', key: 'KEY_LEFTMETA' }
  ),
  true,
  'Test 9b falló'
);
console.log('✓ Test 9: Alias normalizados (SUPER == META == KEY_LEFTMETA) -> true');

// Test 10: Null o undefined
assert.strictEqual(isActionEqual(null, { type: 'SingleKey', key: 'KEY_C' }), false);
assert.strictEqual(isActionEqual(undefined, undefined), false);
console.log('✓ Test 10: Null y undefined manejados limpiamente (fail-closed) -> false');

// Test 11: getActionForCid con clave numérica y string
const mockMapping = { "253": { type: "SingleKey", key: "KEY_LEFTMETA" } };
assert.deepStrictEqual(getActionForCid(mockMapping, 253), { type: "SingleKey", key: "KEY_LEFTMETA" });
assert.deepStrictEqual(getActionForCid(mockMapping, "253"), { type: "SingleKey", key: "KEY_LEFTMETA" });
assert.strictEqual(getActionForCid(mockMapping, 86), null);
console.log('✓ Test 11: getActionForCid admite clave num 253 y str "253"');

// Test 12: Propiedad simétrica isActionEqual(a, b) === isActionEqual(b, a)
const a = { type: 'KeyCombo', modifiers: ['CTRL', 'ALT'], key: 'TAB' };
const b = { type: 'KeyCombo', modifiers: ['ALT', 'CTRL'], key: 'KEY_TAB' };
assert.strictEqual(isActionEqual(a, b), isActionEqual(b, a));
console.log('✓ Test 12: Simetría equal(a, b) === equal(b, a) verificada');

// Test 13: Workspace navigation (derecha e izquierda)
const wsRightA = { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_RIGHT' };
const wsRightB = { type: 'KeyCombo', modifiers: ['ALT', 'CTRL'], key: 'RIGHT' };
const wsLeft = { type: 'KeyCombo', modifiers: ['KEY_LEFTCTRL', 'KEY_LEFTALT'], key: 'KEY_LEFT' };
assert.strictEqual(isActionEqual(wsRightA, wsRightB), true, 'Test 13a falló');
assert.strictEqual(isActionEqual(wsRightA, wsLeft), false, 'Test 13b falló');
console.log('✓ Test 13: Acciones de Espacio de Trabajo (Derecha / Izquierda) comparadas e igualadas canónicamente');

// --- Suite de Verificación Bilingüe (i18n ES / EN — Fase 6) ---
import {
  DICTIONARY,
  STORAGE_KEY,
  detectInitialLocale,
  translate,
  getLocalizedButtons,
  getLocalizedActionSummary,
} from '../src/lib/i18n.js';

// Test 14: Paridad estricta de claves entre español e inglés
const esKeys = Object.keys(DICTIONARY.es).sort();
const enKeys = Object.keys(DICTIONARY.en).sort();
assert.deepStrictEqual(esKeys, enKeys, 'Test 14 falló: Desfase de claves entre DICTIONARY.es y DICTIONARY.en');
for (const k of esKeys) {
  assert.ok(typeof DICTIONARY.es[k] === 'string' && DICTIONARY.es[k].trim().length > 0, `Clave ES vacía: ${k}`);
  assert.ok(typeof DICTIONARY.en[k] === 'string' && DICTIONARY.en[k].trim().length > 0, `Clave EN vacía: ${k}`);
}
console.log(`✓ Test 14: Paridad 100% entre DICTIONARY.es y DICTIONARY.en (${esKeys.length} claves verificadas)`);

// Test 15: Detección automática de idioma (localStorage y navigator.language)
const mockStorageEn = { getItem: (k) => (k === STORAGE_KEY ? 'en' : null) };
const mockStorageEmpty = { getItem: () => null };
assert.strictEqual(detectInitialLocale({ storage: mockStorageEn, navigatorLanguage: 'es-CL' }), 'en', 'Test 15a falló');
assert.strictEqual(detectInitialLocale({ storage: mockStorageEmpty, navigatorLanguage: 'es-ES' }), 'es', 'Test 15b falló');
assert.strictEqual(detectInitialLocale({ storage: mockStorageEmpty, navigatorLanguage: 'en-US' }), 'en', 'Test 15c falló');
assert.strictEqual(detectInitialLocale({ storage: mockStorageEmpty, navigatorLanguage: 'fr-FR' }), 'en', 'Test 15d falló');
console.log('✓ Test 15: Detección de idioma (localStorage > navigator.language > fallback en) verificada');

// Test 16: Botones y resúmenes localizados
const btnsEs = getLocalizedButtons('es');
const btnsEn = getLocalizedButtons('en');
assert.strictEqual(btnsEs.length, 4);
assert.strictEqual(btnsEn.length, 4);
assert.strictEqual(btnsEs[0].name, 'Botón Superior (DPI)');
assert.strictEqual(btnsEn[0].name, 'Top Button (DPI)');
assert.strictEqual(
  getLocalizedActionSummary(0x00FD, { 253: { type: 'SingleKey', key: 'KEY_LEFTMETA' } }, 'en'),
  'Super (Activities)'
);
assert.strictEqual(
  getLocalizedActionSummary(0x00FD, {}, 'en'),
  'System Default'
);
console.log('✓ Test 16: Traducción reactiva de los 4 CIDs y resúmenes de acción verificada');

console.log('--- TODOS LOS 16 TESTS DE FRONTEND (ACTIONS + I18N) PASARON EN VERDE ---');

