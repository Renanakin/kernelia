<script>
  import { onMount } from 'svelte';
  import { fade, slide, fly } from 'svelte/transition';
  import { toasts } from '$lib/stores/toastStore.js';
  import { invokeTool } from '$lib/utils/invoke.js';

  let reports = $state([]);
  let syncing = $state(false);
  let loading = $state(true);
  let showTicket = $state(null);

  async function loadReports() {
    loading = true;
    try {
      const res = await invokeTool('list_cloud_reports', {});
      if (res.success) {
        reports = JSON.parse(res.output).sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
      }
    } catch (e) {
      console.error('Failed to load cloud reports:', e);
      toasts.error(String(e?.message || 'Error al cargar reportes de la nube'));
    } finally {
      loading = false;
    }
  }

  async function syncNow() {
    syncing = true;
    try {
      const res = await invokeTool('upload_cloud_report', {});
      if (res.success) {
        await loadReports();
        toasts.success('Reporte sincronizado con éxito');
        const match = res.output.match(/HT-[A-Z0-9]+/);
        if (match) showTicket = match[0];
      } else {
        toasts.error(res.error || 'Error en la sincronización');
      }
    } catch (e) {
      console.error('Sync failed:', e);
      toasts.error(String(e?.message || 'Error crítico en la comunicación cloud'));
    } finally {
      syncing = false;
    }
  }

  onMount(loadReports);

  function formatDate(iso) {
    return new Date(iso).toLocaleString();
  }
</script>

<div class="flex flex-col h-full space-y-6" in:fade>
  <header class="space-y-1">
    <div class="flex items-center justify-between">
      <h3 class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-[0.2em]">Conectividad Enterprise</h3>
      <div class="px-2 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20 text-[9px] text-blue-400 font-bold uppercase tracking-wider">
        Sincronizado
      </div>
    </div>
    <p class="text-[11px] text-[var(--color-text-dim)] leading-relaxed">
      Sincroniza diagnósticos con Hackteck Cloud para soporte prioritario y monitoreo remoto.
    </p>
  </header>

  <button
    onclick={syncNow}
    disabled={syncing}
    class="relative group w-full py-4 bg-gradient-to-br from-blue-600/20 to-indigo-600/20 hover:from-blue-600/30 hover:to-indigo-600/30 border border-blue-500/30 rounded-2xl transition-all duration-500 flex flex-col items-center justify-center gap-2 overflow-hidden"
  >
    {#if syncing}
      <div class="w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"></div>
      <span class="text-[10px] font-bold text-blue-300 uppercase tracking-widest">Sincronizando...</span>
    {:else}
      <div class="flex items-center gap-3">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-blue-400 group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
        <span class="text-[11px] font-bold text-white uppercase tracking-[0.15em]">Subir Reporte a la Nube</span>
      </div>
      <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent -translate-x-full group-hover:animate-shimmer"></div>
    {/if}
  </button>

  {#if showTicket}
    <div transition:slide class="p-4 bg-green-500/10 border border-green-500/20 rounded-xl space-y-2 relative overflow-hidden">
      <div class="flex justify-between items-center">
        <span class="text-[10px] font-bold text-green-400 uppercase tracking-wider">Ticket Generado</span>
        <button onclick={() => showTicket = null} aria-label="Cerrar ticket" class="text-green-400/50 hover:text-green-400">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      <div class="text-xl font-mono font-bold text-white tracking-widest">{showTicket}</div>
      <p class="text-[10px] text-green-400/70">Usa este ID para consultar con soporte técnico.</p>
    </div>
  {/if}

  <div class="flex-1 space-y-4">
    <h4 class="text-[9px] font-bold text-[var(--color-text-dim)] uppercase tracking-wider">Historial de Sincronización</h4>

    <div class="space-y-3 max-h-[300px] overflow-y-auto pr-1 custom-scrollbar">
      {#if loading}
        {#each Array(3) as _}
          <div class="h-20 bg-white/5 rounded-xl animate-pulse"></div>
        {/each}
      {:else if reports.length === 0}
        <div class="flex flex-col items-center justify-center py-12 text-center space-y-3 opacity-50">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-[var(--color-text-dim)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          <p class="text-[10px] uppercase tracking-widest text-[var(--color-text-dim)]">Sin reportes en la nube</p>
        </div>
      {:else}
        {#each reports as report (report.ticket_id)}
          <div transition:fly={{ y: 10, duration: 300 }} class="group p-4 bg-white/5 hover:bg-white/10 border border-white/5 hover:border-blue-500/30 rounded-xl transition-all duration-300 relative overflow-hidden">
            <div class="flex justify-between items-start mb-2">
              <div class="space-y-0.5">
                <span class="text-[10px] font-mono text-blue-400 group-hover:text-blue-300 transition-colors">{report.ticket_id}</span>
                <div class="text-[11px] font-bold text-white tracking-tight">{report.device_name}</div>
              </div>
              <span class="text-[9px] text-[var(--color-text-dim)] font-mono">{formatDate(report.timestamp)}</span>
            </div>

            <p class="text-[10px] text-[var(--color-text-dim)] line-clamp-2">{report.audit_summary}</p>

            <div class="absolute top-0 right-0 p-2 opacity-0 group-hover:opacity-100 transition-opacity">
              <button aria-label="Ver reporte" class="text-blue-400 hover:text-blue-300">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .animate-shimmer {
    animation: shimmer 2s infinite;
  }

  @keyframes shimmer {
    100% {
      transform: translateX(100%);
    }
  }

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
