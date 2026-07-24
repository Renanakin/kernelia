<script>
  import { messages, isLoading, clearMessages, addUserMessage, addLoadingMessage, resolveLoadingMessage } from '$lib/stores/chat.js';
  import { clearChat, sendMessage } from '$lib/api/runtime/client.js';
  import { settingsOpen } from '$lib/stores/settings.js';
  import { authReady, authStatus } from '$lib/stores/auth.js';
  import MessageBubble from './MessageBubble.svelte';
  import InputBar from './InputBar.svelte';
  import ModelSelector from './ModelSelector.svelte';
  import WelcomeScreen from './WelcomeScreen.svelte';
  import UserManualModal from './UserManualModal.svelte';

  import { auditOpen } from '$lib/stores/settings.js';
  import { tick } from 'svelte';
  import { fade, slide } from 'svelte/transition';

  let chatContainer = $state(null);
  let manualOpen = $state(false);
  const hasMessages = $derived($messages.length > 0);

  function withUiTimeout(promise, timeoutMs = 120000) {
    return Promise.race([
      promise,
      new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Tiempo de espera agotado en UI (120s).')), timeoutMs);
      })
    ]);
  }

  // Auto-scroll al fondo cuando llegan nuevos mensajes
  $effect(() => {
    if ($messages.length > 0 && chatContainer) {
      tick().then(() => {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      });
    }
  });

  async function handleQuickAction(message) {
    addUserMessage(message);
    const loadingId = addLoadingMessage();

    if (!$authReady) {
      resolveLoadingMessage(
        loadingId,
        '',
        [],
        null,
        'KernelIA aun esta validando la sesion. Espera unos segundos e intenta de nuevo.'
      );
      return;
    }

    if (!$authStatus.is_authenticated) {
      resolveLoadingMessage(
        loadingId,
        '',
        [],
        null,
        'Debes iniciar sesion para ejecutar acciones rapidas.'
      );
      return;
    }

    $isLoading = true;

    try {
      const response = await withUiTimeout(
        sendMessage(message)
      );
      resolveLoadingMessage(
        loadingId,
        response.text,
        response.tools_used,
        response.model,
        response.error,
        response.rag_context,
        response.rag_comparison
      );
    } catch (e) {
      resolveLoadingMessage(loadingId, '', [], null, String(e?.message || e));
    } finally {
      $isLoading = false;
    }
  }

  async function handleClearChat() {
    try {
      await clearChat();
      clearMessages();
    } catch (e) {
      console.error('Failed to clear chat:', e);
    }
  }
</script>

<div class="flex flex-col h-full min-h-0 overflow-hidden text-[var(--color-text-main)] font-[var(--font-body)]">
  <!-- Interactive Mesh Background managed via app.css ::before -->

  <!-- Minimalist Action Bar -->
  <div class="flex items-center justify-end gap-3 px-4 py-2 shrink-0 z-20 opacity-60 hover:opacity-100 transition-opacity duration-300">
    <ModelSelector />

    {#if hasMessages}
      <button
        id="btn-clear-chat"
        onclick={handleClearChat}
        class="p-2 text-gray-400 hover:text-white rounded-lg transition-colors"
        title="Limpiar chat"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"></path>
        </svg>
      </button>
    {/if}

    <button
      id="btn-manual"
      onclick={() => (manualOpen = true)}
      class="p-2 text-gray-400 hover:text-white rounded-lg transition-colors"
      title="Manual de usuario"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
      </svg>
    </button>

    <button
      id="btn-audit"
      onclick={() => ($auditOpen = true)}
      class="p-2 text-gray-400 hover:text-white rounded-lg transition-colors"
      title="Historial de Auditoría"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
      </svg>
    </button>

    <button
      id="btn-settings"
      onclick={() => ($settingsOpen = true)}
      class="p-2 text-gray-400 hover:text-white rounded-lg transition-colors"
      title="Configuración"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"></path>
      </svg>
    </button>
  </div>

  <!-- Chat Area -->
  <main id="chat-scroll-container" class="flex-1 min-h-0 overflow-y-auto custom-scrollbar relative z-10" bind:this={chatContainer}>
    {#if hasMessages}
      <div class="max-w-4xl mx-auto py-8 space-y-6">
        {#each $messages as message (message.id)}
          <div in:fade={{ duration: 300 }}>
            <MessageBubble {message} />
          </div>
        {/each}
      </div>
    {:else}
      <div in:fade={{ duration: 600 }}>
        <WelcomeScreen onsend={handleQuickAction} />
      </div>
    {/if}
  </main>

  <!-- Input Bar -->
  <div class="relative z-20 max-w-4xl mx-auto w-full mt-4 shrink-0">
    <InputBar />
  </div>


</div>
<UserManualModal show={manualOpen} onClose={() => (manualOpen = false)} />

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: var(--color-brand-primary);
  }
</style>
