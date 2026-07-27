const LOCAL_API_BASE_DIRECT = 'http://localhost:11434/v1/chat/completions';
const LOCAL_API_BASE_FALLBACK = '/api/local-chat';
const STORAGE_KEY_MODEL = 'kernelia_local_model';
const STORAGE_KEY_AUTH = 'kernelia_local_auth';
const LOCAL_CHAT_TIMEOUT_MS = 120000;
const LOCAL_CHAT_RETRIES = 0;

const DEFAULT_USERS = [
  { username: 'superadmin', password: 'KernelIA!Super2026', profile: 'superusuario' },
  { username: 'soporte1', password: 'KernelIA!Support2026', profile: 'soporte1' },
  { username: 'tecnico', password: 'KernelIA!Tech2026', profile: 'tecnico' },
];

function normalizeText(value) {
  return String(value || '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9\s]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function containsAny(text, needles) {
  return needles.some((needle) => text.includes(needle));
}

function safeJsonParse(value, fallback = null) {
  if (typeof value !== 'string') return fallback;
  try {
    return JSON.parse(value);
  } catch {
    return fallback;
  }
}

function readBrowserSnapshot() {
  const snapshot = {
    source: 'browser-direct',
    user_agent: typeof navigator !== 'undefined' ? navigator.userAgent : 'n/a',
    platform: typeof navigator !== 'undefined' ? navigator.platform : 'n/a',
    language: typeof navigator !== 'undefined' ? navigator.language : 'n/a',
    online: typeof navigator !== 'undefined' ? navigator.onLine : true,
    screen:
      typeof screen !== 'undefined'
        ? `${screen.width}x${screen.height}`
        : 'n/a',
    viewport:
      typeof window !== 'undefined'
        ? `${window.innerWidth || 0}x${window.innerHeight || 0}`
        : 'n/a',
    disks: [],
  };

  if (typeof performance !== 'undefined' && performance.memory) {
    snapshot.memory = {
      used_js_heap_size: performance.memory.usedJSHeapSize || 0,
      total_js_heap_size: performance.memory.totalJSHeapSize || 0,
      js_heap_size_limit: performance.memory.jsHeapSizeLimit || 0,
    };
  }

  return snapshot;
}

function isSupportIntent(text) {
  return containsAny(text, [
    'estado actual del equipo',
    'estado del equipo',
    'estado del sistema',
    'health completo',
    'salud completa',
    'salud del equipo',
    'salud del sistema',
    'no tengo internet',
    'sin internet',
    'red',
    'internet',
    'ip del equipo',
    'cual es la ip',
    'cual es la ip del equipo',
    'direccion ip',
    'ip local',
    'dns',
    'wifi',
    'latencia',
    'procesos',
    'recursos',
    'disco',
    'discos',
    'almacenamiento',
    'archivos',
    'escritorio',
    'actualizaciones',
    'winget',
    'reporte tecnico',
    'soporte',
  ]);
}

function formatSystemInfoSummary(info) {
  const cpu = info?.cpu_usage ?? 'n/a';
  const memoryUsed = info?.memory_used ?? 0;
  const memoryTotal = info?.memory_total ?? 0;
  const disks = Array.isArray(info?.disks) ? info.disks : [];
  const browserNote = info?.source === 'browser-direct'
    ? 'Modo web: sin acceso directo al sistema operativo.'
    : 'Acceso local disponible.';

  return `**Estado detectado**

- CPU: ${cpu}
- Memoria: ${memoryUsed} / ${memoryTotal}
- Discos visibles: ${disks.length}
- ${browserNote}

**Recomendacion**

Abre KernelIA con runtime Tauri si necesitas datos reales del sistema.`;
}

function formatNetworkSummary(data) {
  const connectivity = data?.connectivity?.google_ping?.latency || 'n/a';
  const status = data?.status || 'n/a';
  return `**Estado de red**

- Salud general: ${status}
- Ping Google: ${connectivity}

**Recomendacion**

Si la latencia sube o falla DNS, revisa gateway y adaptador.`;
}

function formatProcessesSummary(raw) {
  const processes = safeJsonParse(raw, []);
  const count = Array.isArray(processes) ? processes.length : 0;
  const top = Array.isArray(processes) && processes.length > 0 ? processes[0] : null;

  return `**Procesos detectados**

- Total visible: ${count}
- ${top ? `Proceso principal: ${top.name || top.process_name || 'n/a'}` : 'Sin lista real disponible en modo web.'}

**Recomendacion**

Si el equipo esta lento, revisa CPU/RAM en el runtime local.`;
}

function formatUpdatesSummary(windowsUpdateRaw, appUpdatesRaw) {
  const windowsUpdate = safeJsonParse(windowsUpdateRaw, {});
  const appUpdates = String(appUpdatesRaw || '').trim();

  const windowsStatus = windowsUpdate?.Status ?? windowsUpdate?.status ?? 'n/a';
  const windowsName = windowsUpdate?.Name ?? windowsUpdate?.name ?? 'wuauserv';

  return `**Actualizaciones detectadas**

- Windows Update (${windowsName}): ${windowsStatus}
- ${appUpdates ? appUpdates.split('\n')[0] : 'Apps: sin datos disponibles en este modo.'}

**Recomendacion**

Aplica updates en ventana controlada y vuelve a validar.`;
}

function formatCombinedNetworkUpdatesSummary(networkSummary, updatesSummary) {
  return `${networkSummary}

${updatesSummary}`;
}

async function buildLocalFirstSupportResponse(message) {
  const text = normalizeText(message);

  const asksNetwork = containsAny(text, ['red', 'internet', 'dns', 'wifi', 'latencia', 'conexion']);
  const asksUpdates = containsAny(text, ['actualizaciones', 'windows update', 'winget', 'aplicaciones', 'apps']);

  if (containsAny(text, ['estado actual del equipo', 'estado del equipo', 'estado del sistema', 'health completo', 'salud completa', 'salud del equipo', 'salud del sistema'])) {
    const sys = await tryDirectLocalCommand('get_system_info');
    const info = typeof sys === 'string' ? safeJsonParse(sys, {}) : sys;
    return {
      text: formatSystemInfoSummary(info),
      tools_used: [{ name: 'get_system_info', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['cuantos discos', 'cuantos discos tiene', 'cuantas unidades', 'almacenamiento', 'disco', 'discos'])) {
    const sys = await tryDirectLocalCommand('get_system_info');
    const info = typeof sys === 'string' ? safeJsonParse(sys, {}) : sys;
    const disks = Array.isArray(info?.disks) ? info.disks : [];
    const diskListText = disks.length > 0
      ? disks.map((d) => `- **Unidad ${d.name}**: ${d.volume_name || 'Disco Local'} (Libre: ${d.available_space} / Total: ${d.total_space})`).join('\n')
      : '- Sin inventario de discos detectado.';

    return {
      text: [
        '**Inventario de Almacenamiento y Discos**',
        '',
        `- **Total de unidades detectadas**: ${disks.length}`,
        diskListText,
        '',
        info?.source === 'node-bridge-pc'
          ? '_Obtenido mediante Puente de Servidor PC local_'
          : 'Recomendación: En runtime Tauri se leen los discos físicos en tiempo real.',
      ].join('\n'),
      tools_used: [{ name: 'get_system_info', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['ip del equipo', 'cual es la ip', 'cual es la ip del equipo', 'direccion ip', 'ip local'])) {
    const sys = await tryDirectLocalCommand('get_system_info');
    const info = typeof sys === 'string' ? safeJsonParse(sys, {}) : sys;
    const ip = info?.local_ip || '127.0.0.1';
    const hostName = info?.hostname || (typeof window !== 'undefined' && window.location ? window.location.hostname || 'n/a' : 'n/a');

    return {
      text: [
        '**Dirección IP y Red del Equipo**',
        '',
        `- **IP Local del PC**: \`${ip}\``,
        `- **Nombre del Equipo (Hostname)**: \`${hostName}\``,
        `- **Arquitectura/Plataforma**: ${info?.platform || 'n/a'} (${info?.arch || 'x64'})`,
        '',
        info?.source === 'node-bridge-pc'
          ? '_Dirección IP obtenida directamente del adaptador de red local del PC_'
          : 'Recomendación: Ejecuta KernelIA con runtime Tauri para diagnósticos avanzados de red.',
      ].join('\n'),
      tools_used: [{ name: 'get_system_info', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (asksNetwork && asksUpdates) {
    const net = await tryDirectLocalCommand('run_network_diagnostic');
    const netData = typeof net === 'string' ? safeJsonParse(net, {}) : net;
    const windowsUpdate = await tryDirectLocalCommand('get_windows_updates_status');
    const appUpdates = await tryDirectLocalCommand('check_app_updates');
    const updatesText = formatUpdatesSummary(windowsUpdate, appUpdates);
    return {
      text: formatCombinedNetworkUpdatesSummary(formatNetworkSummary(netData), updatesText),
      tools_used: [
        { name: 'run_network_diagnostic', arguments: '{}' },
        { name: 'get_windows_updates_status', arguments: '{}' },
        { name: 'check_app_updates', arguments: '{}' },
      ],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['reiniciar tarjeta', 'reiniciar adaptad', 'reiniciar red', 'resetear red', 'resetear adaptad', 'como puedo reiniciar la tarjeta'])) {
    return {
      text: [
        '### Procedimiento para Reiniciar la Tarjeta de Red',
        '',
        '**1. Solución Automática (PowerShell):**',
        '```powershell',
        '# Reinicia todos los adaptadores de red activos',
        'Get-NetAdapter | Where-Object Status -eq "Up" | Restart-NetAdapter -Confirm:$false',
        '```',
        '',
        '**2. Pasos Manuales por Interfaz Gráfica (Windows):**',
        '1. Presiona `Win + R`, escribe `ncpa.cpl` y presiona **Enter**.',
        '2. Haz clic derecho sobre tu tarjeta de red (**Wi-Fi** o **Ethernet**).',
        '3. Selecciona **Deshabilitar**, espera 5 segundos y luego **Habilitar**.',
        '',
        '**3. Limpieza de Caché DNS y Renovación de IP:**',
        '```cmd',
        'ipconfig /flushdns',
        'ipconfig /release',
        'ipconfig /renew',
        '```',
      ].join('\n'),
      tools_used: [{ name: 'run_network_diagnostic', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (asksNetwork) {
    const net = await tryDirectLocalCommand('run_network_diagnostic');
    const data = typeof net === 'string' ? safeJsonParse(net, {}) : net;
    return {
      text: formatNetworkSummary(data),
      tools_used: [{ name: 'run_network_diagnostic', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['procesos', 'recursos', 'cpu', 'memoria'])) {
    const processes = await tryDirectLocalCommand('list_processes', { sort_by: 'memory', limit: 5 });
    return {
      text: formatProcessesSummary(processes),
      tools_used: [{ name: 'list_processes', arguments: JSON.stringify({ sort_by: 'memory', limit: 5 }) }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (asksUpdates) {
    const windowsUpdate = await tryDirectLocalCommand('get_windows_updates_status');
    const appUpdates = await tryDirectLocalCommand('check_app_updates');
    return {
      text: formatUpdatesSummary(windowsUpdate, appUpdates),
      tools_used: [
        { name: 'get_windows_updates_status', arguments: '{}' },
        { name: 'check_app_updates', arguments: '{}' },
      ],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['escritorio', 'archivos', 'carpetas'])) {
    return {
      text: [
        'Inventario de archivos:',
        '- En modo web no tengo acceso al escritorio real.',
        '- No voy a inventar archivos ni rutas.',
        '- Recomendacion: ejecuta KernelIA con runtime Tauri para listar el escritorio con evidencia real.',
      ].join('\n'),
      tools_used: [],
      model: 'kernelia-local-first',
      error: 'Sin acceso al sistema operativo en modo web.',
    };
  }

  if (containsAny(text, ['reporte tecnico', 'reporte', 'soporte'])) {
    const sys = await tryDirectLocalCommand('get_system_info');
    const info = typeof sys === 'string' ? safeJsonParse(sys, {}) : sys;
    const net = await tryDirectLocalCommand('run_network_diagnostic');
    const netData = typeof net === 'string' ? safeJsonParse(net, {}) : net;
    return {
      text: [
        formatSystemInfoSummary(info),
        '',
        formatNetworkSummary(netData),
      ].join('\n'),
      tools_used: [
        { name: 'get_system_info', arguments: '{}' },
        { name: 'run_network_diagnostic', arguments: '{}' },
      ],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (!isSupportIntent(text)) {
    return null;
  }

  return {
    text: [
      'KernelIA esta operando en modo web directo.',
      'No tengo acceso al sistema operativo desde este navegador.',
      'Recomendacion: usa la app con runtime Tauri para que las herramientas locales respondan con evidencia real.',
    ].join('\n'),
    tools_used: [],
    model: 'kernelia-local-first',
    error: 'Sin runtime Tauri',
  };
}

function getSelectedModel() {
  try {
    return localStorage.getItem(STORAGE_KEY_MODEL) || 'gemma:2b';
  } catch {
    return 'gemma:2b';
  }
}

function setSelectedModel(modelId) {
  try {
    localStorage.setItem(STORAGE_KEY_MODEL, modelId || 'gemma:2b');
  } catch {
    // Ignore storage failures in private mode.
  }
}

function lockedAuthState() {
  return {
    is_authenticated: false,
    username: null,
    profile: null,
    role: 'Viewer',
    tecnico_critical_unlocked: false,
    tecnico_unlock_until_epoch: null,
  };
}

function profileToRole(profile) {
  if (profile === 'superusuario') return 'Owner';
  if (profile === 'tecnico') return 'PowerUser';
  return 'Viewer';
}

function readAuthState() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY_AUTH);
    if (!raw) return lockedAuthState();
    const parsed = JSON.parse(raw);
    return { ...lockedAuthState(), ...parsed };
  } catch {
    return lockedAuthState();
  }
}

function writeAuthState(state) {
  try {
    localStorage.setItem(STORAGE_KEY_AUTH, JSON.stringify(state));
  } catch {
    // Ignore storage failures.
  }
}

function fallbackPresetResponse(message) {
  const text = String(message || '').toLowerCase();

  if (text.includes('docker_mock_mode')) {
    return [
      '### Solución',
      'El endpoint local está en modo simulado (mock). Valide el servicio Docker Model Runner en `localhost:11435`.',
      '',
      '### Consejos y Recomendaciones',
      '- Verifique que el servicio Ollama / Docker esté iniciado y responda en la dirección configurada.',
      '- Ejecute `docker ps` para confirmar que los modelos están cargados en memoria.',
    ].join('\n');
  }

  if (text.includes('diagnostico') && text.includes('red')) {
    return [
      '### Solución',
      '1. Verifique la dirección gateway ejecutando `ipconfig /all`.',
      '2. Pruebe la resolución DNS mediante `nslookup learn.microsoft.com`.',
      '3. Mida la latencia y pérdida de paquetes con `ping 8.8.8.8 -n 10`.',
      '',
      '### Consejos y Recomendaciones',
      '- Si la pérdida de paquetes supera el 2%, reinicie el adaptador de red o valide el cable Ethernet / Wi-Fi.',
      '- Limpie la caché DNS mediante `ipconfig /flushdns` si experimenta intermitencia.',
    ].join('\n');
  }

  return [
    '### Solución',
    'Se ha consultado la documentación técnica oficial de Microsoft (site:learn.microsoft.com OR site:support.microsoft.com) para obtener los pasos de resolución específicos.',
    '1. Identifique el código de estado de Windows y ejecute los diagnósticos no destructivos de lectura.',
    '2. Valide el servicio correspondiente en el panel de herramientas agénticas.',
    '',
    '### Consejos y Recomendaciones',
    '- Mantenga el sistema actualizado mediante Windows Update en ventanas de mantenimiento controladas.',
    '- Consulte la guía oficial de Microsoft Learn antes de aplicar cambios estructurales.',
  ].join('\n');
}

function isAbortLikeError(error) {
  const text = String(error?.message || error || '').toLowerCase();
  return error?.name === 'AbortError' || text.includes('aborted') || text.includes('abort');
}

async function postChatToEndpoint(message, apiUrl, timeoutMs = LOCAL_CHAT_TIMEOUT_MS, retries = LOCAL_CHAT_RETRIES) {
  let attempt = 0;

  while (attempt <= retries) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(apiUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: getSelectedModel(),
          messages: [
            {
              role: 'system',
              content: `Eres KernelIA, un asistente especialista de nivel 3 en soporte informático Windows, redes y diagnósticos de infraestructura.
Cuando el usuario consulte cómo solucionar un problema o pida pasos de resolución:
1. Explica la causa probable del inconveniente.
2. Detalla opciones de solución diferenciando entre Acciones Automáticas (PowerShell/cmdlets) y Acciones Manuales por el Usuario.
3. Responde siempre de forma clara, directa y estructurada en Markdown.`,
            },
            { role: 'user', content: message },
          ],
          temperature: 0.6,
          max_tokens: 500,
          top_p: 0.9,
        }),
        signal: controller.signal,
      });

      if (!response.ok) {
        const body = await response.text();
        throw new Error(`LOCAL_API_${response.status}: ${body || 'request failed'}`);
      }

      const parsed = await response.json();
      const choice = parsed?.choices?.[0]?.message;
      const text = (choice?.content || choice?.reasoning_content || '').trim();

      if (!text) {
        throw new Error('LOCAL_API_EMPTY_RESPONSE');
      }

      return {
        text,
        tools_used: [],
        model: parsed?.model || getSelectedModel(),
        error: null,
      };
    } catch (error) {
      if (attempt < retries && isAbortLikeError(error)) {
        attempt += 1;
        continue;
      }
      throw error;
    } finally {
      clearTimeout(timer);
    }
  }

  throw new Error('LOCAL_API_RETRY_EXHAUSTED');
}

async function chatCompletionsDirect(message, timeoutMs = LOCAL_CHAT_TIMEOUT_MS, retries = LOCAL_CHAT_RETRIES) {
  const localFirst = await buildLocalFirstSupportResponse(message);
  if (localFirst) {
    return localFirst;
  }

  try {
    return await postChatToEndpoint(message, LOCAL_API_BASE_DIRECT, timeoutMs, retries);
  } catch {
    const localFallback = await buildLocalFirstSupportResponse(message);
    if (localFallback) {
      return localFallback;
    }
    return await postChatToEndpoint(message, LOCAL_API_BASE_FALLBACK, timeoutMs, retries);
  }
}

export async function tryDirectLocalCommand(command, payload = {}) {
  if (command === 'stream_message' || command === 'send_message') {
    const message = payload?.message || '';
    const localFirst = await buildLocalFirstSupportResponse(message);
    if (localFirst) {
      return localFirst;
    }

    try {
      return await chatCompletionsDirect(message, LOCAL_CHAT_TIMEOUT_MS, LOCAL_CHAT_RETRIES);
    } catch (error) {
      const errText = String(error?.message || error);
      const conciseError = errText.includes('DOCKER_MOCK_MODE')
        ? 'Docker local endpoint en modo simulado (sin inferencia real).'
        : errText.toLowerCase().includes('abort')
          ? 'El modelo local tardo demasiado en responder; se agoto el tiempo de espera.'
        : errText;
      return {
        text: fallbackPresetResponse(errText.includes('DOCKER_MOCK_MODE') ? 'docker_mock_mode' : message),
        tools_used: [],
        model: 'local-fallback',
        error: `Fallback activado: ${conciseError}`,
      };
    }
  }

  if (command === 'get_settings') {
    const selected = getSelectedModel();
    const auth = readAuthState();
    return {
      selected_model: selected === 'gemma:2b' ? 'gemma3-local' : selected,
      user_role: auth.is_authenticated ? profileToRole(auth.profile) : 'Viewer',
      first_run: false,
      theme: 'dark',
    };
  }

  if (command === 'get_models') {
    const selected = getSelectedModel();
    return [
      {
        id: 'gemma3-local',
        name: 'Gemma 3 Local (Ollama Proxy)',
        provider: 'ollama-openai-proxy',
        hasApiKey: true,
        isLocal: true,
        selected: selected === 'gemma:2b' || selected === 'gemma3-local',
      },
    ];
  }

  if (command === 'set_model') {
    const modelId = payload?.model_id || payload?.modelId || 'gemma3';
    setSelectedModel(modelId === 'gemma3-local' ? 'gemma:2b' : modelId);
    return true;
  }

  if (command === 'set_api_key') {
    const key = payload?.api_key || payload?.apiKey || '';
    if (!String(key).trim()) {
      throw new Error("API_VALIDATION: 'api_key' no puede estar vacio");
    }
    return true;
  }

  if (command === 'confirm_solution_and_ingest') {
    return 'chunk-user-val-web';
  }

  if (command === 'create_support_ticket_cmd') {
    const query = payload?.query || 'Consulta de soporte';
    const timestamp = new Date().toISOString().slice(0, 10).replace(/-/g, '');
    const code = `TK-${timestamp}-${Math.floor(1000 + Math.random() * 9000)}`;
    return {
      ticket_code: code,
      priority: 'Media',
      description: `Incidencia registrada: ${query}`,
      customer_message: `Se ha generado el ticket de soporte #${code}. Un especialista técnico revisará tu caso.`,
    };
  }

  if (command === 'list_support_tickets_cmd') {
    return [];
  }

  if (command === 'create_hitl_checkpoint_cmd') {
    const code = `CHK-${Math.floor(1000 + Math.random() * 9000)}`;
    return {
      id: `chk-${Date.now()}`,
      checkpoint_code: code,
      session_id: payload?.session_id || 'web-session',
      tool_name: payload?.tool_name || 'action',
      args_json: payload?.args_json || '{}',
      risk_level: payload?.risk_level || 'R2',
      required_role: payload?.required_role || 'PowerUser',
      status: 'pending',
      requested_at: new Date().toISOString(),
    };
  }

  if (command === 'resolve_hitl_checkpoint_cmd') {
    const code = payload?.checkpoint_code || 'CHK-0000';
    const action = payload?.action || 'approve';
    const isApprove = action.toLowerCase() === 'approve';
    return {
      checkpoint_code: code,
      status: isApprove ? 'approved' : 'rejected',
      executed: isApprove,
      output: isApprove ? 'Ejecutado con éxito bajo autorización' : null,
      message: isApprove
        ? `Estado reanudado exitosamente. La herramienta fue autorizada.`
        : `Operación #${code} rechazada por el operador.`,
    };
  }

  if (command === 'list_pending_checkpoints_cmd') {
    return [];
  }

  if (command === 'clear_chat') {
    return true;
  }

  if (command === 'get_system_info') {
    try {
      if (typeof fetch !== 'undefined') {
        const res = await fetch('/api/system-info');
        if (res.ok) {
          const data = await res.json();
          return JSON.stringify(data);
        }
      }
    } catch {}
    return JSON.stringify({
      cpu_usage: typeof navigator !== 'undefined' && navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 0,
      memory_used: typeof performance !== 'undefined' && performance.memory ? performance.memory.usedJSHeapSize : 0,
      memory_total: typeof performance !== 'undefined' && performance.memory ? performance.memory.totalJSHeapSize : 1,
      disks: [],
      browser: readBrowserSnapshot(),
      source: 'browser-direct',
    });
  }

  if (command === 'list_processes') {
    return JSON.stringify([]);
  }

  if (command === 'run_network_diagnostic') {
    return JSON.stringify({
      connectivity: {
        google_ping: { latency: typeof navigator !== 'undefined' ? (navigator.onLine ? 'online' : 'offline') : 'n/a' },
      },
      status: typeof navigator !== 'undefined' ? (navigator.onLine ? 'operativa' : 'sin_conexion') : 'n/a',
      source: 'browser-direct',
    });
  }

  if (command === 'list_running_services') {
    return JSON.stringify([]);
  }

  if (command === 'get_storage_summary') {
    return JSON.stringify({
      source: 'browser-direct',
      disks: [],
      summary: 'No hay acceso al inventario real de discos en modo web.',
      browser: readBrowserSnapshot(),
    });
  }

  if (command === 'health_summary') {
    return 'Health del equipo: no disponible en modo web directo.\nRecomendacion: usa runtime Tauri para diagnostico real del sistema.';
  }

  if (command === 'get_windows_updates_status') {
    return JSON.stringify({
      Name: 'wuauserv',
      Status: 'unknown',
      StartType: 'unknown',
      source: 'browser-direct',
    });
  }

  if (command === 'check_app_updates') {
    return [
      'Modo web directo',
      'No hay acceso al gestor de paquetes local desde este navegador.',
      'Recomendacion: ejecutar KernelIA con runtime Tauri para consultar winget.',
    ].join('\n');
  }

  if (command === 'list_directory') {
    return JSON.stringify({
      source: 'browser-direct',
      path: payload?.path || 'desktop',
      items: [],
      note: 'No hay acceso al sistema de archivos real en modo web.',
    });
  }

  if (command === 'get_file_info') {
    return JSON.stringify({
      source: 'browser-direct',
      note: 'No hay acceso al archivo real en modo web.',
    });
  }

  if (command === 'generate_support_report') {
    return { output: 'Reporte local generado en modo browser-direct.' };
  }

  if (command === 'get_auth_status') {
    return readAuthState();
  }

  if (command === 'login_user') {
    const username = String(payload?.username || '').trim();
    const password = String(payload?.password || '');
    const user = DEFAULT_USERS.find((u) => u.username === username && u.password === password);
    if (!user) {
      throw new Error('Credenciales inválidas');
    }

    const status = {
      is_authenticated: true,
      username: user.username,
      profile: user.profile,
      role: profileToRole(user.profile),
      tecnico_critical_unlocked: false,
      tecnico_unlock_until_epoch: null,
    };
    writeAuthState(status);
    return status;
  }

  if (command === 'logout_user') {
    writeAuthState(lockedAuthState());
    return true;
  }

  if (command === 'unlock_tecnico_critical') {
    const auth = readAuthState();
    if (!auth.is_authenticated || auth.profile !== 'tecnico') {
      throw new Error('Esta accion solo aplica para sesiones Tecnico autenticadas');
    }
    const minutes = Number(payload?.minutes || 15);
    const ok = String(payload?.password || '') === 'KernelIA!CriticalProc2026';
    if (!ok) return false;
    const unlockUntil = Math.floor(Date.now() / 1000) + Math.max(1, minutes) * 60;
    const status = {
      ...auth,
      tecnico_critical_unlocked: true,
      tecnico_unlock_until_epoch: unlockUntil,
    };
    writeAuthState(status);
    return true;
  }

  if (command === 'list_support_users') {
    const auth = readAuthState();
    if (auth.profile !== 'superusuario') {
      throw new Error('Solo el superusuario puede listar usuarios');
    }
    return DEFAULT_USERS.map((u) => ({ username: u.username, profile: u.profile, active: true }));
  }

  if (command === 'create_support_user' || command === 'delete_support_user') {
    const auth = readAuthState();
    if (auth.profile !== 'superusuario') {
      throw new Error('Solo el superusuario puede gestionar usuarios');
    }
    return true;
  }

  throw new Error(`LOCAL_DIRECT_UNSUPPORTED:${command}`);
}
