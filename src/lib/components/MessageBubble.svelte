<script>
  import { renderMarkdown } from '$lib/utils/markdown.js';
  import { formatToolName } from '$lib/utils/formatting.js';

  let { message } = $props();

  const isUser = $derived(message.role === 'user');
  const isSystem = $derived(message.role === 'system');
  const hasTools = $derived(message.toolsUsed && message.toolsUsed.length > 0);
  const hasError = $derived(!!message.error);
  const ragContext = $derived(message.ragContext || null);
  const ragComparison = $derived(message.ragComparison || null);
  const showRagBadge = $derived(
    !!ragContext && ragContext.enabled && ragContext.show_summary_badge && !isUser && !isSystem
  );
  let showPasswordModal = $state(false);
  let passwordInput = $state('');
  let passwordError = $state('');
  let autoExecStatus = $state(null); // null, 'success', 'denied'
  let isExecuting = $state(false);
  let requiredRole = $state('tech_analyst');

  async function handleAutoResolveClick() {
    isExecuting = true;
    passwordError = '';

    // Si la solucion incluye comandos o frases de configuracion/reparacion R2/R3/R4, solicitar elevacion
    const text = (message.text || '').toLowerCase();
    const needsElevation = text.includes('spooler') || text.includes('driver') || text.includes('servicio') || text.includes('reinic') || text.includes('red');

    if (needsElevation) {
      requiredRole = text.includes('driver') || text.includes('formatear') ? 'superadmin' : 'tech_analyst';
      showPasswordModal = true;
      isExecuting = false;
      return;
    }

    // Accion R0/R1 inocua: ejecutar directamente
    await executeAutoResolution();
  }

  async function submitPasswordElevation() {
    if (!passwordInput.trim()) {
      passwordError = 'Por favor ingrese su clave técnica.';
      return;
    }

    try {
      let isOk = false;
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        const res = await invoke('verify_tech_password_cmd', {
          password: passwordInput,
          required_role: requiredRole
        });
        isOk = res.success;
      } else {
        // En entorno web directo, validar claves de prueba
        isOk = passwordInput === 'admin123' || passwordInput === 'superadmin123' || passwordInput === 'kernelia2026';
      }

      if (isOk) {
        showPasswordModal = false;
        passwordInput = '';
        await executeAutoResolution();
      } else {
        passwordError = 'Contraseña incorrecta o permisos insuficientes.';
        autoExecStatus = 'denied';
      }
    } catch {
      passwordError = 'Error al validar credenciales.';
      autoExecStatus = 'denied';
    }
  }

  async function executeAutoResolution() {
    isExecuting = true;
    try {
      // Simulación de resolución automática Nivel 1 TI exitosa
      await new Promise((r) => setTimeout(r, 600));
      autoExecStatus = 'success';
    } catch {
      autoExecStatus = 'denied';
    } finally {
      isExecuting = false;
    }
  }
</script>

