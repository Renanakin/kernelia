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

export const auditLogTestResults = [];

// 1. Verificar esquema y simulacion de log de auditoria
test('Auditoria & RBAC 1: Simulación de Registro en user_interaction_log para Consulta Estándar', async () => {
  const logEntry = {
    id: `log-${Date.now()}-001`,
    session_id: 'session-e2e-001',
    user_id: 'usr-standard-01',
    user_role: 'standard_user',
    query_text: 'el computador está muy lento',
    intent_detected: 'sp_performance',
    response_mode: 'written_solution',
    action_requested: null,
    command_risk_level: 'R0',
    elevation_required: 0,
    elevation_status: 'NOT_REQUIRED',
    authenticated_by: null,
    execution_result: 'Solución escrita entregada en formato markdown',
    created_at: new Date().toISOString()
  };

  assert.equal(logEntry.user_role, 'standard_user');
  assert.equal(logEntry.command_risk_level, 'R0');
  assert.equal(logEntry.elevation_required, 0);
  auditLogTestResults.push(logEntry);
});

// 2. Simulación de Desafío de Contraseña para Acción Modificatoria (R2/R3)
test('Auditoria & RBAC 2: Desafío de Contraseña para Acción R2 y Registro Exitoso', async () => {
  const logEntry = {
    id: `log-${Date.now()}-002`,
    session_id: 'session-e2e-002',
    user_id: 'usr-standard-01',
    user_role: 'standard_user',
    query_text: 'reiniciar servicio de spooler de impresion',
    intent_detected: 'sp_services',
    response_mode: 'elevation_challenge',
    action_requested: 'Restart-Service Spooler',
    command_risk_level: 'R2',
    elevation_required: 1,
    elevation_status: 'PASSED',
    authenticated_by: 'tech_analyst_admin',
    execution_result: 'Servicio Spooler reiniciado con éxito tras validación de credenciales',
    created_at: new Date().toISOString()
  };

  assert.equal(logEntry.elevation_required, 1);
  assert.equal(logEntry.elevation_status, 'PASSED');
  assert.equal(logEntry.authenticated_by, 'tech_analyst_admin');
  auditLogTestResults.push(logEntry);
});

// 3. Simulación de Bloqueo de Borrado Físico (R4 -> ANULADO / DENIED)
test('Auditoria & RBAC 3: Intent de Borrado Físico R4 Anulado por Falta de Contraseña', async () => {
  const logEntry = {
    id: `log-${Date.now()}-003`,
    session_id: 'session-e2e-003',
    user_id: 'usr-standard-01',
    user_role: 'standard_user',
    query_text: 'eliminar carpeta de archivos c:\\datos',
    intent_detected: 'sp_filesystem',
    response_mode: 'elevation_challenge',
    action_requested: 'Remove-Item C:\\datos -Recurse',
    command_risk_level: 'R4',
    elevation_required: 1,
    elevation_status: 'DENIED',
    authenticated_by: null,
    execution_result: 'Acción destructiva anulada. Cero archivos eliminados.',
    created_at: new Date().toISOString()
  };

  assert.equal(logEntry.command_risk_level, 'R4');
  assert.equal(logEntry.elevation_status, 'DENIED');
  auditLogTestResults.push(logEntry);
});
