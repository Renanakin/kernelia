import test from 'node:test';
import assert from 'node:assert/strict';

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

export const lowContextMatchingResults = [];

function recordUtteranceTest(id, category, utterance, response) {
  lowContextMatchingResults.push({
    id,
    category,
    utterance,
    response: typeof response === 'string' ? response : JSON.stringify(response, null, 2)
  });
}

// 1. Utterance: "el pc murio no prende"
test('Matching Expresion Informal 1: "el pc murio no prende"', async () => {
  const q = 'el pc murio no prende';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordUtteranceTest('UTTER-01', 'Encendido / Arranque', q, res.text);
});

// 2. Utterance: "se me dio vuelta la pantalla"
test('Matching Expresion Informal 2: "se me dio vuelta la pantalla"', async () => {
  const q = 'se me dio vuelta la pantalla';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-02', 'Pantalla / Video', q, res.text);
});

// 3. Utterance: "esta lentisimo colgado"
test('Matching Expresion Informal 3: "esta lentisimo colgado"', async () => {
  const q = 'esta lentisimo colgado';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-03', 'Lentitud / Rendimiento', q, res.text);
});

// 4. Utterance: "no puedo hacer clic touchpad no responde"
test('Matching Expresion Informal 4: "no puedo hacer clic touchpad no responde"', async () => {
  const q = 'no puedo hacer clic touchpad no responde';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-04', 'Teclado / Touchpad', q, res.text);
});

// 5. Utterance: "se escucha como robot sin audio"
test('Matching Expresion Informal 5: "se escucha como robot sin audio"', async () => {
  const q = 'se escucha como robot sin audio';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-05', 'Audio / Micrófono', q, res.text);
});

// 6. Utterance: "conecto el pendrive y no sale"
test('Matching Expresion Informal 6: "conecto el pendrive y no sale"', async () => {
  const q = 'conecto el pendrive y no sale';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-06', 'USB / Almacenamiento', q, res.text);
});

// 7. Utterance: "se traga el papel impresora atascada"
test('Matching Expresion Informal 7: "se traga el papel impresora atascada"', async () => {
  const q = 'se traga el papel impresora atascada';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-07', 'Impresoras / Spooler', q, res.text);
});

// 8. Utterance: "me olvide la clave del windows"
test('Matching Expresion Informal 8: "me olvide la clave del windows"', async () => {
  const q = 'me olvide la clave del windows';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-08', 'Inicio de Sesión / Claves', q, res.text);
});

// 9. Utterance: "no abre el chrome explorador se cae"
test('Matching Expresion Informal 9: "no abre el chrome explorador se cae"', async () => {
  const q = 'no abre el chrome explorador se cae';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-09', 'Aplicaciones / NAVEGADOR', q, res.text);
});

// 10. Utterance: "pantallazo azul carita triste"
test('Matching Expresion Informal 10: "pantallazo azul carita triste"', async () => {
  const q = 'pantallazo azul carita triste';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordUtteranceTest('UTTER-10', 'BSOD / Errores Críticos', q, res.text);
});
