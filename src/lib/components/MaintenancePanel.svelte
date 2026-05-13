<script>
  import { invokeTool } from '$lib/utils/invoke.js';
  import { onMount } from 'svelte';
  import { fade, slide } from 'svelte/transition';

  /** @type {string | null} */
  let junkAnalysis = $state(null);
  /** @type {string | null} */
  let registryAnalysis = $state(null);
  /** @type {Array<{id:string,name:string,interval_hours:number,command:string,enabled?:boolean,next_run?:string|number}>} */
  let scheduledTasks = $state([]);
  let analyzingJunk = $state(false);
  let analyzingRegistry = $state(false);
  let cleaning = $state(false);
  /** @type {string | null} */
  let lastAction = $state(null);
  /** @type {string | null} */
  let lastError = $state(null);

  /** @type {{name:string, interval:number, command:string}} */
  let newTask = $state({
    name: '',
    interval: 24,
    command: 'run_cleanup'
  });
  let showTaskForm = $state(false);

  /** @param {unknown} e */
  function errorText(e) {
    const err = /** @type {{message?: string} | null | undefined} */ (e);
    return String(err?.message || e);
  }

  async function analyzeJunk() {
    analyzingJunk = true;
    lastError = null;
    try {
      const res = await invokeTool('analyze_junk', {});
      if (res.success) {
        junkAnalysis = res.output;
      } else {
        junkAnalysis = null;
        lastError = res.error || 'No se pudo analizar archivos temporales.';
      }
    } catch (e) {
      console.error(e);
      junkAnalysis = null;
      lastError = errorText(e);
    } finally {
      analyzingJunk = false;
    }
  }

  async function analyzeRegistry() {
    analyzingRegistry = true;
    lastError = null;
    try {
      const res = await invokeTool('analyze_registry', {});
      if (res.success) {
        registryAnalysis = res.output;
      } else {
        registryAnalysis = null;
        lastError = res.error || 'No se pudo analizar el registro.';
      }
    } catch (e) {
      console.error(e);
      registryAnalysis = null;
      lastError = errorText(e);
    } finally {
      analyzingRegistry = false;
    }
  }

  async function runCleanup() {
    cleaning = true;
    lastError = null;
    try {
      const res = await invokeTool('run_cleanup', { target_areas: null });
      if (res.success) {
        lastAction = res.output;
        await analyzeJunk();
      } else {
        lastAction = null;
        lastError = res.error || 'La limpieza no pudo ejecutarse.';
      }
    } catch (e) {
      console.error(e);
      lastAction = null;
      lastError = errorText(e);
    } finally {
      cleaning = false;
    }
  }

  async function loadTasks() {
    lastError = null;
    try {
      const res = await invokeTool('list_scheduled_tasks', {});
      if (res.success) {
        scheduledTasks = JSON.parse(res.output || '[]');
      } else {
        scheduledTasks = [];
        lastError = res.error || 'No se pudieron cargar las tareas.';
      }
    } catch (e) {
      console.error(e);
      scheduledTasks = [];
      lastError = errorText(e);
    }
  }

  async function addScheduledTask() {
    if (!newTask.name) return;
    lastError = null;
    try {
      const res = await invokeTool('schedule_maintenance', {
          name: newTask.name,
          interval_hours: newTask.interval,
          command: newTask.command
      });

      if (!res.success) {
        lastError = res.error || 'No se pudo programar la tarea.';
        return;
      }

      showTaskForm = false;
      newTask = { name: '', interval: 24, command: 'run_cleanup' };
      await loadTasks();
    } catch (e) {
      console.error(e);
      lastError = errorText(e);
    }
  }

  /** @param {string} id */
  async function deleteTask(id) {
    lastError = null;
    try {
      const res = await invokeTool('delete_scheduled_task', { id });
      if (!res.success) {
        lastError = res.error || 'No se pudo eliminar la tarea.';
        return;
      }
      await loadTasks();
    } catch (e) {
      console.error(e);
      lastError = errorText(e);
    }
  }

  /** @param {string} id
   *  @param {boolean} enabled */
  async function toggleTask(id, enabled) {
    lastError = null;
    try {
      const res = await invokeTool('toggle_scheduled_task', { id, enabled });
      if (!res.success) {
        lastError = res.error || 'No se pudo actualizar la tarea.';
        return;
      }
      await loadTasks();
    } catch (e) {
      console.error(e);
      lastError = errorText(e);
    }
  }

  onMount(() => {
    loadTasks();
  });
