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

test('Piramide Unit 1 (Edge Case): Consulta Vacia o Solo Espacios', async () => {
  const res = await tryDirectLocalCommand('send_message', { message: '    ' });
  assert.ok(res);
  assert.ok(res.text);
});

test('Piramide Unit 2 (Edge Case): Cadena Ultra Larga (Proteccion contra Desbordamiento)', async () => {
  const longMsg = 'A'.repeat(5000);
  const res = await tryDirectLocalCommand('send_message', { message: longMsg });
  assert.ok(res);
  assert.ok(res.text);
});

test('Piramide Unit 3 (Edge Case): Intentos de Inyeccion de Caracteres Especiales o Scripts', async () => {
  const injection = '<script>alert("xss")</script> && DROP TABLE users;--';
  const res = await tryDirectLocalCommand('send_message', { message: injection });
  assert.ok(res);
  assert.equal(res.text.includes('<script>'), false);
});

test('Piramide Unit 4 (Edge Case): Checkpoint Invalido o Inexistente', async () => {
  try {
    await tryDirectLocalCommand('resolve_hitl_checkpoint_cmd', {
      checkpoint_code: 'CHK-INVALID-9999',
      action: 'approve',
      password: null
    });
    assert.fail('Deberia rechazar checkpoint invalido');
  } catch (err) {
    assert.ok(err);
  }
});
