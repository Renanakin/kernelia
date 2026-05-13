<script>
  import { getAuditLogs } from '$lib/api/runtime/client.js';
  import { onMount } from "svelte";
  import {
    Shield,
    Clock,
    Terminal,
    CheckCircle2,
    XCircle,
    ChevronDown,
    ChevronUp,
    Search,
    RefreshCw,
  } from "lucide-svelte";
  import { fade, slide, fly } from "svelte/transition";

  export let show = false;
  export let onClose = () => {};

  let logs = [];
  let loading = false;
  let loadError = "";
  let searchTerm = "";
  let expandedLog = null;
  let wasShown = false;

  function normalizeLog(log) {
    return {
      ...log,
      tool_name: log.tool_name || log.tool || "Sistema",
      arguments: log.arguments || log.action || "",
      agent: log.agent || "Sistema",
    };
  }

  async function loadLogs() {
    if (loading) return;
    loading = true;
    loadError = "";
    try {
      const result = await getAuditLogs();
      logs = Array.isArray(result) ? result.map(normalizeLog) : [];
    } catch (err) {
      console.error("Error loading audit logs:", err);
      loadError = err?.message || String(err);
      logs = [];
    } finally {
      loading = false;
    }
  }

  $: filteredLogs = logs.filter(
    (log) =>
      (log.tool_name || "").toLowerCase().includes(searchTerm.toLowerCase()) ||
      (log.arguments || "").toLowerCase().includes(searchTerm.toLowerCase()) ||
      (log.agent || "").toLowerCase().includes(searchTerm.toLowerCase()),
  );

  onMount(() => {
    if (show) loadLogs();
  });

  $: if (show && !wasShown) {
    wasShown = true;
    loadLogs();
  } else if (!show && wasShown) {
    wasShown = false;
    expandedLog = null;
    searchTerm = "";
  }

  function toggleExpand(index) {
    expandedLog = expandedLog === index ? null : index;
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/80 backdrop-blur-md"
    on:click|self={onClose}
    transition:fade={{ duration: 200 }}
  >
    <div
      class="w-full max-w-4xl max-h-[85vh] glass-panel border-[var(--glass-border)] rounded-2xl shadow-2xl overflow-hidden flex flex-col"
      transition:fly={{ y: 20, duration: 300 }}
    >
      <!-- Header -->
      <div
        class="p-6 border-b border-white/5 flex items-center justify-between bg-[var(--color-bg-input)]"
      >
        <div class="flex items-center gap-3">
          <div class="p-2 bg-blue-500/10 rounded-lg">
            <Shield class="w-6 h-6 text-blue-400" />
          </div>
          <div>
            <h2 class="text-xl font-bold text-white tracking-tight">
              Panel de Auditoría
            </h2>
            <p class="text-xs text-gray-400 mt-1">
              Acciones ejecutadas por el sistema en este equipo
            </p>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <button
            on:click={loadLogs}
            class="p-2 hover:bg-white/5 rounded-full transition-colors text-gray-400 hover:text-white"
            title="Refrescar"
          >
            <RefreshCw class="w-5 h-5 {loading ? 'animate-spin' : ''}" />
          </button>
          <button
            on:click={onClose}
            class="px-4 py-2 bg-white/5 hover:bg-white/10 rounded-xl text-sm font-medium text-gray-300 transition-all border border-white/10"
          >
            Cerrar
          </button>
        </div>
      </div>

      <!-- Search & Filters -->
      <div class="p-4 bg-white/[0.02] border-b border-white/5">
        <div class="relative">
          <Search
            class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
          />
          <input
            type="text"
            bind:value={searchTerm}
            placeholder="Buscar por comando o herramienta..."
            class="w-full bg-[#0a0c10] border border-white/5 rounded-xl py-2.5 pl-10 pr-4 text-sm text-gray-200 placeholder:text-gray-600 focus:outline-none focus:border-blue-500/50 transition-all"
          />
        </div>
      </div>

      <!-- Logs Table -->
      <div class="flex-1 overflow-y-auto custom-scrollbar">
        {#if loading && logs.length === 0}
          <div class="flex flex-col items-center justify-center py-20 gap-4">
            <RefreshCw class="w-8 h-8 text-blue-500/50 animate-spin" />
            <p class="text-gray-500 text-sm italic">
              Sincronizando registros...
            </p>
          </div>
        {:else if loadError}
          <div
            class="flex flex-col items-center justify-center py-20 gap-4 opacity-70"
          >
            <XCircle class="w-12 h-12 text-red-400" />
            <p class="text-red-200 text-sm">
              No se pudieron cargar los registros
            </p>
            <p class="text-gray-500 text-xs font-mono max-w-lg text-center">
              {loadError}
            </p>
          </div>
        {:else if filteredLogs.length === 0}
          <div
            class="flex flex-col items-center justify-center py-20 gap-4 opacity-40"
          >
            <Terminal class="w-12 h-12 text-gray-500" />
            <p class="text-gray-500 text-sm">
              No se encontraron registros de Auditoría
            </p>
          </div>
        {:else}
          <div class="divide-y divide-white/5">
            {#each filteredLogs as log, i}
              <div class="group hover:bg-white/[0.02] transition-colors">
                <!-- Log Entry Header -->
                <button
                  on:click={() => toggleExpand(i)}
                  class="w-full text-left p-4 flex items-center gap-4 focus:outline-none"
                >
                  <div
                    class="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-full {log.success
                      ? 'bg-green-500/10'
                      : 'bg-red-500/10'}"
                  >
                    {#if log.success}
                      <CheckCircle2 class="w-4 h-4 text-green-400" />
                    {:else}
                      <XCircle class="w-4 h-4 text-red-400" />
                    {/if}
                  </div>

                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-0.5">
                      <span
                        class="text-xs font-bold text-gray-400 bg-white/5 border border-white/10 px-2 py-0.5 rounded leading-none uppercase tracking-tighter"
                        >{log.agent || "Sistema"}</span
                      >
                      <span
                        class="text-sm font-mono text-blue-400 bg-blue-400/10 px-2 py-0.5 rounded leading-none"
                        >{log.tool_name}</span
                      >
                      <span
                        class="text-xs text-gray-500 flex items-center gap-1"
                      >
                        <Clock class="w-3 h-3" />
                        {log.timestamp}
                      </span>
                    </div>
                    <p
                      class="text-sm text-gray-300 truncate font-mono bg-black/30 px-2 py-1 rounded border border-white/5 mt-2"
                    >
                      {log.arguments}
                    </p>
                  </div>

                  <div class="flex-shrink-0">
                    {#if expandedLog === i}
                      <ChevronUp class="w-4 h-4 text-gray-500" />
                    {:else}
                      <ChevronDown class="w-4 h-4 text-gray-500" />
                    {/if}
                  </div>
                </button>

                <!-- Expanded Details -->
                {#if expandedLog === i}
                  <div class="px-16 pb-6 pt-0" transition:slide>
                    <div
                      class="space-y-4 bg-black/40 rounded-xl p-4 border border-white/5"
                    >
                      <div>
                        <h4
                          class="text-xs font-bold text-gray-500 uppercase tracking-widest mb-2"
                        >
                          Argumentos Completos
                        </h4>
                        <pre
                          class="text-xs text-blue-200/80 whitespace-pre-wrap break-all font-mono leading-relaxed">{log.arguments}</pre>
                      </div>

                      {#if !log.success || log.error}
                        <div>
                          <h4
                            class="text-xs font-bold text-red-400/70 uppercase tracking-widest mb-2"
                          >
                            Error / Resultado
                          </h4>
                          <div
                            class="p-3 bg-red-500/5 border border-red-500/10 rounded-lg"
                          >
                            <p class="text-xs text-red-200/70 font-mono">
                              {log.error ||
                                "Operación fallida sin mensaje de error"}
                            </p>
                          </div>
                        </div>
                      {:else}
                        <div
                          class="flex items-center gap-2 text-xs text-green-400/70 italic"
                        >
                          <CheckCircle2 class="w-3 h-3" />
                          Operación completada satisfactoriamente
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div
        class="p-4 border-t border-white/5 bg-black/40 flex justify-between items-center text-[10px] text-gray-600 uppercase tracking-widest"
      >
        <span>Kernel IA v{window.__APP_VERSION__ || "1.0"}</span>
        <span>Auditoría de Seguridad Activa</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>


