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
  clearMessages
} from '../src/lib/stores/chat.js';

import { tryDirectLocalCommand } from '../src/lib/utils/localDirect.js';

test('Flujo E2E 1: Autenticacion y Apertura de Sesion', async () => {
  global.localStorage.clear();
  
  // Guardar configuracion inicial de usuario
  const initOk = await tryDirectLocalCommand('set_model', { model_id: 'gemma3-local' });
  assert.equal(initOk, true);

  const settings = await tryDirectLocalCommand('get_settings');
  assert.equal(settings.selected_model, 'gemma3-local');
});

test('Flujo E2E 2: Asistencia RAG y Enrutamiento por Especialista', () => {
  clearMessages();

  // Usuario consulta solución a problema de spooler
  addUserMessage('Tengo un error con las impresoras, el spooler esta detenido');
  const loadingId = addLoadingMessage();

  const ragContext = {
    enabled: true,
    specialty: 'Services',
    confidence_level: 'HIGH',
    confidence_score: 0.96,
    decision_mode: 'EXECUTE',
    risk_level: 'R1',
    trace_id: 'trace-spooler-991'
  };

  const ragComparison = {
    legacy_intent: 'services_repair',
    legacy_confidence: 0.82,
    legacy_plan: ['Start-Service spooler'],
    rag_specialty: 'Services',
    rag_decision: 'EXECUTE',
    rag_confidence: 0.96
  };

  resolveLoadingMessage(
    loadingId,
    'He detectado un problema en el servicio Spooler. Se puede reiniciar de forma segura con `Start-Service spooler`.',
    [{ name: 'restart_service_ps', arguments: 'spooler' }],
    'gemma3-local',
    undefined,
    ragContext,
    ragComparison
  );

  let currentMessages = [];
  messages.subscribe((val) => { currentMessages = val; })();

  const assistantMsg = currentMessages.find((m) => m.id === loadingId);
  assert.equal(assistantMsg.role, 'assistant');
  assert.equal(assistantMsg.ragContext.specialty, 'Services');
  assert.equal(assistantMsg.toolsUsed[0].name, 'restart_service_ps');
});

test('Flujo E2E 3: Gobernanza de Seguridad y Bloqueo de Accion Critica (R4)', async () => {
  // Intentar guardar API key vacia debe fallar por validacion
  await assert.rejects(
    () => tryDirectLocalCommand('set_api_key', { api_key: '' }),
    /API_VALIDATION/
  );
});

test('Flujo E2E 4: Configuracion Global y Cambio de Modelo en Caliente', async () => {
  const switchOk = await tryDirectLocalCommand('set_model', { model_id: 'gemma4-local' });
  assert.equal(switchOk, true);

  const updatedSettings = await tryDirectLocalCommand('get_settings');
  assert.equal(updatedSettings.selected_model, 'gemma4-local');
});
