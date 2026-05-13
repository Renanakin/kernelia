<script>
  import { renderMarkdown } from '$lib/utils/markdown.js';
  import { formatToolName, summarizeToolArgs } from '$lib/utils/formatting.js';

  let { message } = $props();

  const isUser = $derived(message.role === 'user');
  const isSystem = $derived(message.role === 'system');
  const hasTools = $derived(message.toolsUsed && message.toolsUsed.length > 0);
  const hasError = $derived(!!message.error);
</script>

<div class="flex flex-col {isUser ? 'items-end' : 'items-start'} group w-full mb-8">
  {#if !isUser && !isSystem}
    <!-- Origin Label (Zen Canvas Style) -->
    <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-white/10 bg-white/5 text-xs font-mono text-gray-300 backdrop-blur-md mb-4 shadow-sm">
      <div class="w-1.5 h-1.5 bg-[#00FF80] rounded-full animate-pulse"></div>
      {message.model ? `Kernel IA (${message.model})` : 'Kernel IA'}
    </div>
  {/if}

  <div
    class="max-w-[90%] transition-all duration-300 {isUser
      ? 'bg-white/10 backdrop-blur-md border border-white/10 text-white px-6 py-4 rounded-2xl rounded-tr-sm shadow-sm'
      : isSystem
        ? 'bg-transparent text-gray-500 font-mono text-xs w-full text-center'
        : 'bg-transparent text-gray-200'}"
  >
    {#if message.currentTool}
      <div class="flex items-center gap-3 text-[var(--color-brand-success)] py-2 mb-2">
        <div class="relative w-4 h-4 shrink-0">
          <div class="absolute inset-0 border-2 border-[var(--color-brand-success)]/20 rounded-full"></div>
          <div class="absolute inset-0 border-2 border-[var(--color-brand-success)] rounded-full border-t-transparent animate-spin"></div>
        </div>
        <span class="text-xs font-mono tracking-widest uppercase truncate">EJECUTANDO HERRAMIENTA: {formatToolName(message.currentTool)}...</span>
      </div>
    {/if}

    {#if message.isLoading && !message.currentTool}
      <div class="flex items-center gap-3 text-gray-400 py-2 mt-2">
        <div class="relative w-4 h-4">
          <div class="absolute inset-0 border-2 border-gray-400/20 rounded-full"></div>
          <div class="absolute inset-0 border-2 border-gray-400 rounded-full border-t-transparent animate-spin"></div>
        </div>
        <span class="text-xs font-mono tracking-wider">
          {message.content ? 'PROCESANDO...' : 'PENSANDO...'}
        </span>
      </div>
    {:else if hasError}
      <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 mb-2">
        <div class="flex items-center gap-2 text-red-400 text-sm font-bold mb-1">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          SISTEMA ERROR
        </div>
        <span class="text-sm text-red-200/80">{message.error}</span>
      </div>
    {:else if message.content}
      <!-- Use typography sizing based on Zen Canvas for assistant -->
      <div class="markdown-content leading-relaxed {isUser ? 'text-sm' : 'text-lg md:text-xl font-light text-gray-200'}">
        {@html renderMarkdown(message.content)}
      </div>
    {/if}

    {#if hasTools}
      <div class="mt-4 pt-4 border-t border-white/5 space-y-2 max-w-sm">
        {#each message.toolsUsed as tool}
          <div class="flex items-center gap-3 bg-white/5 hover:bg-white/10 rounded-xl px-4 py-3 border border-white/5 transition-colors group/tool cursor-pointer">
            <div class="p-2 rounded-lg bg-[#00FF80]/10 text-[#00FF80]">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="16 18 22 12 16 6"></polyline>
                <polyline points="8 6 2 12 8 18"></polyline>
              </svg>
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-[10px] font-bold text-[#00FF80] uppercase tracking-tighter">TOOL EXECUTED</span>
              <span class="text-xs text-white font-mono truncate">{formatToolName(tool.name)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <div class="flex {isUser ? 'justify-end' : 'justify-start'} mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
      <span class="text-[10px] text-gray-500 font-mono uppercase tracking-widest">{message.timestamp}</span>
    </div>
  </div>
</div>
