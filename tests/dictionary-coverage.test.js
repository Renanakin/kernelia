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

test('Diccionario Cobertura 1: Redes y Conectividad (NetworkAgent)', async () => {
  const res = await tryDirectLocalCommand('run_network_diagnostic');
  assert.ok(res);
});

test('Diccionario Cobertura 2: Servicios de Windows (ServicesAgent & Compuerta HITL R2/R3)', async () => {
  const chk = await tryDirectLocalCommand('create_hitl_checkpoint_cmd', {
    session_id: 'sess-dict-svc',
    tool_name: 'restart_service_ps',
    args_json: '{"service_name":"spooler"}',
    risk_level: 'R2',
    required_role: 'PowerUser'
  });

  assert.ok(chk.checkpoint_code.startsWith('CHK-'));
  assert.equal(chk.status, 'pending');

  const resolution = await tryDirectLocalCommand('resolve_hitl_checkpoint_cmd', {
    checkpoint_code: chk.checkpoint_code,
    action: 'approve',
    password: null
  });

  assert.equal(resolution.status, 'approved');
  assert.equal(resolution.executed, true);
});

test('Diccionario Cobertura 3: Procesos y Rendimiento (ProcessAgent & PerformanceAgent)', async () => {
  const sysInfo = await tryDirectLocalCommand('get_system_info');
  assert.ok(sysInfo);
});

test('Diccionario Cobertura 4: Controladores y Dispositivos (DriversAgent)', async () => {
  const report = await tryDirectLocalCommand('generate_support_report');
  const text = typeof report === 'string' ? report : report?.output || JSON.stringify(report);
  assert.ok(text);
});

test('Diccionario Cobertura 5: Mantenimiento e Integridad (MaintenanceAgent)', async () => {
  const sysInfo = await tryDirectLocalCommand('get_system_info');
  assert.ok(sysInfo);
});

test('Diccionario Cobertura 6: Almacenamiento y Archivos (FilesystemAgent)', async () => {
  const storage = await tryDirectLocalCommand('get_storage_summary');
  assert.ok(storage);
});

test('Diccionario Cobertura 7: Software y Actualizaciones (SoftwareAgent)', async () => {
  const updates = await tryDirectLocalCommand('get_windows_updates_status');
  assert.ok(updates);
});