</script>

<div class="space-y-6 max-h-[calc(100vh-200px)] overflow-y-auto pr-2 custom-scrollbar" in:fade>
  <div class="grid grid-cols-2 gap-4">
    <div class="bg-white/5 p-4 rounded-2xl border border-white/5 backdrop-blur-md">
      <div class="flex items-center gap-3 mb-2">
        <div class="p-2 bg-blue-500/20 rounded-lg">
          <svg class="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
        </div>
        <h4 class="text-[11px] font-bold text-white uppercase tracking-wider">Archivos Basura</h4>
      </div>
      <p class="text-[9px] text-[var(--color-text-dim)] mb-4 leading-relaxed">Libera espacio eliminando temporales, prefetch y archivos de sistema innecesarios.</p>
      <div class="flex gap-2">
        <button onclick={analyzeJunk} disabled={analyzingJunk} class="flex-1 py-1.5 bg-white/5 hover:bg-white/10 rounded-lg text-[9px] font-bold text-white transition-all uppercase">
          {analyzingJunk ? 'Escaneando...' : 'Analizar'}
        </button>
        <button onclick={runCleanup} disabled={cleaning} class="flex-1 py-1.5 bg-blue-500/20 hover:bg-blue-500/30 border border-blue-500/30 rounded-lg text-[9px] font-bold text-blue-400 transition-all uppercase">
          {cleaning ? 'Limpiando...' : 'Limpiar'}
        </button>
      </div>
    </div>

    <div class="bg-white/5 p-4 rounded-2xl border border-white/5 backdrop-blur-md">
      <div class="flex items-center gap-3 mb-2">
        <div class="p-2 bg-orange-500/20 rounded-lg">
          <svg class="w-4 h-4 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" /></svg>
        </div>
        <h4 class="text-[11px] font-bold text-white uppercase tracking-wider">Registro</h4>
      </div>
      <p class="text-[9px] text-[var(--color-text-dim)] mb-4 leading-relaxed">Repara entradas de inicio inválidas y optimiza la base de datos de configuración.</p>
      <div class="flex gap-2">
        <button onclick={analyzeRegistry} disabled={analyzingRegistry} class="flex-1 py-1.5 bg-white/5 hover:bg-white/10 rounded-lg text-[9px] font-bold text-white transition-all uppercase">
          {analyzingRegistry ? 'Analizando...' : 'Verificar'}
        </button>
        <button onclick={() => invokeTool('fix_registry', {}).then((res) => {
          if (res.success) {
            lastAction = res.output;
            lastError = null;
          } else {
            lastAction = null;
            lastError = res.error || 'No se pudo reparar el registro.';
          }
        })} class="flex-1 py-1.5 bg-orange-500/20 hover:bg-orange-500/30 border border-orange-500/30 rounded-lg text-[9px] font-bold text-orange-400 transition-all uppercase">
          Reparar
        </button>
      </div>
    </div>
  </div>

  {#if junkAnalysis || registryAnalysis}
    <div class="grid grid-cols-1 gap-4" transition:slide>
      {#if junkAnalysis}
        <div class="bg-black/20 p-3 rounded-xl border border-white/5 font-mono text-[10px] text-blue-200/80 max-h-40 overflow-y-auto custom-scrollbar whitespace-pre-wrap">
          {junkAnalysis}
        </div>
      {/if}
      {#if registryAnalysis}
        <div class="bg-black/20 p-3 rounded-xl border border-white/5 font-mono text-[10px] text-orange-200/80 max-h-40 overflow-y-auto custom-scrollbar whitespace-pre-wrap">
          {registryAnalysis}
        </div>
      {/if}
    </div>
  {/if}

  <section class="space-y-4">
    <div class="flex justify-between items-center">
      <h3 class="text-[10px] font-bold text-[var(--color-text-dim)] uppercase tracking-widest flex items-center gap-2">
        <span class="w-1.5 h-1.5 bg-[var(--color-brand-primary)] rounded-full animate-pulse"></span>
        Autonomia de Mantenimiento
      </h3>
      <button
        onclick={() => showTaskForm = !showTaskForm}
        class="p-1.5 bg-[var(--color-brand-primary)]/10 text-[var(--color-brand-primary)] rounded-lg hover:bg-[var(--color-brand-primary)]/20 transition-all"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" /></svg>
      </button>
    </div>

    {#if showTaskForm}
      <div class="bg-white/5 p-4 rounded-2xl border border-[var(--color-brand-primary)]/20 space-y-3" transition:slide>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <label class="text-[9px] text-[var(--color-text-dim)] uppercase ml-1">Nombre</label>
            <input bind:value={newTask.name} type="text" placeholder="Ej: Limpieza Semanal" class="w-full bg-black/40 border border-white/10 rounded-lg px-3 py-1.5 text-[11px] text-white focus:border-[var(--color-brand-primary)]/50 transition-all outline-none" />
          </div>
          <div class="space-y-1">
            <label class="text-[9px] text-[var(--color-text-dim)] uppercase ml-1">Intervalo (Horas)</label>
            <input bind:value={newTask.interval} type="number" class="w-full bg-black/40 border border-white/10 rounded-lg px-3 py-1.5 text-[11px] text-white focus:border-[var(--color-brand-primary)]/50 transition-all outline-none" />
          </div>
        </div>
        <button onclick={addScheduledTask} class="w-full py-2 bg-[var(--color-brand-primary)] text-black text-[10px] font-bold rounded-lg uppercase tracking-widest hover:brightness-110 transition-all">
          Programar Tarea
        </button>
      </div>
    {/if}

    <div class="grid grid-cols-1 gap-2">
      {#if scheduledTasks.length === 0}
        <div class="bg-white/5 p-8 rounded-2xl border border-white/5 text-center">
          <p class="text-[10px] text-[var(--color-text-dim)] uppercase tracking-widest">No hay agentes de mantenimiento activos</p>
        </div>
      {:else}
        {#each scheduledTasks as task}
          <div class="group bg-white/5 p-4 rounded-2xl border border-white/5 flex justify-between items-center hover:bg-white/10 transition-all">
            <div class="flex items-center gap-4">
              <div class="w-1.5 h-8 rounded-full {task.enabled ? 'bg-green-500' : 'bg-red-500/30'}"></div>
              <div>
                <p class="text-[11px] font-bold text-white uppercase tracking-wider">{task.name}</p>
                <p class="text-[9px] text-[var(--color-text-dim)]">Proximo ciclo: {new Date(task.next_run ?? Date.now()).toLocaleString()}</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                onclick={() => toggleTask(task.id, !task.enabled)}
                class="p-2 rounded-lg {task.enabled ? 'bg-green-500/10 text-green-400' : 'bg-white/5 text-white/30'} hover:scale-105 transition-all"
                title={task.enabled ? 'Pausar' : 'Activar'}
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              </button>
              <button
                onclick={() => deleteTask(task.id)}
                class="p-2 bg-red-500/10 text-red-400 rounded-lg hover:bg-red-500/20 transition-all opacity-0 group-hover:opacity-100"
                title="Eliminar"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  {#if lastAction}
    <div class="bg-green-500/10 p-3 rounded-xl border border-green-500/20 flex items-start gap-3" transition:fade>
      <svg class="w-4 h-4 text-green-400 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
      <div>
        <p class="text-[9px] font-bold text-green-400 uppercase tracking-widest mb-1">Operacion Completada</p>
        <p class="text-[10px] text-green-200/70 font-mono leading-relaxed">{lastAction}</p>
      </div>
    </div>
  {/if}

  {#if lastError}
    <div class="bg-red-500/10 p-3 rounded-xl border border-red-500/20 flex items-start gap-3" transition:fade>
      <svg class="w-4 h-4 text-red-400 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
      <div>
        <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest mb-1">Error</p>
        <p class="text-[10px] text-red-200/70 font-mono leading-relaxed">{lastError}</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 3px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }
</style>
