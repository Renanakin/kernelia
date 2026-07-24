const LOCAL_API_BASE_DIRECT = 'http://localhost:11435/v1/chat/completions';
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
    return {
      text: [
        'Estado de almacenamiento:',
        '- No tengo acceso al inventario real de discos desde el navegador.',
        `- Discos visibles en esta sesion: ${disks.length} (solo runtime web)`,
        info?.source === 'browser-direct'
          ? '- Modo web: solo puedo ver datos simulados del runtime actual.'
          : '- Acceso local disponible.',
        'Recomendacion: en runtime Tauri puedo leer los discos reales del equipo.',
      ].join('\n'),
      tools_used: [{ name: 'get_system_info', arguments: '{}' }],
      model: 'kernelia-local-first',
      error: null,
    };
  }

  if (containsAny(text, ['ip del equipo', 'cual es la ip', 'cual es la ip del equipo', 'direccion ip', 'ip local'])) {
    const browser = readBrowserSnapshot();
    const hostName = typeof window !== 'undefined' && window.location ? window.location.hostname || 'n/a' : 'n/a';
    return {
      text: [
        '**IP del equipo**',
        '',
        '- No puedo leer la IP real del equipo desde el navegador.',
        `- Host visible en el navegador: ${hostName}`,
        `- Estado de conexion del navegador: ${browser.online ? 'en linea' : 'sin conexion'}`,
        '',
        '**Recomendacion**',
        '',
        'Ejecuta KernelIA con runtime Tauri para obtener la IP local real del sistema.',
      ].join('\n'),
      tools_used: [],
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
      'El endpoint local esta en modo simulado (mock).',
      'No está generando inferencia real del LLM.',
      'Accion requerida: validar Docker Model Runner en localhost:11435.',
    ].join('\n');
  }

  if (text.includes('diagnostico') && text.includes('red')) {
    return [
      'Modo local directo activo (fallback).',
      'Diagnostico rapido de red:',
      '1. Verifica gateway: ipconfig /all',
      '2. Prueba DNS: nslookup github.com',
      '3. Prueba latencia: ping 8.8.8.8 -n 10',
      '4. Si hay perdida > 2%, revisa cable/WiFi y drivers.',
    ].join('\n');
  }

  if (text.includes('reporte') || text.includes('soporte')) {
    return [
      'Modo local directo activo (fallback).',
      'Reporte tecnico base:',
      '- Estado: backend IA sin respuesta valida en este intento',
      '- Accion sugerida: validar Docker Model Runner en 11435',
      '- Siguiente paso: repetir prueba con prompt corto ("Di OK")',
    ].join('\n');
  }

  if (text.includes('proceso') || text.includes('recursos')) {
    return [
      'Modo local directo activo (fallback).',
      'No pude consultar procesos por tool nativa en este modo web.',
      'Sugerencia: abre Administrador de tareas y ordena por CPU/RAM.',
      'Si quieres, te doy un checklist de optimizacion paso a paso.',
    ].join('\n');
  }

  if (text.includes('escritorio') || text.includes('archivos')) {
    return [
      'Modo local directo activo (fallback).',
      'La lectura del escritorio requiere runtime Tauri/tool local.',
      'En este modo te puedo guiar con comandos PowerShell para listar archivos.',
    ].join('\n');
  }

  return 'Modo local directo activo. No hubo respuesta del modelo en este intento, pero el sistema sigue operativo.';
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
            { role: 'system', content: 'Responde de forma clara, tecnica y directa. No inventes datos del equipo si no vienen de una herramienta.' },
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

  if (command === 'clear_chat') {
    return true;
  }

  if (command === 'get_system_info') {
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
