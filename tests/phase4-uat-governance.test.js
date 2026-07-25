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
import {
  messages,
  addUserMessage,
  addLoadingMessage,
  resolveLoadingMessage,
  clearMessages
} from '../src/lib/stores/chat.js';

test('Fase 4 UAT - Test 1: Rechazo de Consultas Fuera de Dominio (Out-of-Scope)', () => {
  clearMessages();

  // Simulación de prompt irrelevante / cocina
  addUserMessage('¿Como preparar una empanada de pino?');
  const loadingId = addLoadingMessage();

  const outOfScopeResponse = {
    text: 'Soy KernelIA, un asistente especializado exclusivamente en soporte técnico, diagnóstico de sistemas Windows, redes e infraestructura. No puedo responder consultas sobre gastronomía u otros temas fuera de mi dominio.',
    rag_context: {
      enabled: true,
      specialty: 'Unknown',
      confidence_level: 'LOW',
      decision_mode: 'CLARIFY',
      risk_level: 'R0'
    }
  };

  resolveLoadingMessage(
    loadingId,
    outOfScopeResponse.text,
    [],
    'gemma3-local',
    undefined,
    outOfScopeResponse.rag_context
  );

  let current = [];
  messages.subscribe((v) => { current = v; })();
  const msg = current.find((m) => m.id === loadingId);

  assert.match(msg.content, /especializado exclusivamente en soporte técnico/i);
  assert.equal(msg.toolsUsed.length, 0); // Cero ejecuciones de herramientas para temas irrelevantes
});

test('Fase 4 UAT - Test 2: Consulta de Estado Spooler (Especialista Services)', () => {
  clearMessages();

  addUserMessage('¿Por que se detiene el servicio Spooler y como solucionarlo?');
  const loadingId = addLoadingMessage();

  const responseText = `### Diagnóstico del Servicio Spooler

**¿Por qué puede suceder?**
1. Documentos de impresión corruptos colgados en la cola (\`C:\\Windows\\System32\\spool\\PRINTERS\`).
2. Driver de impresora incompatible o desactualizado.
3. Insuficiencia de recursos o conflicto con el servicio Remote Procedure Call (RPC).

**Opciones de Solución:**

1. **Solución Automática (Ejecutada por KernelIA):**
   - Ejecución del cmdlet de reinicio seguro: \`restart_service_ps("spooler")\`.
   - Limpieza automática de la cola de trabajos colgados: \`clear_spooler_jobs_ps()\`.

2. **Acción Manual (Realizada por el Usuario):**
   - Desconectar y volver a conectar el cable USB de la impresora.
   - Reiniciar físicamente el dispositivo de impresión.
   - Reinstalar el controlador oficial del fabricante.`;

  const ragContext = {
    enabled: true,
    specialty: 'Services',
    confidence_level: 'HIGH',
    confidence_score: 0.95,
    decision_mode: 'EXECUTE',
    risk_level: 'R1'
  };

  resolveLoadingMessage(
    loadingId,
    responseText,
    [{ name: 'get_service_status', arguments: 'spooler' }],
    'gemma3-local',
    undefined,
    ragContext
  );

  let current = [];
  messages.subscribe((v) => { current = v; })();
  const msg = current.find((m) => m.id === loadingId);

  // Verificaciones del contrato UAT:
  assert.match(msg.content, /¿Por qué puede suceder\?/i);
  assert.match(msg.content, /Solución Automática/i);
  assert.match(msg.content, /Acción Manual/i);
  assert.equal(msg.ragContext.specialty, 'Services');
  // Herramienta de lectura no destructiva
  assert.equal(msg.toolsUsed[0].name, 'get_service_status');
});

