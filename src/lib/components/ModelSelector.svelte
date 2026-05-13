<script>
  import { onMount } from 'svelte';
  import { models, selectedModelId, settingsOpen, loadSettings } from '$lib/stores/settings.js';
  import { setModel } from '$lib/api/runtime/client.js';

  let dropdownOpen = $state(false);

  onMount(async () => {
    await loadSettings();
  });

  async function selectModel(modelId) {
    try {
      await setModel(modelId);
      $selectedModelId = modelId;
      $models = $models.map((m) => ({ ...m, selected: m.id === modelId }));
    } catch (e) {
      console.error('Failed to set model:', e);
    }
    dropdownOpen = false;
  }

  function getProviderIcon(provider) {
    const icons = {
      zhipu: 'CN',
      google: '◆',
      groq: '⚡',
      openrouter: '🌐',
      deepseek: '◈',
      ollama: '🦙',
    };
    return icons[provider] || '◌';
  }

  function handleClickOutside(e) {
    if (dropdownOpen && !e.target.closest('.model-selector')) {
      dropdownOpen = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="model-selector relative">
  <button
    onclick={() => (dropdownOpen = !dropdownOpen)}
    class="flex items-center gap-2 px-3 py-1.5 text-xs bg-[var(--color-bg-input)] border border-[var(--glass-border)] rounded-lg hover:border-[var(--color-brand-secondary)]/50 transition-colors"
  >
    {#if $models.length > 0}
      {@const current = $models.find((m) => m.id === $selectedModelId)}
      {#if current}
        <span>{getProviderIcon(current.provider)}</span>
        <span class="text-[var(--color-text-main)]">{current.name}</span>
        {#if !current.hasApiKey && !current.isLocal}
          <span class="text-[#ffb020]" title="API key no configurada">⚠</span>
        {/if}
      {/if}
    {:else}
      <span class="text-[var(--color-text-dim)]">Cargando modelos...</span>
    {/if}
    <svg
      class="w-3 h-3 text-[var(--color-text-dim)] transition-transform {dropdownOpen ? 'rotate-180' : ''}"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>

  {#if dropdownOpen}
    <div class="absolute top-full left-0 mt-1 w-72 bg-[var(--color-bg-card)] border border-[var(--glass-border)] rounded-xl shadow-xl z-50 overflow-hidden">
      <div class="p-2 text-xs text-[var(--color-text-dim)] border-b border-[var(--glass-border)] font-[var(--font-heading)]">
        MODELOS CLOUD
      </div>
      {#each $models.filter((m) => !m.isLocal) as model}
        <button
          onclick={() => selectModel(model.id)}
          class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm hover:bg-[rgba(255, 255, 255, 0.05)] transition-colors {model.id === $selectedModelId
            ? 'bg-[var(--color-brand-secondary)]/10 border-l-2 border-[var(--color-brand-secondary)]'
            : ''}"
        >
          <span class="text-base">{getProviderIcon(model.provider)}</span>
          <div class="flex-1 min-w-0">
            <div class="text-[var(--color-text-main)] truncate">{model.name}</div>
            <div class="text-[10px] text-[var(--color-text-dim)]">{model.provider}</div>
          </div>
          {#if !model.hasApiKey}
            <span class="text-[10px] text-[#ffb020] bg-[#ffb020]/10 px-1.5 py-0.5 rounded">Sin key</span>
          {:else}
            <span class="text-[var(--color-brand-success)]">✓</span>
          {/if}
        </button>
      {/each}

      {#if $models.some((m) => m.isLocal)}
        <div class="p-2 text-xs text-[var(--color-text-dim)] border-t border-[var(--glass-border)] font-[var(--font-heading)]">
          MODELOS LOCALES
        </div>
        {#each $models.filter((m) => m.isLocal) as model}
          <button
            onclick={() => selectModel(model.id)}
            class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm hover:bg-[rgba(255, 255, 255, 0.05)] transition-colors"
          >
            <span class="text-base">🦙</span>
            <div class="flex-1">
              <div class="text-[var(--color-text-main)]">{model.name}</div>
            </div>
          </button>
        {/each}
      {/if}

      <div class="border-t border-[var(--glass-border)]">
        <button
          onclick={() => {
            dropdownOpen = false;
            $settingsOpen = true;
          }}
          class="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--color-text-dim)] hover:bg-[rgba(255, 255, 255, 0.05)] transition-colors"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"></path>
          </svg>
          Configurar API keys
        </button>
      </div>
    </div>
  {/if}
</div>
