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

export const masterSolutionResults = [];

function recordPerfectScoreTest(id, category, query, response) {
  masterSolutionResults.push({
    id,
    category,
    query,
    response: typeof response === 'string' ? response : JSON.stringify(response, null, 2)
  });
}

// 1. Hardware / Power: PC no prende / boton no hace nada
test('Master Solution 1: "aprieto el boton y no pasa nada el pc murio"', async () => {
  const q = 'aprieto el boton y no pasa nada el pc murio';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'), 'Debe incluir la sección ### Solución');
  assert.ok(res.text.includes('### Consejos y Recomendaciones'), 'Debe incluir la sección ### Consejos y Recomendaciones');
  recordPerfectScoreTest('PERFECT-01', 'Hardware / Encendido', q, res.text);
});

// 2. Hardware / Video: pantalla al reves / se dio vuelta
test('Master Solution 2: "la pantalla se dio vuelta de la nada"', async () => {
  const q = 'la pantalla se dio vuelta de la nada';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-02', 'Pantalla / Orientación', q, res.text);
});

// 3. Hardware / Rendimiento: lentisimo / ventilador a full
test('Master Solution 3: "esta lentisimo el ventilador suena mucho"', async () => {
  const q = 'esta lentisimo el ventilador suena mucho';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-03', 'Rendimiento / Temperatura', q, res.text);
});

// 4. Periféricos / Entrada: teclado escribe raro la ñ no sale
test('Master Solution 4: "el teclado escribe raro la ñ no sale"', async () => {
  const q = 'el teclado escribe raro la ñ no sale';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-04', 'Teclado / Idioma', q, res.text);
});

// 5. Audio / Sonido: se escucha como robot sin audio
test('Master Solution 5: "se escucha como robot no tengo sonido en la reunion"', async () => {
  const q = 'se escucha como robot no tengo sonido en la reunion';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-05', 'Audio / Micrófono', q, res.text);
});

// 6. Almacenamiento / USB: pendrive no aparece pide formatear
test('Master Solution 6: "conecto el pendrive y me pide formatear no abre"', async () => {
  const q = 'conecto el pendrive y me pide formatear no abre';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-06', 'USB / Permisos', q, res.text);
});

// 7. Impresora / Spooler: se traga el papel cola atascada
test('Master Solution 7: "la impresora no imprime cola atascada triangulo amarillo"', async () => {
  const q = 'la impresora no imprime cola atascada triangulo amarillo';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-07', 'Impresora / Spooler', q, res.text);
});

// 8. Autenticación / Claves: olvide la clave de windows cuenta bloqueada
test('Master Solution 8: "me olvide la clave del windows cuenta bloqueada"', async () => {
  const q = 'me olvide la clave del windows cuenta bloqueada';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-08', 'Autenticación / Claves', q, res.text);
});

// 9. Redes / Wi-Fi: sin conexion a internet modo avion solo
test('Master Solution 9: "no tengo internet modo avion se activo solo"', async () => {
  const q = 'no tengo internet modo avion se activo solo';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-09', 'Redes / Wi-Fi', q, res.text);
});

// 10. BSOD / Diagnóstico: pantallazo azul carita triste error 0x000
test('Master Solution 10: "pantallazo azul carita triste error 0x000"', async () => {
  const q = 'pantallazo azul carita triste error 0x000';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordPerfectScoreTest('PERFECT-10', 'BSOD / Diagnóstico SFC', q, res.text);
});