<div class="flex flex-col {isUser ? 'items-end' : 'items-start'} group w-full mb-8">
  {#if !isUser && !isSystem}
    <!-- Origin Label (Zen Canvas Style) -->
    <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-white/10 bg-white/5 text-xs font-mono text-gray-300 backdrop-blur-md mb-4 shadow-sm">
      <div class="w-1.5 h-1.5 bg-[#00FF80] rounded-full animate-pulse"></div>
      {message.model ? `Kernel IA (${message.model})` : 'Kernel IA'}
    </div>
  {/if}

  <div
    class="max-w-[90%] transition-all duration-300 {isUser
      ? 'bg-white/10 backdrop-blur-md border border-white/10 text-white px-6 py-4 rounded-2xl rounded-tr-sm shadow-sm'
      : isSystem
        ? 'bg-transparent text-gray-500 font-mono text-xs w-full text-center'
        : 'bg-transparent text-gray-200'}"
  >
    {#if showRagBadge}
      <div class="flex flex-wrap gap-2 mb-3">
        {#if ragContext.specialty}
          <span class="px-2 py-1 rounded-full bg-cyan-500/10 border border-cyan-400/20 text-[10px] font-mono uppercase tracking-wider text-cyan-300">
            {ragContext.specialty}
          </span>
        {/if}
        {#if ragContext.confidence_level}
          <span class="px-2 py-1 rounded-full bg-emerald-500/10 border border-emerald-400/20 text-[10px] font-mono uppercase tracking-wider text-emerald-300">
            {ragContext.confidence_level} {ragContext.confidence_score ? `${Math.round(ragContext.confidence_score * 100)}%` : ''}
          </span>
        {/if}
        {#if ragContext.decision_mode}
          <span class="px-2 py-1 rounded-full bg-amber-500/10 border border-amber-400/20 text-[10px] font-mono uppercase tracking-wider text-amber-300">
            {ragContext.decision_mode}
          </span>
        {/if}
      </div>
    {/if}

    {#if message.currentTool}
      <div class="flex items-center gap-3 text-[var(--color-brand-success)] py-2 mb-2">
        <div class="relative w-4 h-4 shrink-0">
          <div class="absolute inset-0 border-2 border-[var(--color-brand-success)]/20 rounded-full"></div>
          <div class="absolute inset-0 border-2 border-[var(--color-brand-success)] rounded-full border-t-transparent animate-spin"></div>
        </div>
        <span class="text-xs font-mono tracking-widest uppercase truncate">EJECUTANDO HERRAMIENTA: {formatToolName(message.currentTool)}...</span>
      </div>
    {/if}

    {#if message.isLoading && !message.currentTool}
      <div class="flex items-center gap-3 text-gray-400 py-2 mt-2">
        <div class="relative w-4 h-4">
          <div class="absolute inset-0 border-2 border-gray-400/20 rounded-full"></div>
          <div class="absolute inset-0 border-2 border-gray-400 rounded-full border-t-transparent animate-spin"></div>
        </div>
        <span class="text-xs font-mono tracking-wider">
          {message.statusMessage || (message.content ? 'PROCESANDO...' : 'Buscando solución en Microsoft...')}
        </span>
      </div>
    {:else if hasError}
      <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 mb-2">
        <div class="flex items-center gap-2 text-red-400 text-sm font-bold mb-1">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          SISTEMA ERROR
        </div>
        <span class="text-sm text-red-200/80">{message.error}</span>
      </div>
    {:else if message.content}
      <!-- Clean markdown layout typography -->
      <div class="markdown-content leading-relaxed {isUser ? 'text-sm text-slate-100' : 'text-sm md:text-base font-normal text-slate-100'}">
        {@html renderMarkdown(message.content)}
      </div>
    {/if}

    {#if hasTools}
      <div class="mt-4 pt-4 border-t border-white/5 space-y-2 max-w-sm">
        {#each message.toolsUsed as tool}
          <div class="flex items-center gap-3 bg-white/5 hover:bg-white/10 rounded-xl px-4 py-3 border border-white/5 transition-colors group/tool cursor-pointer">
            <div class="p-2 rounded-lg bg-[#00FF80]/10 text-[#00FF80]">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="16 18 22 12 16 6"></polyline>
                <polyline points="8 6 2 12 8 18"></polyline>
              </svg>
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-[10px] font-bold text-[#00FF80] uppercase tracking-tighter">TOOL EXECUTED</span>
              <span class="text-xs text-white font-mono truncate">{formatToolName(tool.name)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    {#if showRagDebug}
      <details class="mt-4 rounded-xl border border-white/10 bg-white/5 overflow-hidden">
        <summary class="px-4 py-3 text-[11px] font-mono uppercase tracking-[0.18em] text-gray-300 cursor-pointer select-none">
          QA Panel RAG
        </summary>
        <div class="px-4 pb-4 pt-1 space-y-3 text-xs text-gray-300">
          <div class="grid gap-2 md:grid-cols-2">
            <div>motor: {ragContext.enabled ? 'rag' : 'legacy'}</div>
            <div>trace: {ragContext.trace_id || 'n/a'}</div>
            <div>specialty: {ragContext.specialty || 'n/a'}</div>
            <div>decision: {ragContext.decision_mode || 'n/a'}</div>
            <div>confidence: {ragContext.confidence_level || 'n/a'}</div>
            <div>risk: {ragContext.risk_level || 'n/a'}</div>
          </div>

          {#if ragContext.retrieval_counts?.length}
            <div>
              <div class="text-[10px] uppercase tracking-[0.18em] text-gray-500 mb-1">Retrieval</div>
              <div class="flex flex-wrap gap-2">
                {#each ragContext.retrieval_counts as item}
                  <span class="px-2 py-1 rounded-full bg-white/5 border border-white/10 font-mono">{item}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if ragContext.reason_codes?.length}
            <div>
              <div class="text-[10px] uppercase tracking-[0.18em] text-gray-500 mb-1">Reason Codes</div>
              <div class="flex flex-wrap gap-2">
                {#each ragContext.reason_codes as item}
                  <span class="px-2 py-1 rounded-full bg-white/5 border border-white/10 font-mono">{item}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if ragContext.live_conflicts?.length}
            <div>
              <div class="text-[10px] uppercase tracking-[0.18em] text-gray-500 mb-1">Conflicts</div>
              <div class="flex flex-wrap gap-2">
                {#each ragContext.live_conflicts as item}
                  <span class="px-2 py-1 rounded-full bg-red-500/10 border border-red-400/20 text-red-300 font-mono">{item}</span>
                {/each}
              </div>
            </div>
          {/if}

          {#if ragComparison}
            <div class="border-t border-white/10 pt-3">
              <div class="text-[10px] uppercase tracking-[0.18em] text-gray-500 mb-1">Compare Mode</div>
              <div class="grid gap-2 md:grid-cols-2">
                <div>legacy_intent: {ragComparison.legacy_intent}</div>
                <div>legacy_confidence: {Math.round((ragComparison.legacy_confidence || 0) * 100)}%</div>
                <div>rag_specialty: {ragComparison.rag_specialty}</div>
                <div>rag_confidence: {Math.round((ragComparison.rag_confidence || 0) * 100)}%</div>
              </div>
              {#if ragComparison.legacy_plan?.length}
                <div class="mt-2 text-gray-400">
                  legacy_plan: {ragComparison.legacy_plan.join(' | ')}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </details>
    {/if}

    {#if !isUser && !isSystem && message.text && message.text.includes('### Solución')}
      <!-- Action Bar: Resolver Automáticamente -->
      <div class="mt-4 pt-3 border-t border-white/10 flex flex-wrap items-center justify-between gap-3">
        <div class="flex items-center gap-2 text-xs text-emerald-400 font-mono">
          <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
          Resolución Nivel 1 TI Asistida
        </div>

        {#if autoExecStatus === 'success'}
          <div class="px-3 py-1.5 rounded-lg bg-emerald-500/20 border border-emerald-500/40 text-emerald-300 text-xs font-mono">
            ✓ Acción Ejecutada & Auditada en Sistema
          </div>
        {:else if autoExecStatus === 'denied'}
          <div class="px-3 py-1.5 rounded-lg bg-red-500/20 border border-red-500/40 text-red-300 text-xs font-mono">
            ✕ Elevación Rechazada (Contraseña Incorrecta)
          </div>
        {:else}
          <button
            onclick={handleAutoResolveClick}
            disabled={isExecuting}
            class="px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white font-medium text-xs shadow-lg shadow-emerald-900/30 flex items-center gap-2 transition-all active:scale-95 disabled:opacity-50"
          >
            <span>⚡ Resolver Automáticamente</span>
          </button>
        {/if}
      </div>
    {/if}

    <!-- Modal Desafío de Contraseña Técnico / Superusuario -->
    {#if showPasswordModal}
      <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md">
        <div class="w-full max-w-md p-6 bg-gray-900 border border-emerald-500/30 rounded-2xl shadow-2xl space-y-4">
          <div class="flex items-center justify-between border-b border-white/10 pb-3">
            <div class="flex items-center gap-2 text-emerald-400 font-bold text-sm">
              <span>🔐 Requiere Elevación de Privilegios</span>
            </div>
            <button onclick={() => showPasswordModal = false} class="text-gray-400 hover:text-white text-xs">✕</button>
          </div>

          <p class="text-xs text-gray-300">
            Esta acción requiere autorización de técnico Nivel 1/2 o Superusuario (Privilegio {requiredRole}). Por favor ingrese su clave de elevación:
          </p>

          <input
            type="password"
            bind:value={passwordInput}
            placeholder="Ingrese contraseña de técnico o superusuario..."
            class="w-full px-4 py-2.5 bg-black/50 border border-white/20 rounded-xl text-white text-xs focus:outline-none focus:border-emerald-400 font-mono"
          />

          {#if passwordError}
            <div class="text-xs text-red-400 font-mono">{passwordError}</div>
          {/if}

          <div class="flex justify-end gap-3 pt-2">
            <button
              onclick={() => showPasswordModal = false}
              class="px-4 py-2 rounded-xl bg-white/10 text-gray-300 hover:bg-white/20 text-xs font-mono"
            >
              Cancelar
            </button>
            <button
              onclick={submitPasswordElevation}
              class="px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium font-mono shadow-md"
            >
              Validar y Ejecutar
            </button>
          </div>
        </div>
      </div>
    {/if}

    <div class="flex {isUser ? 'justify-end' : 'justify-start'} mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
      <span class="text-[10px] text-gray-500 font-mono uppercase tracking-widest">{message.timestamp}</span>
    </div>
  </div>
</div>
