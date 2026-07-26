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

export const batteryResults = [];

function recordTest(expert, type, question, response, toolUsed) {
  batteryResults.push({
    expert,
    type,
    question,
    response: typeof response === 'string' ? response : JSON.stringify(response, null, 2),
    toolUsed
  });
}

// 🌐 1. ESPECIALISTA REDES (NetworkAgent)
test('Health Bateria 1.1 (Monosilabo): Wi-Fi', async () => {
  const q = 'Wi-Fi';
  const res = await tryDirectLocalCommand('run_network_diagnostic');
  assert.ok(res);
  recordTest('NetworkAgent', 'Monosílabo', q, res, 'run_network_diagnostic');
});

test('Health Bateria 1.2 (Estandard): ¿Cual es la IP local y la salud de red?', async () => {
  const q = '¿Cuál es la IP local y la salud de la red?';
  const res = await tryDirectLocalCommand('run_network_diagnostic');
  assert.ok(res);
  recordTest('NetworkAgent', 'Estándar', q, res, 'run_network_diagnostic');
});

test('Health Bateria 1.3 (Compleja): Diagnosticar latencia de gateway y resolucion DNS', async () => {
  const q = 'Analizar si la pérdida de paquetes en el gateway y la falta de resolución DNS están bloqueando el acceso corporativo.';
  const res = await tryDirectLocalCommand('run_network_diagnostic');
  assert.ok(res);
  recordTest('NetworkAgent', 'Compleja', q, res, 'run_network_diagnostic');
});

// ⚙️ 2. ESPECIALISTA SERVICIOS (ServicesAgent)
test('Health Bateria 2.1 (Monosilabo): Spooler', async () => {
  const q = 'Spooler';
  const res = await tryDirectLocalCommand('list_running_services');
  assert.ok(res);
  recordTest('ServicesAgent', 'Monosílabo', q, res, 'list_running_services');
});

test('Health Bateria 2.2 (Estandard): Estado de servicios de impresion y Windows Update', async () => {
  const q = '¿Cuáles son los servicios críticos de impresión y Windows Update ejecutándose actualmente?';
  const res = await tryDirectLocalCommand('list_running_services');
  assert.ok(res);
  recordTest('ServicesAgent', 'Estándar', q, res, 'list_running_services');
});

test('Health Bateria 2.3 (Compleja): Verificar si la detencion de BITS e IIS afecta la cola de impresion', async () => {
  const q = 'Verificar si la detención inesperada del servicio BITS y la cola de impresión está bloqueando las tareas en segundo plano.';
  const res = await tryDirectLocalCommand('list_running_services');
  assert.ok(res);
  recordTest('ServicesAgent', 'Compleja', q, res, 'list_running_services');
});

// 📊 3. ESPECIALISTA RENDIMIENTO Y PROCESOS (PerformanceAgent & ProcessAgent)
test('Health Bateria 3.1 (Monosilabo): CPU', async () => {
  const q = 'CPU';
  const res = await tryDirectLocalCommand('get_system_info');
  assert.ok(res);
  recordTest('PerformanceAgent', 'Monosílabo', q, res, 'get_system_info');
});

test('Health Bateria 3.2 (Estandard): ¿Cual es el consumo actual de CPU y memoria RAM?', async () => {
  const q = '¿Cuál es el porcentaje de uso actual de CPU y memoria RAM en el sistema?';
  const res = await tryDirectLocalCommand('get_system_info');
  assert.ok(res);
  recordTest('PerformanceAgent', 'Estándar', q, res, 'get_system_info');
});

test('Health Bateria 3.3 (Compleja): Analisis de consumo de procesos desbordados y Kernel-Power Event 41', async () => {
  const q = 'Analizar si el consumo de memoria RAM desbordado por el explorador de archivos está provocando inestabilidad y eventos Kernel-Power.';
  const res = await tryDirectLocalCommand('list_processes', { sort_by: 'memory', limit: 5 });
  assert.ok(res);
  recordTest('ProcessAgent', 'Compleja', q, res, 'list_processes');
});

// 🔌 4. ESPECIALISTA CONTROLADORES Y DISPOSITIVOS (DriversAgent)
test('Health Bateria 4.1 (Monosilabo): GPU', async () => {
  const q = 'GPU';
  const res = await tryDirectLocalCommand('generate_support_report');
  assert.ok(res);
  recordTest('DriversAgent', 'Monosílabo', q, res, 'generate_support_report');
});

