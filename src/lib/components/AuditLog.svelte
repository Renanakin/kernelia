<script>
  import { invokeTool } from '$lib/utils/invoke.js';
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";

  let logs = $state([]);
  let loading = $state(true);
  let limit = $state(50);

  async function fetchLogs() {
    try {
      const result = await invokeTool('get_audit_logs', { limit });
      if (result.success) {
        logs = JSON.parse(result.output);
      }
      loading = false;
    } catch (e) {
      console.error("Failed to fetch audit logs:", e);
      logs = [];
      loading = false;
    }
  }

  onMount(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 15000);
    return () => clearInterval(interval);
  });

  function formatDate(timestamp) {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  <div class="flex items-center justify-between mb-4">
    <h3
      class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-wider"
    >
      Historial de Auditoría
    </h3>
    <button
      onclick={fetchLogs}
      class="p-1 hover:bg-white/5 rounded-md transition-colors text-[var(--color-text-dim)]"
      title="Refrescar"
    >
      <svg
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M23 4v6h-6"></path>
        <path d="M1 20v-6h6"></path>
        <path
          d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"
        ></path>
      </svg>
    </button>
  </div>

  {#if loading && logs.length === 0}
    <div class="flex flex-col items-center justify-center h-40 space-y-4">
      <div
        class="w-6 h-6 border-2 border-[var(--color-brand-primary)] border-t-transparent rounded-full animate-spin"
      ></div>
      <p class="text-[10px] text-[var(--color-text-dim)]">
        Cargando registros...
      </p>
    </div>
  {:else if logs.length === 0}
    <div class="flex flex-col items-center justify-center h-40 text-center p-4">
      <p class="text-xs text-[var(--color-text-dim)]">
        No hay acciones registradas aún.
      </p>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto custom-scrollbar space-y-3 pr-1">
      {#each logs as log (log.timestamp + '-' + log.tool + '-' + log.action)}
        <div
          class="bg-white/5 p-3 rounded-xl border border-white/5 space-y-2 hover:border-white/10 transition-colors"
        >
          <div class="flex justify-between items-start">
            <span
              class="text-[10px] font-mono text-[var(--color-brand-primary)] uppercase tracking-tight"
            >
              {log.tool}
            </span>
            <span class="text-[9px] text-[var(--color-text-dim)]">
              {formatDate(log.timestamp)}
            </span>
          </div>

          <p
            class="text-[11px] text-white/80 line-clamp-2 font-mono break-all bg-black/20 p-1.5 rounded border border-white/5"
          >
            {log.action}
          </p>

          <div class="flex items-center gap-2">
            <span
              class="w-1.5 h-1.5 rounded-full {log.success
                ? 'bg-green-500'
                : 'bg-red-500'}"
            ></span>
            <span
              class="text-[10px] {log.success
                ? 'text-green-400'
                : 'text-red-400'}"
            >
              {log.success ? "Completado" : "Error"}
            </span>
          </div>

          {#if log.error}
            <div
              class="text-[10px] text-red-300/70 bg-red-500/10 p-2 rounded-lg border border-red-500/20 mt-2 italic"
            >
              {log.error}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 2px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
  }
</style>

