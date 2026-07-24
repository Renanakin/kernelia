<script>
  import { setApiKey } from '$lib/api/runtime/client.js';
  import { invokeWithPolicy } from '$lib/utils/invoke.js';
  import {
    models,
    settingsOpen,
    userRole,
    ragEngineEnabled,
    ragCompareMode,
    ragDebugPanel,
    ragShowConfidenceBadge,
    saveSettings
  } from '$lib/stores/settings.js';

  let apiKeys = $state({});
  let saving = $state({});
  let successMsg = $state({});

  let megabossPassword = $state('');
  let megabossUnlockPassword = $state('');
  let megabossMinutes = $state(20);
  let megabossStatus = $state({ password_set: false, unlocked: false, unlock_until_epoch: null });
  let megabossMsg = $state('');

  const roles = ['Owner', 'PowerUser', 'Viewer'];

  async function handleRoleChange(newRole) {
    await saveSettings({ user_role: newRole });
  }

  async function toggleSetting(key, value) {
    await saveSettings({ [key]: value });
  }

  async function refreshMegabossStatus() {
    try {
      megabossStatus = await invokeWithPolicy('megaboss_status');
    } catch (e) {
      console.error('Failed to load MegaBoss status:', e);
    }
  }

  async function saveMegabossPassword() {
    const pass = megabossPassword.trim();
    if (pass.length < 6) {
      megabossMsg = 'La clave debe tener al menos 6 caracteres.';
      return;
    }
    try {
      await invokeWithPolicy('set_megaboss_password', { password: pass });
      megabossPassword = '';
      megabossMsg = 'Clave MegaBoss guardada.';
      await refreshMegabossStatus();
    } catch (e) {
      megabossMsg = `Error: ${String(e)}`;
    }
  }

  async function unlockMegaboss() {
    const pass = megabossUnlockPassword.trim();
    if (!pass) return;
    try {
      const ok = await invokeWithPolicy('unlock_megaboss', { password: pass, minutes: Number(megabossMinutes) });
      megabossMsg = ok ? 'MegaBoss desbloqueado.' : 'Clave MegaBoss incorrecta.';
      megabossUnlockPassword = '';
      await refreshMegabossStatus();
    } catch (e) {
      megabossMsg = `Error: ${String(e)}`;
    }
  }

  $effect(() => {
    if ($models.length > 0) {
      for (const model of $models) {
        if (!(model.id in apiKeys)) apiKeys[model.id] = '';
      }
    }
  });

  $effect(() => {
    if ($settingsOpen) {
      refreshMegabossStatus();
    }
  });

  async function saveApiKey(modelId) {
    const key = apiKeys[modelId]?.trim();
    if (!key) return;

    saving[modelId] = true;
    try {
      await setApiKey(modelId, key);
      $models = $models.map((m) => (m.id === modelId ? { ...m, hasApiKey: true } : m));
      apiKeys[modelId] = '';
      successMsg[modelId] = 'Guardada';
      setTimeout(() => (successMsg[modelId] = ''), 2000);
    } catch (e) {
      console.error('Failed to save API key:', e);
      successMsg[modelId] = 'Error';
    } finally {
      saving[modelId] = false;
    }
  }

  function close() {
    $settingsOpen = false;
  }
</script>

