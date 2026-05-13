<script>
  import { getQuickChecks, runQuickCheck } from '$lib/api/runtime/client.js';
  import { createEventDispatcher, onMount } from "svelte";
  import { userRole } from "$lib/stores/settings.js";

  const dispatch = createEventDispatcher();

  // ──────────────────────────────────────────────────────────
  // State
  // ──────────────────────────────────────────────────────────
  export let isProcessing = false;

  /** @type {Array<{id:string,label:string,description:string,icon:string,color:string,kind:string,required_permissions:string[]}>} */
  let checks = [];
  let loadingChecks = true;
  /** @type {string | null} */
  let loadError = null;

  /** Mapa de resultados por check.id */
  /** @type {Record<string, {status:string, output?:string, error?:string, label?:string}>} */
  let results = {};

  /** ID del check que está en ejecución */
  /** @type {string | null} */
  let runningId = null;

  // Paleta de colores por nombre
  /** @type {Record<string, {grad: string, glow: string}>} */
  const colorMap = {
    blue:   { grad: "linear-gradient(135deg, #3b82f6, #06b6d4)", glow: "#3b82f6" },
    orange: { grad: "linear-gradient(135deg, #f97316, #facc15)", glow: "#f97316" },
    yellow: { grad: "linear-gradient(135deg, #eab308, #f59e0b)", glow: "#eab308" },
    green:  { grad: "linear-gradient(135deg, #22c55e, #34d399)", glow: "#22c55e" },
    purple: { grad: "linear-gradient(135deg, #a855f7, #ec4899)", glow: "#a855f7" },
    red:    { grad: "linear-gradient(135deg, #ef4444, #f97316)", glow: "#ef4444" },
  };

  /** @param {string} name */
  function getColor(name) {
    return colorMap[name] || colorMap["blue"];
  }

  // ──────────────────────────────────────────────────────────
  // Cargar checks desde backend (ya filtrados por RBAC)
  // ──────────────────────────────────────────────────────────
  onMount(async () => {
    await loadChecks();
  });

  async function loadChecks() {
    loadingChecks = true;
    loadError = null;
    try {
      checks = await getQuickChecks();
    } catch (e) {
      loadError = e?.toString() || "Error cargando diagnósticos.";
      console.error("[QuickChecks] load error:", e);
    } finally {
      loadingChecks = false;
    }
  }

  // ──────────────────────────────────────────────────────────
  // Ejecutar un check
  // ──────────────────────────────────────────────────────────
  /** @param {{id:string,label:string,kind:string}} check */
  async function runCheck(check) {
    if (isProcessing || runningId) return;

    runningId = check.id;
    results[check.id] = { status: "running" };

    try {
      const res = await runQuickCheck(check.id);

      if (res.kind === "llm_prompt") {
        // Inyectar prompt al chat para que el LLM responda
        dispatch("runPrompt", { prompt: res.output });
        results[check.id] = {
          status: "sent_to_chat",
          label: check.label,
        };
      } else {
        // Resultado directo — renderizar inline
        results[check.id] = {
          status: res.success ? "success" : "error",
          output: res.output,
          error: res.error,
          label: check.label,
        };
      }
    } catch (err) {
      results[check.id] = {
        status: "error",
        error: err?.toString() || "Error de comunicación con el sistema.",
        label: check.label,
      };
    } finally {
      runningId = null;
    }
  }

  /** @param {string} id */
  function clearResult(id) {
    const { [id]: _, ...rest } = results;
    results = rest;
  }

  // ──────────────────────────────────────────────────────────
  // Helpers de renderizado
  // ──────────────────────────────────────────────────────────
  /** @param {string} raw */
  function formatOutput(raw) {
    try {
      const parsed = JSON.parse(raw);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return raw;
    }
  }
</script>

