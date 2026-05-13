<script>
  import { toasts } from '$lib/stores/toastStore.js';
  import { flip } from 'svelte/animate';
  import { fly } from 'svelte/transition';
</script>

<div class="fixed bottom-6 right-6 z-[9999] flex flex-col gap-3 pointer-events-none">
  {#each $toasts as toast (toast.id)}
    <div 
      animate:flip={{ duration: 300 }}
      in:fly={{ x: 50, duration: 400 }}
      out:fly={{ x: 20, opacity: 0, duration: 300 }}
      class="pointer-events-auto px-4 py-3 rounded-xl border glass-panel shadow-2xl flex items-center gap-3 min-w-[280px]"
      class:border-green-500={toast.type === 'success'}
      class:border-red-500={toast.type === 'error'}
      class:border-blue-500={toast.type === 'info'}
    >
      {#if toast.type === 'success'}
        <div class="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.5)]"></div>
      {:else if toast.type === 'error'}
        <div class="w-2 h-2 rounded-full bg-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]"></div>
      {:else}
        <div class="w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]"></div>
      {/if}
      
      <p class="text-[11px] font-bold text-white uppercase tracking-wider">{toast.message}</p>
      
      <button 
        onclick={() => toasts.remove(toast.id)}
        class="ml-auto text-white/30 hover:text-white transition-colors"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/each}
</div>
