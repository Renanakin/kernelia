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

export const batteryV2Results = [];

function recordV2Test(id, category, question, response) {
  batteryV2Results.push({
    id,
    category,
    question,
    response: typeof response === 'string' ? response : JSON.stringify(response, null, 2)
  });
}

// 🛡️ 1. SEGURIDAD Y ACCESO (Security & Auth)
test('Bateria V2 - Caso 1: Error de Remediation CredSSP en Conexion Escritorio Remoto RDP', async () => {
  const q = '¿Cómo solucionar el error de autenticación CredSSP en conexión RDP cuando falta la actualización de cifrado en Windows 11?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text);
  assert.ok(res.text.includes('### Solución'), 'Debe incluir la sección ### Solución');
  assert.ok(res.text.includes('### Consejos y Recomendaciones'), 'Debe incluir la sección ### Consejos y Recomendaciones');
  recordV2Test('V2-01', 'Seguridad & Autenticación', q, res.text);
});

// 🌐 2. REDES Y ADAPTADORES VIRTUALES (Network & DNS)
test('Bateria V2 - Caso 2: Falla de Resolucion de Nombres LLMNR y Adaptador VPN', async () => {
  const q = '¿Cómo resolver la falla de resolución de nombres de dominio locales en adaptadores VPN mediante la directiva LLMNR / NetBIOS?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordV2Test('V2-02', 'Redes & Adaptadores VPN', q, res.text);
});

// 📁 3. ALMACENAMIENTO Y SOMBRAS VSS (Storage & VSS)
test('Bateria V2 - Caso 3: Error de VSS Volume Shadow Copy Service 0x8004230f', async () => {
  const q = '¿Cómo reparar el error 0x8004230f del servicio de copias de sombra de volumen VSS en particiones NTFS/ReFS?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordV2Test('V2-03', 'Almacenamiento & VSS', q, res.text);
});

// 📊 4. RENDIMIENTO Y VISOR DE EVENTOS (Performance & EventViewer)
test('Bateria V2 - Caso 4: Evento EventViewer ID 1001 CrashDump y Fuga de Memoria', async () => {
  const q = '¿Cómo diagnosticar un cierre inesperado del sistema con evento EventViewer ID 1001 y volcado de memoria minidump?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordV2Test('V2-04', 'Rendimiento & EventViewer', q, res.text);
});

// 📦 5. SOFTWARE Y UPDATE (Software & Updates)
test('Bateria V2 - Caso 5: Error 0x80070002 en Windows Update al Instalar Parche Acumulativo', async () => {
  const q = '¿Cómo corregir el error 0x80070002 en Windows Update cuando los parches acumulativos quedan atascados en descarga?';
  const res = await tryDirectLocalCommand('send_message', { message: q });
  assert.ok(res);
  assert.ok(res.text.includes('### Solución'));
  assert.ok(res.text.includes('### Consejos y Recomendaciones'));
  recordV2Test('V2-05', 'Software & Actualizaciones', q, res.text);
});
