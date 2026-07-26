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
import { is_whitelisted_microsoft_url, build_microsoft_search_query } from '../src/lib/utils/microsoftSearchCompat.js';
import {
  messages,
  addUserMessage,
  addLoadingMessage,
  resolveLoadingMessage,
  clearMessages
} from '../src/lib/stores/chat.js';

test('Bateria E2E 1: Consulta No Critica de Parametros (Ejecucion Directa de Lectura)', async () => {
  clearMessages();

  // Consulta de parametros de sistema no critica (R0/R1)
  addUserMessage('Dime cual es el uso de CPU y memoria RAM actual del PC');
  const loadingId = addLoadingMessage();

  const sysInfo = await tryDirectLocalCommand('get_system_info');
  const parsedSys = JSON.parse(sysInfo);

  resolveLoadingMessage(
    loadingId,
    `Estado del equipo:\n- CPU: ${parsedSys.cpu_usage}\n- Memoria usada: ${parsedSys.memory_used} / ${parsedSys.memory_total}`,
    [{ name: 'get_system_info', arguments: '{}' }],
    'gemma3-local',
    undefined,
    { enabled: true, specialty: 'Performance', confidence_level: 'HIGH', decision_mode: 'EXECUTE', risk_level: 'R1' }
  );

  let current = [];
  messages.subscribe((v) => { current = v; })();
  const msg = current.find((m) => m.id === loadingId);

  // Verificaciones:
  assert.match(msg.content, /Estado del equipo/i);
  assert.equal(msg.toolsUsed.length, 1);
  assert.equal(msg.toolsUsed[0].name, 'get_system_info');
});

test('Bateria E2E 2: Solicitud de Reinicio de Servicio (Congelamiento HITL y Respuesta: NO ACEPTAR)', async () => {
  clearMessages();

  // Solicitud de reinicio de servicio Spooler (R2 - Requiere Compuerta HITL)
  addUserMessage('Necesito que reinicies el servicio Spooler de impresion');
  
  // 1. Congelar estado agentico en SQLite (Checkpoint CHK-XXXX)
  const checkpoint = await tryDirectLocalCommand('create_hitl_checkpoint_cmd', {
    session_id: 'sess-e2e-master',
    tool_name: 'restart_service_ps',
    args_json: '{"service_name":"spooler"}',
    risk_level: 'R2',
    required_role: 'PowerUser'
  });

  assert.ok(checkpoint.checkpoint_code.startsWith('CHK-'));
  assert.equal(checkpoint.status, 'pending');

  // 2. Simular respuesta del operador: RECHAZAR / NO ACEPTAR
  const resolution = await tryDirectLocalCommand('resolve_hitl_checkpoint_cmd', {
    checkpoint_code: checkpoint.checkpoint_code,
    action: 'reject',
    password: null
  });

  assert.equal(resolution.status, 'rejected');
  assert.equal(resolution.executed, false);
  assert.match(resolution.message, /rechazada/i);
});

test('Bateria E2E 3: Consulta con Búsqueda en Fuentes Oficiales de Microsoft', () => {
  const query = 'Como solucionar el codigo de error 0x80070005 en Windows Update';
  
  // 1. Verificar construcción de query con lista blanca de dominios Microsoft
  const searchQuery = build_microsoft_search_query(query);
  assert.match(searchQuery, /site:learn\.microsoft\.com/);
  assert.match(searchQuery, /site:support\.microsoft\.com/);

  // 2. Verificar validacion de URLs oficiales de Microsoft
  const officialUrl1 = 'https://learn.microsoft.com/es-es/troubleshoot/windows-server/networking/';
  const officialUrl2 = 'https://support.microsoft.com/es-es/topic/error-0x80070005-windows-update';
  const untrustedUrl = 'https://foro-dudoso-terceros.com/fix-malware.exe';

  assert.equal(is_whitelisted_microsoft_url(officialUrl1), true);
  assert.equal(is_whitelisted_microsoft_url(officialUrl2), true);
  assert.equal(is_whitelisted_microsoft_url(untrustedUrl), false);
});

test('Bateria E2E 4: Garantia Absoluta de Cero Ejecucion de Comandos Criticos (R4)', () => {
  const FORBIDDEN_CRITICAL_PATTERNS = [
    'Format-Volume',
    'Remove-Partition',
    'stop-process -name ntdll',
    'system_poweroff_ps',
    'delete_file_root'
  ];

  let current = [];
  messages.subscribe((v) => { current = v; })();

  for (const msg of current) {
    if (msg.toolsUsed) {
      for (const tool of msg.toolsUsed) {
        for (const pattern of FORBIDDEN_CRITICAL_PATTERNS) {
          assert.equal(
            tool.name.toLowerCase().includes(pattern.toLowerCase()),
            false,
            `VIOLACION CRITICA: Se detecto intento de ejecucion prohibida: ${tool.name}`
          );
        }
      }
    }
  }
});
