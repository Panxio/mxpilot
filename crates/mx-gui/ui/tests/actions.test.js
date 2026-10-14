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

console.log('--- TODOS LOS 12 TESTS DE ACTIONS.JS PASARON EN VERDE ---');