test('Health Bateria 4.2 (Estandard): Inspeccionar dispositivos con error Codigo 43', async () => {
  const q = '¿Hay algún dispositivo gráfico o USB reportando código 43 en el administrador de dispositivos?';
  const res = await tryDirectLocalCommand('generate_support_report');
  assert.ok(res);
  recordTest('DriversAgent', 'Estándar', q, res, 'generate_support_report');
});

test('Health Bateria 4.3 (Compleja): Diagnostico de inestabilidad en drivers de audio y video tras actualizacion', async () => {
  const q = 'Verificar si la degradación de resolución en pantalla y la falta de audio se deben a controladores PnP corruptos.';
  const res = await tryDirectLocalCommand('generate_support_report');
  assert.ok(res);
  recordTest('DriversAgent', 'Compleja', q, res, 'generate_support_report');
});

// 🧹 5. ESPECIALISTA MANTENIMIENTO E INTEGRIDAD (MaintenanceAgent)
test('Health Bateria 5.1 (Monosilabo): SFC', async () => {
  const q = 'SFC';
  const res = await tryDirectLocalCommand('get_system_info');
  assert.ok(res);
  recordTest('MaintenanceAgent', 'Monosílabo', q, res, 'get_system_info');
});

test('Health Bateria 5.2 (Estandard): Verificacion de integridad de archivos del sistema', async () => {
  const q = '¿Cuál es el estado de salud de los archivos principales del sistema operativo?';
  const res = await tryDirectLocalCommand('get_system_info');
  assert.ok(res);
  recordTest('MaintenanceAgent', 'Estándar', q, res, 'get_system_info');
});

test('Health Bateria 5.3 (Compleja): Analisis de salud de disco, archivos temporales y temporales basura acumulados', async () => {
  const q = 'Evaluar la presencia de archivos temporales corruptos y la necesidad de optimización de integridad del volumen C:';
  const res = await tryDirectLocalCommand('get_system_info');
  assert.ok(res);
  recordTest('MaintenanceAgent', 'Compleja', q, res, 'get_system_info');
});

// 📁 6. ESPECIALISTA ALMACENAMIENTO Y ARCHIVOS (FilesystemAgent)
test('Health Bateria 6.1 (Monosilabo): Espacio', async () => {
  const q = 'Espacio';
  const res = await tryDirectLocalCommand('get_storage_summary');
  assert.ok(res);
  recordTest('FilesystemAgent', 'Monosílabo', q, res, 'get_storage_summary');
});

test('Health Bateria 6.2 (Estandard): ¿Cuanto espacio libre queda en el disco C?', async () => {
  const q = '¿Cuánto espacio libre y total queda disponible en la partición C:?';
  const res = await tryDirectLocalCommand('get_storage_summary');
  assert.ok(res);
  recordTest('FilesystemAgent', 'Estándar', q, res, 'get_storage_summary');
});

test('Health Bateria 6.3 (Compleja): Evaluacion de llenado de disco principal y permisos NTFS en carpetas de usuario', async () => {
  const q = 'Inspeccionar si el llenado de la unidad principal C: está bloqueando los permisos de escritura en la carpeta Escritorio.';
  const res = await tryDirectLocalCommand('get_storage_summary');
  assert.ok(res);
  recordTest('FilesystemAgent', 'Compleja', q, res, 'get_storage_summary');
});

// 📦 7. ESPECIALISTA SOFTWARE Y ACTUALIZACIONES (SoftwareAgent)
test('Health Bateria 7.1 (Monosilabo): Winget', async () => {
  const q = 'Winget';
  const res = await tryDirectLocalCommand('get_windows_updates_status');
  assert.ok(res);
  recordTest('SoftwareAgent', 'Monosílabo', q, res, 'get_windows_updates_status');
});

test('Health Bateria 7.2 (Estandard): ¿Hay actualizaciones de seguridad pendientes por instalar?', async () => {
  const q = '¿Cuál es el estado del servicio de Windows Update y hay parches pendientes?';
  const res = await tryDirectLocalCommand('get_windows_updates_status');
  assert.ok(res);
  recordTest('SoftwareAgent', 'Estándar', q, res, 'get_windows_updates_status');
});

test('Health Bateria 7.3 (Compleja): Verificacion de paquetes desactualizados e inventario de aplicaciones corporativas', async () => {
  const q = 'Analizar si la acumulación de parches de seguridad pendientes en Windows Update afecta la estabilidad de aplicaciones corporativas.';
  const res = await tryDirectLocalCommand('get_windows_updates_status');
  assert.ok(res);
  recordTest('SoftwareAgent', 'Compleja', q, res, 'get_windows_updates_status');
});