{#if $settingsOpen}
  <div class="fixed inset-0 bg-black/80 z-[1000] flex items-center justify-center" onclick={close}>
    <div class="bg-[var(--color-bg-card)] border border-[var(--glass-border)] rounded-2xl w-[620px] max-h-[80vh] overflow-hidden shadow-2xl" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--glass-border)]">
        <h2 class="text-lg font-[var(--font-heading)] text-[var(--color-brand-secondary)]">Configuración</h2>
        <button onclick={close} class="text-[var(--color-text-dim)] hover:text-[var(--color-text-main)] transition-colors">X</button>
      </div>

      <div class="overflow-y-auto max-h-[60vh] p-6">
        <div class="mb-8">
          <h3 class="text-sm font-semibold text-[var(--color-text-main)] mb-1">Seguridad (RBAC)</h3>
          <p class="text-xs text-[var(--color-text-dim)] mb-3">Define tu nivel de acceso al sistema.</p>
          <div class="flex gap-2 p-1 bg-[var(--color-bg-input)] rounded-xl border border-[var(--glass-border)]">
            {#each roles as role}
              <button onclick={() => handleRoleChange(role)} class="flex-1 py-2 text-xs font-medium rounded-lg transition-all duration-300 {role === $userRole ? 'bg-[var(--color-brand-secondary)] text-white shadow-lg' : 'text-[var(--color-text-dim)] hover:text-[var(--color-text-main)] hover:bg-white/5'}">
                {role}
              </button>
            {/each}
          </div>
        </div>

        <div class="mb-8">
          <h3 class="text-sm font-semibold text-[var(--color-text-main)] mb-1">Rollout RAG</h3>
          <p class="text-xs text-[var(--color-text-dim)] mb-3">Controla la activacion gradual del nuevo nucleo y el panel interno de QA.</p>

          <div class="space-y-3">
            <label class="flex items-center justify-between gap-4 bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
              <div>
                <div class="text-sm text-[var(--color-text-main)]">Activar motor RAG nuevo</div>
                <div class="text-xs text-[var(--color-text-dim)]">Si se desactiva, el chat sigue con el flujo operativo base sin contexto RAG gobernado.</div>
              </div>
              <input type="checkbox" checked={$ragEngineEnabled} onchange={(e) => toggleSetting('rag_engine_enabled', e.currentTarget.checked)} />
            </label>

            <label class="flex items-center justify-between gap-4 bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
              <div>
                <div class="text-sm text-[var(--color-text-main)]">Modo comparativo QA</div>
                <div class="text-xs text-[var(--color-text-dim)]">Muestra comparacion entre la lectura legacy de intencion y la decision del RAG nuevo.</div>
              </div>
              <input type="checkbox" checked={$ragCompareMode} onchange={(e) => toggleSetting('rag_compare_mode', e.currentTarget.checked)} />
            </label>

            <label class="flex items-center justify-between gap-4 bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
              <div>
                <div class="text-sm text-[var(--color-text-main)]">Panel interno de debug</div>
                <div class="text-xs text-[var(--color-text-dim)]">Expone especialidad, confianza, decision, trace y conflictos para revision tecnica.</div>
              </div>
              <input type="checkbox" checked={$ragDebugPanel} onchange={(e) => toggleSetting('rag_debug_panel', e.currentTarget.checked)} />
            </label>

            <label class="flex items-center justify-between gap-4 bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
              <div>
                <div class="text-sm text-[var(--color-text-main)]">Mostrar badges de confianza</div>
                <div class="text-xs text-[var(--color-text-dim)]">Hace visible la especialidad y la confianza del RAG cuando la respuesta venga del flujo gobernado.</div>
              </div>
              <input type="checkbox" checked={$ragShowConfidenceBadge} onchange={(e) => toggleSetting('rag_show_confidence_badge', e.currentTarget.checked)} />
            </label>
          </div>
        </div>

        <div class="mb-8">
          <h3 class="text-sm font-semibold text-[var(--color-text-main)] mb-1">MegaBoss</h3>
          <p class="text-xs text-[var(--color-text-dim)] mb-3">Los comandos de máximo privilegio requieren esta clave.</p>

          <div class="bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)] mb-3">
            <div class="text-xs text-[var(--color-text-dim)] mb-2">
              Estado: {megabossStatus.unlocked ? 'Desbloqueado' : 'Bloqueado'} · Clave: {megabossStatus.password_set ? 'Configurada' : 'No configurada'}
            </div>
            <div class="flex gap-2">
              <input type="password" bind:value={megabossPassword} placeholder="Nueva clave MegaBoss" class="flex-1 bg-[var(--color-bg-input)] text-[var(--color-text-main)] border border-[var(--glass-border)] rounded-lg px-3 py-2 text-xs outline-none focus:border-[var(--color-brand-secondary)] transition-colors" />
              <button onclick={saveMegabossPassword} class="px-3 py-2 bg-[var(--color-brand-secondary)] text-white text-xs rounded-lg hover:brightness-110 transition-all">Guardar</button>
            </div>
          </div>

          <div class="bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
            <div class="flex gap-2 mb-2">
              <input type="password" bind:value={megabossUnlockPassword} placeholder="Clave MegaBoss" class="flex-1 bg-[var(--color-bg-input)] text-[var(--color-text-main)] border border-[var(--glass-border)] rounded-lg px-3 py-2 text-xs outline-none focus:border-[var(--color-brand-secondary)] transition-colors" />
              <input type="number" min="1" max="240" bind:value={megabossMinutes} class="w-20 bg-[var(--color-bg-input)] text-[var(--color-text-main)] border border-[var(--glass-border)] rounded-lg px-2 py-2 text-xs outline-none focus:border-[var(--color-brand-secondary)] transition-colors" />
              <button onclick={unlockMegaboss} class="px-3 py-2 bg-green-600 text-white text-xs rounded-lg hover:brightness-110 transition-all">Desbloquear</button>
            </div>
            {#if megabossMsg}
              <div class="text-xs text-[var(--color-text-dim)]">{megabossMsg}</div>
            {/if}
          </div>
        </div>

        <div class="mb-6">
          <h3 class="text-sm font-semibold text-[var(--color-text-main)] mb-1">API Keys</h3>
          <p class="text-xs text-[var(--color-text-dim)] mb-4">Las claves se almacenan encriptadas localmente.</p>

          <div class="space-y-4">
            {#each $models.filter((m) => !m.isLocal) as model}
              <div class="bg-[var(--color-bg-input)] rounded-xl p-4 border border-[var(--glass-border)]">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <span class="text-sm font-medium text-[var(--color-text-main)]">{model.name}</span>
                    <span class="text-[10px] text-[var(--color-text-dim)] ml-2">{model.provider}</span>
                  </div>
                  {#if model.hasApiKey}
                    <span class="text-xs text-[var(--color-brand-success)] bg-[var(--color-brand-success)]/10 px-2 py-0.5 rounded-full">Configurada</span>
                  {/if}
                </div>
                <div class="flex gap-2">
                  <input type="password" bind:value={apiKeys[model.id]} placeholder={model.hasApiKey ? '****** (cambiar)' : 'Pega tu API key aqui'} class="flex-1 bg-[var(--color-bg-input)] text-[var(--color-text-main)] border border-[var(--glass-border)] rounded-lg px-3 py-2 text-xs outline-none focus:border-[var(--color-brand-secondary)] transition-colors" />
                  <button onclick={() => saveApiKey(model.id)} disabled={!apiKeys[model.id]?.trim() || saving[model.id]} class="px-3 py-2 bg-[var(--color-brand-secondary)] text-white text-xs rounded-lg hover:brightness-110 disabled:opacity-30 disabled:cursor-not-allowed transition-all">{saving[model.id] ? '...' : 'Guardar'}</button>
                </div>
                {#if successMsg[model.id]}
                  <div class="text-xs mt-1 text-[var(--color-text-dim)]">{successMsg[model.id]}</div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="px-6 py-3 border-t border-[var(--glass-border)] flex justify-end">
        <button onclick={close} class="px-4 py-2 text-sm text-[var(--color-text-dim)] hover:text-[var(--color-text-main)] transition-colors">Cerrar</button>
      </div>
    </div>
  </div>
{/if}

