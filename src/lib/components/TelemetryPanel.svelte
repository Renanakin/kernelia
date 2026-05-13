<script>
  import {
    generateSupportReport,
    listProcesses,
    listRunningServices,
    runNetworkDiagnostic,
  } from '$lib/api/runtime/client.js';
  import { invokeWithPolicy } from '$lib/utils/invoke.js';
  import { onMount } from 'svelte';
  import { fade, slide } from 'svelte/transition';
  import AuditLog from './AuditLog.svelte';
  import MaintenancePanel from './MaintenancePanel.svelte';
  import CloudPanel from './CloudPanel.svelte';
  import { sidebarTab } from '$lib/stores/settings.js';

  let activeTab = $state('telemetry');
  let systemInfo = $state(null);
  let processes = $state([]);
  let networkStatus = $state(null);
  let services = $state([]);
  let loading = $state(false);
  let lastUpdate = $state(new Date());
  const TAB_STORAGE_KEY = 'kernel_sidebar_tab';

  function normalizeTab(tab) {
    return ['telemetry', 'audit', 'maintenance', 'cloud'].includes(tab) ? tab : 'telemetry';
  }

  async function updateData() {
    if (activeTab !== 'telemetry') return;

    loading = true;
    try {
      const infoRaw = await invokeWithPolicy('get_system_info');
      systemInfo = JSON.parse(infoRaw);

      const procRaw = await listProcesses('memory', 5);
      const parsedProcesses = JSON.parse(procRaw || '[]');
      processes = Array.isArray(parsedProcesses) ? parsedProcesses : [];

      const netRaw = await runNetworkDiagnostic();
      networkStatus = JSON.parse(netRaw);

      const servRaw = await listRunningServices();
      const parsedServices = JSON.parse(servRaw || '[]');
      const allServices = Array.isArray(parsedServices) ? parsedServices : [parsedServices];
      services = allServices.slice(0, 5);

      lastUpdate = new Date();
    } catch (e) {
      console.error('Failed to update telemetry:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    const storedTab = normalizeTab(localStorage.getItem(TAB_STORAGE_KEY));
    activeTab = storedTab;
    sidebarTab.set(storedTab);

    const unsubscribe = sidebarTab.subscribe((tab) => {
      const safeTab = normalizeTab(tab);
      if (activeTab !== safeTab) {
        activeTab = safeTab;
      }
      localStorage.setItem(TAB_STORAGE_KEY, safeTab);
    });

    if (activeTab === 'telemetry') {
      updateData();
    }
    const interval = setInterval(updateData, 10000);
    return () => {
      clearInterval(interval);
      unsubscribe();
    };
  });

  function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function setTab(tab) {
    const safeTab = normalizeTab(tab);
    activeTab = safeTab;
    sidebarTab.set(safeTab);
    localStorage.setItem(TAB_STORAGE_KEY, safeTab);
    if (safeTab === 'telemetry') {
      updateData();
    }
    loading = false;
  }
</script>

<aside class="w-full h-full glass-panel rounded-2xl flex flex-col z-20 overflow-hidden shadow-[0_8px_32px_rgba(0,0,0,0.8)] border border-[var(--glass-border)]">
  <!-- Header with Tabs -->
  <div class="border-b border-[var(--glass-border)]">
    <div class="p-4 flex justify-between items-center pb-2">
      <h2 class="text-[10px] font-bold tracking-[0.2em] text-[var(--color-text-dim)] uppercase">Dashboard de Control</h2>
      <div class="flex items-center gap-2">
         <span class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></span>
         <span class="text-[9px] text-[var(--color-text-dim)]">{lastUpdate.toLocaleTimeString()}</span>
      </div>
    </div>
    
    <div class="flex px-4 gap-4">
      <button 
        type="button"
        onclick={() => setTab('telemetry')}
        class="text-[10px] font-bold pb-2 transition-all relative {activeTab === 'telemetry' ? 'text-[var(--color-brand-primary)]' : 'text-[var(--color-text-dim)]'}"
      >
        TELEMETRÍA
        {#if activeTab === 'telemetry'}
          <div class="absolute bottom-0 left-0 w-full h-0.5 bg-[var(--color-brand-primary)] shadow-[0_0_8px_var(--color-brand-primary)]" in:fade></div>
        {/if}
      </button>
      <button 
        type="button"
        onclick={() => setTab('audit')}
        class="text-[10px] font-bold pb-2 transition-all relative {activeTab === 'audit' ? 'text-[var(--color-brand-primary)]' : 'text-[var(--color-text-dim)]'}"
      >
        AUDITORÍA
        {#if activeTab === 'audit'}
          <div class="absolute bottom-0 left-0 w-full h-0.5 bg-[var(--color-brand-primary)] shadow-[0_0_8px_var(--color-brand-primary)]" in:fade></div>
        {/if}
      </button>
      <button 
        type="button"
        onclick={() => setTab('maintenance')}
        class="text-[10px] font-bold pb-2 transition-all relative {activeTab === 'maintenance' ? 'text-[var(--color-brand-primary)]' : 'text-[var(--color-text-dim)]'}"
      >
        MANTENIMIENTO
        {#if activeTab === 'maintenance'}
          <div class="absolute bottom-0 left-0 w-full h-0.5 bg-[var(--color-brand-primary)] shadow-[0_0_8px_var(--color-brand-primary)]" in:fade></div>
        {/if}
      </button>
      <button 
        type="button"
        onclick={() => setTab('cloud')}
        class="text-[10px] font-bold pb-2 transition-all relative {activeTab === 'cloud' ? 'text-[var(--color-brand-primary)]' : 'text-[var(--color-text-dim)]'}"
      >
        CLOUD
        {#if activeTab === 'cloud'}
          <div class="absolute bottom-0 left-0 w-full h-0.5 bg-[var(--color-brand-primary)] shadow-[0_0_8px_var(--color-brand-primary)]" in:fade></div>
        {/if}
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto custom-scrollbar p-4">
    {#if activeTab === 'telemetry'}
      <div class="space-y-6" in:fade>
        {#if loading}
          <div class="flex flex-col items-center justify-center h-40 space-y-4">
            <div class="w-8 h-8 border-2 border-[var(--color-brand-primary)] border-t-transparent rounded-full animate-spin"></div>
            <p class="text-xs text-[var(--color-text-dim)]">Escaneando Kernel...</p>
          </div>
        {:else}
          <!-- CPU & RAM -->
          <section class="space-y-3">
            <h3 class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-wider">Recursos Críticos</h3>
            <div class="space-y-4 bg-white/5 p-3 rounded-xl border border-white/5">
              <!-- CPU -->
              <div class="space-y-1.5">
                <div class="flex justify-between text-xs">
                  <span class="text-[var(--color-text-dim)]">CPU</span>
                  <span class="text-white font-mono">{systemInfo?.cpu_usage?.toFixed(1)}%</span>
                </div>
                <div class="h-1.5 w-full bg-white/10 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-[var(--color-brand-primary)] shadow-[0_0_8px_var(--color-brand-primary)] transition-all duration-1000"
                    style="width: {systemInfo?.cpu_usage}%"
                  ></div>
                </div>
              </div>

              <!-- RAM -->
              <div class="space-y-1.5">
                <div class="flex justify-between text-xs">
                  <span class="text-[var(--color-text-dim)]">Memoria</span>
                  <span class="text-white font-mono">{((systemInfo?.memory_used / systemInfo?.memory_total) * 100).toFixed(1)}%</span>
                </div>
                <div class="h-1.5 w-full bg-white/10 rounded-full overflow-hidden">
                  <div 
                    class="h-full bg-[var(--color-brand-secondary)] shadow-[0_0_8px_var(--color-brand-secondary)] transition-all duration-1000"
                    style="width: {(systemInfo?.memory_used / systemInfo?.memory_total) * 100}%"
                  ></div>
                </div>
                <div class="flex justify-between text-[10px] text-[var(--color-text-dim)]">
                  <span>{formatBytes(systemInfo?.memory_used)}</span>
                  <span>{formatBytes(systemInfo?.memory_total)}</span>
                </div>
              </div>
            </div>
          </section>

          <!-- Disks -->
          <section class="space-y-3">
            <h3 class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-wider">Almacenamiento</h3>
            <div class="space-y-3">
              {#each systemInfo?.disks || [] as disk}
                <div class="bg-white/5 p-3 rounded-xl border border-white/5 space-y-1.5">
                  <div class="flex justify-between text-xs">
                    <span class="text-[var(--color-text-dim)] truncate max-w-[120px]">{disk.mount_point}</span>
                    <span class="text-white font-mono">{((disk.used_space / disk.total_space) * 100).toFixed(0)}%</span>
                  </div>
                  <div class="h-1.5 w-full bg-white/10 rounded-full overflow-hidden">
                    <div 
                      class="h-full bg-gradient-to-r from-orange-500 to-red-500 transition-all duration-1000"
                      style="width: {(disk.used_space / disk.total_space) * 100}%"
                    ></div>
                  </div>
                </div>
              {/each}
            </div>
          </section>

          <!-- Network -->
          <section class="space-y-3">
            <h3 class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-wider">Conectividad</h3>
            <div class="bg-white/5 p-3 rounded-xl border border-white/5 space-y-2">
              <div class="flex items-center justify-between text-xs">
                <span class="text-[var(--color-text-dim)]">Ping (8.8.8.8)</span>
                <span class="font-mono text-green-400">{networkStatus?.connectivity?.google_ping?.latency || '--'}</span>
              </div>
            </div>
          </section>

          <!-- Actions -->
          <div class="pt-4 border-t border-[var(--glass-border)]">
            <button 
              onclick={() => generateSupportReport().then(res => alert(res.output))}
              class="w-full py-2.5 bg-white/5 hover:bg-[var(--color-brand-primary)]/10 border border-white/10 hover:border-[var(--color-brand-primary)]/30 rounded-xl text-[10px] font-bold transition-all duration-300 flex items-center justify-center gap-2 uppercase tracking-wider"
            >
              Generar Reporte Completo
            </button>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'audit'}
      <div class="h-full" in:fade>
        <AuditLog />
      </div>
    {:else if activeTab === 'maintenance'}
      <div class="h-full" in:fade>
        <MaintenancePanel />
      </div>
    {:else}
      <div class="h-full" in:fade>
        <CloudPanel />
      </div>
    {/if}
  </div>
</aside>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 3px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.03);
    border-radius: 10px;
  }
</style>