<!-- ─────────────────────────────────────────────────────── -->
<!-- Container                                               -->
<!-- ─────────────────────────────────────────────────────── -->
<div class="qc-container">
  <div class="qc-header">
    <div class="qc-title-row">
      <span class="qc-icon-title">⚡</span>
      <div>
        <h3 class="qc-title">Diagnósticos Rápidos</h3>
        <p class="qc-subtitle">
          Ejecución directa del núcleo — Rol activo:
          <span class="role-badge">{$userRole}</span>
        </p>
      </div>
      <button class="refresh-btn" on:click={loadChecks} title="Recargar checks" disabled={loadingChecks}>
        <svg class:spin={loadingChecks} xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="1 4 1 10 7 10"></polyline>
          <path d="M3.51 15a9 9 0 1 0 .49-3.5"></path>
        </svg>
      </button>
    </div>
  </div>

  <!-- Loading skeleton -->
  {#if loadingChecks}
    <div class="skeleton-grid">
      {#each Array(4) as _}
        <div class="skeleton-card">
          <div class="sk-icon"></div>
          <div class="sk-lines">
            <div class="sk-line sk-line-a"></div>
            <div class="sk-line sk-line-b"></div>
          </div>
        </div>
      {/each}
    </div>

  {:else if loadError}
    <div class="error-banner">
      <span>⚠️ {loadError}</span>
      <button on:click={loadChecks}>Reintentar</button>
    </div>

  {:else if checks.length === 0}
    <div class="empty-state">
      🔒 No hay diagnósticos disponibles para el rol <strong>{$userRole}</strong>.
      Sube tu nivel de acceso en Configuración.
    </div>

  {:else}
    <div class="qc-grid">
      {#each checks as check (check.id)}
        {@const color = getColor(check.color)}
        {@const res = results[check.id]}
        {@const isRunning = runningId === check.id}

        <button
          class="qc-card"
          class:running={isRunning}
          class:has-result={res && res.status !== "running"}
          disabled={!!runningId || isProcessing}
          on:click={() => runCheck(check)}
          aria-label="Ejecutar: {check.label}"
          id="qc-btn-{check.id}"
        >
          <!-- Shimmer on hover -->
          <div class="card-shimmer"></div>

          <!-- Header del check -->
          <div class="card-top">
            <div
              class="card-icon"
              style="background: {color.grad}; box-shadow: 0 4px 12px {color.glow}44;"
            >
              {#if isRunning}
                <span class="spinner">◌</span>
              {:else}
                {check.icon}
              {/if}
            </div>

            <div class="card-info">
              <span class="card-label">{check.label}</span>
              <span class="card-desc">{check.description}</span>
            </div>

            <div class="card-badge-area">
              {#if check.kind === "llm_prompt"}
                <div class="badge badge-ai">IA</div>
              {:else}
                <div class="badge badge-tool">TOOL</div>
              {/if}
            </div>
          </div>

          <!-- Resultado inline -->
          {#if res && res.status !== "running"}
            <div
              class="result-box"
              class:result-success={res.status === "success"}
              class:result-error={res.status === "error"}
              class:result-chat={res.status === "sent_to_chat"}
              role="region"
              aria-label="Resultado de {check.label}"
            >
              {#if res.status === "sent_to_chat"}
                <div class="result-sent">
                  <span>💬 Prompt enviado al chat. Revisa la respuesta del asistente.</span>
                </div>
              {:else if res.status === "error"}
                <div class="result-error-content">
                  <span class="result-error-icon">✗</span>
                  <pre>{res.error}</pre>
                </div>
              {:else}
                <pre class="result-output">{formatOutput(res.output || '')}</pre>
              {/if}
            </div>
          {/if}

          <!-- Progress bar when running -->
          {#if isRunning}
            <div class="progress-bar" style="--glow:{color.glow}"></div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* ── Container ── */
  .qc-container {
    padding: 1.25rem 1.5rem;
    background: rgba(18, 18, 28, 0.7);
    border-radius: 1.25rem;
    border: 1px solid rgba(255, 255, 255, 0.07);
    backdrop-filter: blur(14px);
    margin-bottom: 1.25rem;
  }

  /* ── Header ── */
  .qc-header { margin-bottom: 1.25rem; }

  .qc-title-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .qc-icon-title {
    font-size: 1.4rem;
    line-height: 1;
  }

  .qc-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 700;
    color: #f1f5f9;
    letter-spacing: -0.01em;
  }

  .qc-subtitle {
    margin: 0.15rem 0 0;
    font-size: 0.72rem;
    color: #64748b;
  }

  .role-badge {
    font-weight: 700;
    color: #38bdf8;
  }

  .refresh-btn {
    margin-left: auto;
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.1);
    color: #94a3b8;
    border-radius: 0.5rem;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    padding: 0;
    flex-shrink: 0;
  }

  .refresh-btn:hover:not(:disabled) {
    background: rgba(255,255,255,0.12);
    color: #f1f5f9;
  }

  .refresh-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  @keyframes spin-kf {
    to { transform: rotate(360deg); }
  }
  .spin { animation: spin-kf 0.9s linear infinite; }

  /* ── Grid ── */
  .qc-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.875rem;
  }

  /* ── Card ── */
  .qc-card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 0.9rem 0.9rem 0.9rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 1rem;
    cursor: pointer;
    text-align: left;
    transition: transform 0.2s cubic-bezier(.4,0,.2,1),
                background 0.2s, border-color 0.2s, box-shadow 0.25s;
    overflow: hidden;
  }

  .qc-card:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.14);
    transform: translateY(-3px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.28);
  }

  .qc-card:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    transform: none !important;
  }

  .qc-card.running {
    border-color: rgba(99,102,241,0.4);
    box-shadow: 0 0 20px rgba(99,102,241,0.15);
  }

  .qc-card.has-result {
    padding-bottom: 0;
  }

  /* Shimmer overlay */
  .card-shimmer {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(135deg, rgba(255,255,255,0) 40%, rgba(255,255,255,0.04) 50%, rgba(255,255,255,0) 60%);
    background-size: 200% 200%;
    opacity: 0;
    transition: opacity 0.3s;
    pointer-events: none;
  }

  .qc-card:hover:not(:disabled) .card-shimmer {
    opacity: 1;
    animation: shimmer 2s linear infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* ── Card top ── */
  .card-top {
    display: flex;
    align-items: flex-start;
    gap: 0.7rem;
  }

  .card-icon {
    width: 38px;
    height: 38px;
    min-width: 38px;
    border-radius: 0.7rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.15rem;
    flex-shrink: 0;
  }

  .spinner {
    display: inline-block;
    animation: pulse-spin 0.8s linear infinite;
    font-size: 1.2rem;
  }

  @keyframes pulse-spin {
    0%   { transform: rotate(0deg); opacity: 1; }
    50%  { opacity: 0.5; }
    100% { transform: rotate(360deg); opacity: 1; }
  }

  .card-info {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }

  .card-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #e2e8f0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-desc {
    font-size: 0.68rem;
    color: #64748b;
    margin-top: 0.15rem;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-badge-area { display: flex; gap: 0.3rem; flex-shrink: 0; }

  .badge {
    font-size: 0.55rem;
    font-weight: 800;
    padding: 0.15rem 0.45rem;
    border-radius: 0.4rem;
    letter-spacing: 0.04em;
  }

  .badge-ai {
    background: rgba(99, 102, 241, 0.18);
    color: #818cf8;
    border: 1px solid rgba(99,102,241,0.3);
  }

  .badge-tool {
    background: rgba(16, 185, 129, 0.14);
    color: #34d399;
    border: 1px solid rgba(16,185,129,0.25);
  }

  /* ── Progress bar ── */
  .progress-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, var(--glow, #6366f1), transparent);
    animation: progress-anim 1.4s ease-in-out infinite;
  }

  @keyframes progress-anim {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }

  /* ── Result box ── */
  .result-box {
    position: relative;
    margin: 0.75rem -0.9rem -0rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid rgba(255,255,255,0.06);
    border-radius: 0 0 1rem 1rem;
    font-size: 0.7rem;
    line-height: 1.5;
    cursor: default;
  }

  .result-success {
    background: rgba(16, 185, 129, 0.06);
    border-top-color: rgba(16,185,129,0.15);
  }

  .result-error {
    background: rgba(239, 68, 68, 0.06);
    border-top-color: rgba(239,68,68,0.15);
  }

  .result-chat {
    background: rgba(99, 102, 241, 0.07);
    border-top-color: rgba(99,102,241,0.2);
  }

  .result-sent {
    color: #818cf8;
    font-size: 0.72rem;
  }

  .result-error-content {
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
  }

  .result-error-icon {
    color: #f87171;
    font-weight: 700;
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  .result-error-content pre,
  .result-output {
    color: #94a3b8;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.65rem;
    max-height: 180px;
    overflow-y: auto;
  }

  .result-output {
    color: #a5f3c0;
  }

  .close-result {
    position: absolute;
    top: 0.45rem;
    right: 0.45rem;
    background: rgba(255,255,255,0.07);
    border: none;
    color: #64748b;
    cursor: pointer;
    border-radius: 50%;
    width: 18px;
    height: 18px;
    font-size: 0.65rem;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    transition: background 0.15s, color 0.15s;
  }

  .close-result:hover {
    background: rgba(239,68,68,0.2);
    color: #f87171;
  }

  /* ── Skeleton ── */
  .skeleton-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.875rem;
  }

  .skeleton-card {
    display: flex;
    gap: 0.7rem;
    padding: 0.9rem;
    background: rgba(255,255,255,0.02);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: 1rem;
    animation: sk-pulse 1.4s ease-in-out infinite;
  }

  .sk-icon {
    width: 38px;
    height: 38px;
    border-radius: 0.7rem;
    background: rgba(255,255,255,0.06);
    flex-shrink: 0;
  }

  .sk-lines { flex: 1; display: flex; flex-direction: column; gap: 0.5rem; padding-top: 0.2rem; }
  .sk-line { border-radius: 4px; background: rgba(255,255,255,0.06); }
  .sk-line-a { height: 10px; width: 70%; }
  .sk-line-b { height: 8px; width: 90%; }

  @keyframes sk-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.45; }
  }

  /* ── Error / Empty ── */
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: rgba(239,68,68,0.08);
    border: 1px solid rgba(239,68,68,0.2);
    border-radius: 0.75rem;
    font-size: 0.78rem;
    color: #f87171;
  }

  .error-banner button {
    background: rgba(239,68,68,0.15);
    border: 1px solid rgba(239,68,68,0.3);
    color: #f87171;
    padding: 0.25rem 0.75rem;
    border-radius: 0.5rem;
    cursor: pointer;
    font-size: 0.72rem;
    transition: background 0.2s;
  }

  .error-banner button:hover { background: rgba(239,68,68,0.25); }

  .empty-state {
    text-align: center;
    color: #475569;
    font-size: 0.8rem;
    padding: 1.5rem 1rem;
    background: rgba(255,255,255,0.02);
    border-radius: 0.75rem;
    border: 1px dashed rgba(255,255,255,0.08);
  }

  .empty-state strong { color: #64748b; }

  /* ── Scrollbar for result output ── */
  .result-output::-webkit-scrollbar { width: 4px; }
  .result-output::-webkit-scrollbar-track { background: transparent; }
  .result-output::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.12); border-radius: 2px; }
</style>
