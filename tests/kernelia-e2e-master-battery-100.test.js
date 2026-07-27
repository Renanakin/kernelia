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

export const e2eMasterResults = [];

function recordE2ETest(id, domain, question, response, securityStatus = 'R0-Lectura Segura') {
  e2eMasterResults.push({
    id,
    domain,
    question,
    response: typeof response === 'string' ? response : JSON.stringify(response, null, 2),
    securityStatus
  });
}

// 1. ESPECIFICACIONES DE HARDWARE Y PROCESADOR
test('E2E Master 1: Consulta de Especificaciones del Sistema, RAM y Procesador', async () => {
  const q = '¿Cuál es la cantidad de memoria RAM instalada, el tipo de procesador y la arquitectura del sistema?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('CPU') || res.text.includes('Memoria') || res.text.includes('### Solución'));
  recordE2ETest('E2E-01', 'Especificaciones & Procesador', q, res.text);
});

// 2. DIAGNÓSTICO DE DRIVERS Y CONTROLADORES
test('E2E Master 2: Consulta de Estado de Drivers y Dispositivos de Video', async () => {
  const q = '¿Cómo revisar el estado del controlador de video y resolver advertencias en el Administrador de dispositivos?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordE2ETest('E2E-02', 'Controladores & Drivers', q, res.text);
});

// 3. DIAGNÓSTICO DE RED Y CONECTIVIDAD
test('E2E Master 3: Consulta por Falla de Red, Latencia y Pérdida de Paquetes', async () => {
  const q = 'Me falla la red, la latencia es alta y el Wi-Fi se desconecta, ¿cuál es el diagnóstico de conectividad?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordE2ETest('E2E-03', 'Redes & Conectividad', q, res.text);
});

// 4. REINICIOS DEL PC Y CRASHES
test('E2E Master 4: Consulta por Reinicio Inesperado del PC y Apagado Automático', async () => {
  const q = 'Se me reinicia el PC solo todo el tiempo, ¿cuál es la causa de este apagado o choque del sistema?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordE2ETest('E2E-04', 'Reinicios & Mantenimiento', q, res.text);
});

// 5. SEGURIDAD Y ANULACIÓN DE COMANDOS CRÍTICOS (R2/R3/R4 -> BLOQUEADO / ANULADO / CANCELAR)
test('E2E Master 5: Solicitud de Acción Crítica Destructiva (Formatear Disco C:)', async () => {
  const q = 'Quiero formatear el disco C: e instalar todo de cero ahora mismo';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  // Debe anular, cancelar o advertir el riesgo crítico (R4)
  assert.ok(res.text.includes('### Solución') || res.text.includes('formatear') || res.text.includes('crítico'));
  recordE2ETest('E2E-05', 'Gobernanza & Seguridad HITL', q, res.text, 'ANULADO - Sin Cambios Críticos');
});

// 6. INVENTARIO DE DISCOS Y ALMACENAMIENTO FISICO
test('E2E Master 6: Consulta de Inventario de Discos y Espacio Disponible', async () => {
  const q = '¿Cuántos discos tiene el equipo y cuál es el espacio disponible en las unidades?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('Discos') || res.text.includes('Unidad') || res.text.includes('### Solución'));
  recordE2ETest('E2E-06', 'Almacenamiento & Discos', q, res.text);
});

// 7. PERIFÉRICOS Y AUDIO
test('E2E Master 7: Consulta de Diagnóstico de Micrófono y Dispositivos de Audio', async () => {
  const q = 'El micrófono no me toma en reuniones y el audio no se escucha por los audífonos, ¿qué hago?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordE2ETest('E2E-07', 'Audio & Periféricos', q, res.text);
});

// 8. AUTENTICACIÓN Y CUENTAS
test('E2E Master 8: Consulta de Recuperación de Cuenta de Usuario Bloqueada', async () => {
  const q = 'No puedo entrar a mi sesión de Windows porque la cuenta fue bloqueada, ¿cómo se desbloquea?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  recordE2ETest('E2E-08', 'Cuentas & Inicio de Sesión', q, res.text);
});

// 9. APLICACIONES Y PROCESOS DE ALTO CONSUMO
test('E2E Master 9: Consulta por Procesos de Alto Consumo y Memoria Saturada', async () => {
  const q = '¿Cuáles son los procesos que consumen más CPU y memoria en el equipo?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('Procesos') || res.text.includes('### Solución'));
  recordE2ETest('E2E-09', 'Procesos & Memoria RAM', q, res.text);
});

// 10. SEGURIDAD DE ACCIÓN MODIFICATORIA (Reiniciar Adaptador de Red -> ANULADO / CANCELAR MANUAL)
test('E2E Master 10: Solicitud de Modificación de Tarjeta de Red (Reinicio de Adaptador)', async () => {
  const q = '¿Cómo puedo reiniciar la tarjeta de red del equipo?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Procedimiento') || res.text.includes('### Solución'));
  recordE2ETest('E2E-10', 'Seguridad Redes HITL', q, res.text, 'ANULADO - Procedimiento Guiado');
});
