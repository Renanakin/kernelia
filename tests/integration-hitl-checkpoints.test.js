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

test('Integracion A HITL 1: Congelamiento de Estado Agentico (create_hitl_checkpoint_cmd)', async () => {
  const chk = await tryDirectLocalCommand('create_hitl_checkpoint_cmd', {
    session_id: 'sess-test-99',
    tool_name: 'restart_service_ps',
    args_json: '{"service_name":"spooler"}',
    risk_level: 'R2',
    required_role: 'PowerUser'
  });

  assert.ok(chk.checkpoint_code);
  assert.ok(chk.checkpoint_code.startsWith('CHK-'));
  assert.equal(chk.status, 'pending');
  assert.equal(chk.tool_name, 'restart_service_ps');
  assert.equal(chk.risk_level, 'R2');
});

test('Integracion A HITL 2: Reanudacion de Estado Congelado (resolve_hitl_checkpoint_cmd - Approve)', async () => {
  const chk = await tryDirectLocalCommand('create_hitl_checkpoint_cmd', {
    session_id: 'sess-test-100',
    tool_name: 'restart_service_ps',
    args_json: '{"service_name":"spooler"}',
    risk_level: 'R2',
    required_role: 'PowerUser'
  });

  const res = await tryDirectLocalCommand('resolve_hitl_checkpoint_cmd', {
    checkpoint_code: chk.checkpoint_code,
    action: 'approve',
    password: null
  });

  assert.equal(res.checkpoint_code, chk.checkpoint_code);
  assert.equal(res.status, 'approved');
  assert.equal(res.executed, true);
  assert.match(res.message, /reanudado exitosamente/i);
});

test('Integracion A HITL 3: Rechazo y Cancelacion de Operacion (resolve_hitl_checkpoint_cmd - Reject)', async () => {
  const chk = await tryDirectLocalCommand('create_hitl_checkpoint_cmd', {
    session_id: 'sess-test-101',
    tool_name: 'system_poweroff_ps',
    args_json: '{}',
    risk_level: 'R3',
    required_role: 'MegaBoss'
  });

  const res = await tryDirectLocalCommand('resolve_hitl_checkpoint_cmd', {
    checkpoint_code: chk.checkpoint_code,
    action: 'reject',
    password: null
  });

  assert.equal(res.checkpoint_code, chk.checkpoint_code);
  assert.equal(res.status, 'rejected');
  assert.equal(res.executed, false);
  assert.match(res.message, /rechazada/i);
});
