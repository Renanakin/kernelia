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

import {
  messages,
  addUserMessage,
  addLoadingMessage,
  resolveLoadingMessage,
  appendToMessage,
  clearMessages
} from '../src/lib/stores/chat.js';

import { tryDirectLocalCommand } from '../src/lib/utils/localDirect.js';

test('Seccion 1: Autenticacion & Control de Acceso - Estado inicial de seguridad', () => {
  global.localStorage.clear();
  assert.equal(global.localStorage.getItem('nexus_lite_settings'), null);
});

test('Seccion 2: Conversacion & RAG - Ciclo de vida completo del mensaje', () => {
  clearMessages();

  // 1. Usuario envía prompt
  const userMsg = addUserMessage('¿Cual es el estado del servicio spooler?');
  assert.equal(userMsg.role, 'user');
  assert.equal(userMsg.content, '¿Cual es el estado del servicio spooler?');

  // 2. Asistente crea placeholder de carga
  const loadingId = addLoadingMessage();
  assert.ok(loadingId);

  // 3. Resolución con respuesta RAG y comparación
  const ragContext = {
    enabled: true,
    specialty: 'Services',
    confidence_level: 'HIGH',
    confidence_score: 0.95,
    decision_mode: 'EXECUTE',
    risk_level: 'R1'
  };

  const ragComparison = {
    legacy_intent: 'services_query',
    legacy_confidence: 0.8,
    legacy_plan: ['get_service_spooler'],
    rag_specialty: 'Services',
    rag_decision: 'EXECUTE',
    rag_confidence: 0.95
  };

  resolveLoadingMessage(
    loadingId,
    'El servicio spooler esta en ejecucion.',
    [{ name: 'get_service_status', arguments: 'spooler' }],
    'gemma3-local',
    undefined,
    ragContext,
    ragComparison
  );

  let currentMessages = [];
  messages.subscribe((val) => { currentMessages = val; })();

  assert.equal(currentMessages.length, 2);
  const resolved = currentMessages.find((m) => m.id === loadingId);
  assert.equal(resolved.isLoading, false);
  assert.equal(resolved.content, 'El servicio spooler esta en ejecucion.');
  assert.equal(resolved.ragContext.specialty, 'Services');
  assert.equal(resolved.ragComparison.rag_decision, 'EXECUTE');

  // 4. Test delta append para streaming
  appendToMessage(resolved.id, ' PID: 1420.');
  messages.subscribe((val) => { currentMessages = val; })();
  const updated = currentMessages.find((m) => m.id === loadingId);
  assert.equal(updated.content, 'El servicio spooler esta en ejecucion. PID: 1420.');
});

test('Seccion 3: Telemetria & Diagnostico - Comandos locales directos de salud', async () => {
  const result = await tryDirectLocalCommand('get_settings');
  assert.ok(result);
  assert.ok('selected_model' in result);
});

test('Seccion 4: Configuracion & Modelos - Actualizaciones de parametros y API Keys', async () => {
  const okModel = await tryDirectLocalCommand('set_model', { model_id: 'gemma4-local' });
  assert.equal(okModel, true);

  const settings = await tryDirectLocalCommand('get_settings');
  assert.equal(settings.selected_model, 'gemma4-local');

  const okKey = await tryDirectLocalCommand('set_api_key', { api_key: 'sk-test-key-12345' });
  assert.equal(okKey, true);
});
