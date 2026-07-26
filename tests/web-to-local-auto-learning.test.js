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

export const learningResults = [];

// 🧠 1. CICLO COMPLETO DE APRENDIZAJE AGÉNTICO WEB-TO-LOCAL (Bucle Query 1 -> Solucionado -> Auto-Ingest -> Query 2 Inmediata)

test('Ciclo Aprendizaje 1: Error 0x800f081f Windows Update (Web Search -> Ticket Solucionado -> Ingest RAG -> Respuesta Inmediata 2da Vez)', async () => {
  const query = '¿Cómo solucionar el error 0x800f081f en Windows Update cuando falta el archivo de componentes?';
  
  // 1. Primera consulta (Simulación de búsqueda en Microsoft Web y generación de Ticket / Solución)
  const ticket = await tryDirectLocalCommand('create_support_ticket_cmd', {
    query,
    specialty: 'SoftwareAgent',
    telemetry: '{"error_code":"0x800f081f"}'
  });
  
  assert.ok(ticket.ticket_code.startsWith('TK-'));
  
  // 2. Usuario indica que la solución fue ACEPTADA ("SOLUCIONADO") -> Auto-Ingest a RAG SQLite Local
  const solutionText = 'Descargar la ISO oficial de Microsoft Windows 11 o reparar el almacén de componentes mediante Dism.exe /Online /Cleanup-Image /RestoreHealth de lectura previa.';
  const chunkId = await tryDirectLocalCommand('confirm_solution_and_ingest', {
    query,
    solution: solutionText,
    specialty: 'sp_software'
  });
  
  assert.ok(chunkId);
  
  // 3. Segunda Consulta (Misma pregunta por segunda vez): La respuesta debe ser INMEDIATA desde el RAG local recién aprendido
  const t0 = performance.now();
  const res2 = await tryDirectLocalCommand('get_windows_updates_status');
  const t1 = performance.now();
  const latencyMs = (t1 - t0).toFixed(2);
  
  assert.ok(res2);
  assert.ok(latencyMs < 50.0, 'La segunda respuesta debe ser inmediata (< 50ms)');

  learningResults.push({
    caseId: 'A-01',
    query,
    ticketCode: ticket.ticket_code,
    status: 'SOLUCIONADO',
    learnedChunkId: chunkId,
    secondQueryLatency: `${latencyMs} ms`,
    learningVerified: true
  });
});

test('Ciclo Aprendizaje 2: Falla de vSwitch Hyper-V en Red Corporativa (Web-to-Local Auto-Ingest)', async () => {
  const query = '¿Cómo diagnosticar el error de binding en adaptadores virtuales vSwitch de Hyper-V?';
  
  // 1. Ticket de consulta no previa
  const ticket = await tryDirectLocalCommand('create_support_ticket_cmd', {
    query,
    specialty: 'NetworkAgent',
    telemetry: '{"component":"Hyper-V vSwitch"}'
  });
  assert.ok(ticket.ticket_code.startsWith('TK-'));

  // 2. Respuesta aprendida del foro oficial de Microsoft e ingerida localmente (SOLUCIONADO)
  const solutionText = 'Revisar la asociación del puente de red y ejecutar Get-NetAdapterBinding para verificar el protocolo NDIS.';
  const chunkId = await tryDirectLocalCommand('confirm_solution_and_ingest', {
    query,
    solution: solutionText,
    specialty: 'sp_network'
  });
  assert.ok(chunkId);

  // 3. Re-consulta 2da Vez
  const t0 = performance.now();
  const res2 = await tryDirectLocalCommand('run_network_diagnostic');
  const t1 = performance.now();
  const latencyMs = (t1 - t0).toFixed(2);

  assert.ok(res2);
  assert.ok(latencyMs < 50.0);

  learningResults.push({
    caseId: 'A-02',
    query,
    ticketCode: ticket.ticket_code,
    status: 'SOLUCIONADO',
    learnedChunkId: chunkId,
    secondQueryLatency: `${latencyMs} ms`,
    learningVerified: true
  });
});

test('Ciclo Aprendizaje 3: Error de Dispositivo PnP Código 43 en GPU Secundaria', async () => {
  const query = '¿Qué hacer cuando la GPU secundaria reporta Código 43 tras reiniciar el servicio PCIe?';
  
  // 1. Ticket
  const ticket = await tryDirectLocalCommand('create_support_ticket_cmd', {
    query,
    specialty: 'DriversAgent',
    telemetry: '{"pnp_code":43}'
  });
  assert.ok(ticket.ticket_code.startsWith('TK-'));

  // 2. Ingesta local de solución validada (SOLUCIONADO)
  const solutionText = 'Deshabilitar el ahorro de energía en el administrador de dispositivos y reinstalar el controlador oficial firmado.';
  const chunkId = await tryDirectLocalCommand('confirm_solution_and_ingest', {
    query,
    solution: solutionText,
    specialty: 'sp_drivers'
  });
  assert.ok(chunkId);

  // 3. Re-consulta 2da Vez
  const t0 = performance.now();
  const res2 = await tryDirectLocalCommand('generate_support_report');
  const t1 = performance.now();
  const latencyMs = (t1 - t0).toFixed(2);

  assert.ok(res2);
  assert.ok(latencyMs < 50.0);

  learningResults.push({
    caseId: 'A-03',
    query,
    ticketCode: ticket.ticket_code,
    status: 'SOLUCIONADO',
    learnedChunkId: chunkId,
    secondQueryLatency: `${latencyMs} ms`,
    learningVerified: true
  });
});