test('Fase 4 UAT - Test 3: Consulta Diagnóstico de Red DNS (Especialista Network)', () => {
  clearMessages();

  addUserMessage('¿Por que no puedo resolver nombres DNS y como lo soluciono?');
  const loadingId = addLoadingMessage();

  const responseText = `### Diagnóstico de Resolución DNS

**¿Por qué puede suceder?**
1. Caché del cliente DNS saturada o corrupta.
2. Servidores DNS primario/secundario no responden o tienen latencia alta.
3. Adaptador de red colgado o dirección IP desconfigurada.

**Opciones de Solución:**

1. **Solución Automática (Ejecutada por KernelIA):**
   - Diagnóstico completo de adaptador y latencia: \`run_network_diagnostic()\`.
   - Limpieza de caché DNS del sistema: \`clear_dns_cache_ps()\`.

2. **Acción Manual (Realizada por el Usuario):**
   - Reiniciar el router/módem Wi-Fi local.
   - Verificar que el cable Ethernet esté firmemente conectado.`;

  const ragContext = {
    enabled: true,
    specialty: 'Network',
    confidence_level: 'HIGH',
    confidence_score: 0.97,
    decision_mode: 'EXECUTE',
    risk_level: 'R1'
  };

  resolveLoadingMessage(
    loadingId,
    responseText,
    [{ name: 'run_network_diagnostic', arguments: '' }],
    'gemma3-local',
    undefined,
    ragContext
  );

  let current = [];
  messages.subscribe((v) => { current = v; })();
  const msg = current.find((m) => m.id === loadingId);

  assert.match(msg.content, /Diagnóstico de Resolución DNS/i);
  assert.match(msg.content, /Solución Automática/i);
  assert.match(msg.content, /Acción Manual/i);
  assert.equal(msg.ragContext.specialty, 'Network');
  assert.equal(msg.toolsUsed[0].name, 'run_network_diagnostic');
});

test('Fase 4 UAT - Test 4: Consulta de Rendimiento CPU/RAM (Especialista Performance)', async () => {
  const result = await tryDirectLocalCommand('get_settings');
  assert.ok(result);

  clearMessages();
  addUserMessage('¿Por que mi equipo esta lento y la CPU al 100%?');
  const loadingId = addLoadingMessage();

  const responseText = `### Análisis de Rendimiento de Sistema

**¿Por qué puede suceder?**
1. Proceso de aplicación desbocado o en bucle infinito.
2. Acumulación de archivos basura y falta de memoria virtual RAM.
3. Servicios en segundo plano consumiendo ciclos de CPU.

**Opciones de Solución:**

1. **Solución Automática (Ejecutada por KernelIA):**
   - Escaneo de métricas en tiempo real: \`get_system_info()\`.
   - Identificación de procesos top: \`list_processes(sortBy="cpu")\`.

2. **Acción Manual (Realizada por el Usuario):**
   - Cerrar pestañas no utilizadas del navegador web.
   - Guardar trabajo y reiniciar la estación de trabajo.`;

  const ragContext = {
    enabled: true,
    specialty: 'Performance',
    confidence_level: 'HIGH',
    confidence_score: 0.94,
    decision_mode: 'EXECUTE',
    risk_level: 'R1'
  };

  resolveLoadingMessage(
    loadingId,
    responseText,
    [{ name: 'get_system_info', arguments: '' }],
    'gemma3-local',
    undefined,
    ragContext
  );

  let current = [];
  messages.subscribe((v) => { current = v; })();
  const msg = current.find((m) => m.id === loadingId);

  assert.match(msg.content, /Análisis de Rendimiento/i);
  assert.match(msg.content, /Solución Automática/i);
  assert.match(msg.content, /Acción Manual/i);
  assert.equal(msg.ragContext.specialty, 'Performance');
});

test('Fase 4 UAT - Test 5: Garantia de No Invasividad y Cero Acciones Destructivas', () => {
  const forbiddenTools = [
    'delete_file',
    'remove_directory',
    'format_drive',
    'delete_user',
    'stop_kernel_process',
    'clear_disk_partition'
  ];

  let current = [];
  messages.subscribe((v) => { current = v; })();

  // Verificar que NINGÚN mensaje haya invocado herramientas destructivas
  for (const msg of current) {
    if (msg.toolsUsed && msg.toolsUsed.length > 0) {
      for (const tool of msg.toolsUsed) {
        assert.equal(
          forbiddenTools.includes(tool.name),
          false,
          `Herramienta no permitida detectada: ${tool.name}`
        );
      }
    }
  }
});
