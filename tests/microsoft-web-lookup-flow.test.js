import test from 'node:test';
import assert from 'node:assert/strict';

// Mock localStorage para Node test runner
function createMockStorage() {
  const map = new Map();
  return {
    getItem(key) { return map.has(key) ? map.get(key) : null; },
    setItem(key, value) { map.set(key, String(value)); },
    removeItem(key) { map.delete(key); },
    clear() { map.clear(); }
  };
}

if (!global.localStorage) {
  global.localStorage = createMockStorage();
}

import { tryDirectLocalCommand } from '../src/lib/utils/localDirect.js';

test('Flujo Autoconocimiento Web Microsoft 1: Estructura de Respuesta con Solución y Consejos', async () => {
  const query = '¿Cómo resolver la falla de sincronización de licencias KMS en Windows Server 2022?';
  const res = await tryDirectLocalCommand('send_message', { message: query });
  
  assert.ok(res);
  assert.ok(res.text);
  assert.ok(res.text.includes('### Solución'), 'La respuesta debe incluir la sección ### Solución');
  assert.ok(res.text.includes('### Consejos y Recomendaciones'), 'La respuesta debe incluir ### Consejos y Recomendaciones');
});

test('Flujo Autoconocimiento Web Microsoft 2: Dominio Oficial Microsoft Whitelist Preservado', async () => {
  const query = '¿Cómo reparar la corrupción de BitLocker TPM 2.0?';
  const res = await tryDirectLocalCommand('send_message', { message: query });
  
  assert.ok(res);
  assert.ok(res.text.includes('Microsoft') || res.text.includes('Solución'));
});
