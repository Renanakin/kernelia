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

function getSelectedModel() {
  try {
    return localStorage.getItem(STORAGE_KEY_MODEL) || 'gemma3';
  } catch {
    return 'gemma3';
  }
}

function setSelectedModel(modelId) {
  try {
    localStorage.setItem(STORAGE_KEY_MODEL, modelId || 'gemma3');
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
      'Accion requerida: validar proxy OpenAI localhost:11435 y backend Ollama localhost:11434.',
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
      '- Accion sugerida: validar proxy OpenAI en 11435 y Ollama en 11434',
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
  try {
    return await postChatToEndpoint(message, LOCAL_API_BASE_DIRECT, timeoutMs, retries);
  } catch {
    return await postChatToEndpoint(message, LOCAL_API_BASE_FALLBACK, timeoutMs, retries);
  }
}

export async function tryDirectLocalCommand(command, payload = {}) {
  if (command === 'stream_message' || command === 'send_message') {
    const message = payload?.message || '';
    try {
      return await chatCompletionsDirect(message, LOCAL_CHAT_TIMEOUT_MS, LOCAL_CHAT_RETRIES);
    } catch (error) {
      const errText = String(error?.message || error);
      const conciseError = errText.includes('DOCKER_MOCK_MODE')
        ? 'Docker local endpoint en modo simulado (sin inferencia real).'
        : errText.toLowerCase().includes('abort')
          ? 'El modelo local tardó demasiado en responder; se agotó el tiempo de espera.'
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
      selected_model: selected === 'gemma3' ? 'gemma3-local' : selected,
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
        selected: selected === 'gemma3' || selected === 'gemma3-local',
      },
    ];
  }

  if (command === 'set_model') {
    const modelId = payload?.model_id || payload?.modelId || 'gemma3';
    setSelectedModel(modelId === 'gemma3-local' ? 'gemma3' : modelId);
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
      cpu_usage: 0,
      memory_used: 0,
      memory_total: 1,
      disks: [],
      source: 'local-direct-stub',
    });
  }

  if (command === 'list_processes') {
    return JSON.stringify([]);
  }

  if (command === 'run_network_diagnostic') {
    return JSON.stringify({
      connectivity: {
        google_ping: { latency: 'n/a' },
      },
      source: 'local-direct-stub',
    });
  }

  if (command === 'list_running_services') {
    return JSON.stringify([]);
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
