<script>
  import { inputText, isLoading, addUserMessage, addLoadingMessage, resolveLoadingMessage } from '$lib/stores/chat.js';
  import { sendMessage } from '$lib/api/runtime/client.js';
  import { authReady, authStatus } from '$lib/stores/auth.js';

  let textareaEl = $state(null);

  function withUiTimeout(promise, timeoutMs = 120000) {
    return Promise.race([
      promise,
      new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Tiempo de espera agotado en UI (120s).')), timeoutMs);
      })
    ]);
  }

  function autoResize() {
    if (textareaEl) {
      textareaEl.style.height = 'auto';
      textareaEl.style.height = Math.min(textareaEl.scrollHeight, 150) + 'px';
    }
  }

  async function handleSend() {
    const text = $inputText.trim();
    if (!text || $isLoading) return;

    if (!$authReady) {
      addUserMessage(text);
      resolveLoadingMessage(
        addLoadingMessage(),
        '',
        [],
        null,
        'KernelIA aun esta validando la sesion. Espera unos segundos e intenta de nuevo.'
      );
      return;
    }

    if (!$authStatus.is_authenticated) {
      addUserMessage(text);
      resolveLoadingMessage(
        addLoadingMessage(),
        '',
        [],
        null,
        'Debes iniciar sesion para usar el chat de KernelIA.'
      );
      return;
    }

    // Limpiar input
    $inputText = '';
    if (textareaEl) {
      textareaEl.style.height = 'auto';
    }

    // Agregar mensaje del usuario
    addUserMessage(text);

    // Mostrar indicador de carga
    const loadingId = addLoadingMessage();
    $isLoading = true;

    try {
      const response = await withUiTimeout(
        sendMessage(text)
      );
      resolveLoadingMessage(
        loadingId,
        response.text,
        response.tools_used,
        response.model,
        response.error
      );
    } catch (e) {
      resolveLoadingMessage(loadingId, '', [], null, String(e?.message || e));
    } finally {
      $isLoading = false;
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }
</script>

<div class="command-dock flex items-end p-2 pl-6 mx-auto w-full relative z-30">
  <div class="pb-3 shrink-0">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00FF80" stroke-width="2" class="opacity-80"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
  </div>
  
  <textarea
    bind:this={textareaEl}
    bind:value={$inputText}
    oninput={autoResize}
    onkeydown={handleKeydown}
    placeholder="Pregunta a la IA, invoca una herramienta..."
    rows="1"
    disabled={$isLoading || !$authReady || !$authStatus.is_authenticated}
    id="chat-input"
    class="w-full bg-transparent border-none outline-none text-white px-4 py-3 font-light text-lg placeholder:text-gray-500 resize-none custom-scrollbar"
    style="max-height: 150px;"
  ></textarea>
  
  <div class="flex gap-2 pr-2 pb-2 shrink-0">
    <kbd class="hidden md:flex items-center justify-center px-2.5 py-1 bg-white/10 rounded-md border border-white/10 text-xs font-mono text-gray-400">↵</kbd>
    <button
      id="btn-send-message"
      onclick={handleSend}
      disabled={$isLoading || !$inputText.trim() || !$authReady || !$authStatus.is_authenticated}
      class="w-10 h-10 flex items-center justify-center bg-white text-black rounded-xl hover:scale-105 transition-transform disabled:opacity-30 disabled:cursor-not-allowed"
      title="Ejecutar"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
    </button>
  </div>
</div>

<style>
  .command-dock {
    background: rgba(15, 15, 18, 0.6);
    backdrop-filter: blur(30px);
    -webkit-backdrop-filter: blur(30px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 20px 40px rgba(0,0,0,0.4), inset 0 1px 0 rgba(255,255,255,0.1);
    border-radius: 20px;
    transition: all 0.3s ease;
  }
  
  .command-dock:focus-within {
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 20px 50px rgba(0, 255, 128, 0.1), inset 0 1px 0 rgba(255,255,255,0.2);
  }

  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 255, 128, 0.5);
  }
</style>
