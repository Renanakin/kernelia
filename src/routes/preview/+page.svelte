<script>
  import { onMount } from 'svelte';

  let mounted = false;
  let showPanels = false;

  onMount(() => { 
    setTimeout(() => { mounted = true; }, 100); 
  });
</script>

<svelte:head>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600&family=JetBrains+Mono:wght@300;400&display=swap');

    :global(body) {
      font-family: 'Outfit', sans-serif;
      background-color: #020202;
      color: #FFFFFF;
      overflow: hidden;
    }

    /* Dynamic Mesh Gradient (The "Aura") */
    .aura-background {
      position: absolute;
      top: -50%;
      left: -50%;
      width: 200%;
      height: 200%;
      background: 
        radial-gradient(circle at 50% 50%, rgba(20, 40, 100, 0.4) 0%, transparent 40%),
        radial-gradient(circle at 80% 20%, rgba(0, 255, 128, 0.15) 0%, transparent 30%),
        radial-gradient(circle at 20% 80%, rgba(120, 0, 255, 0.15) 0%, transparent 30%);
      filter: blur(100px);
      animation: aura-shift 20s infinite alternate ease-in-out;
      z-index: 0;
      pointer-events: none;
    }

    @keyframes aura-shift {
      0% { transform: translate(0, 0) scale(1); }
      100% { transform: translate(-10%, -5%) scale(1.1); }
    }

    /* Raycast-style Command Bar */
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

    /* Floating Side Panels (Progressive Disclosure) */
    .side-panel {
      background: rgba(10, 10, 12, 0.7);
      backdrop-filter: blur(40px);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 24px;
      transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.6s ease;
    }

    .fade-up {
      opacity: 0;
      transform: translateY(40px);
      transition: all 1s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .fade-up.active {
      opacity: 1;
      transform: translateY(0);
    }
    
    /* Noise overlay for texture */
    .noise {
      position: absolute;
      inset: 0;
      background-image: url('data:image/svg+xml;utf8,%3Csvg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noiseFilter"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch"/%3E%3C/filter%3E%3Crect width="100%25" height="100%25" filter="url(%23noiseFilter)"/%3E%3C/svg%3E');
      opacity: 0.03;
      mix-blend-mode: overlay;
      pointer-events: none;
      z-index: 1;
    }
  </style>
</svelte:head>

<div class="h-screen w-screen relative flex flex-col items-center justify-center">
  <!-- Dynamic Background -->
  <div class="aura-background"></div>
  <div class="noise"></div>

  <!-- Top Navigation (Minimal) -->
  <header class="absolute top-0 left-0 w-full p-6 flex justify-between items-center z-20 fade-up" class:active={mounted}>
    <div class="flex items-center gap-3">
      <div class="w-6 h-6 rounded bg-white text-black flex items-center justify-center font-bold text-xs">NX</div>
      <span class="font-medium tracking-wide">Kernel<span class="text-gray-400">AI</span></span>
    </div>
    
    <!-- PM/UX Feature: Toggle Complexity -->
    <button 
      class="px-4 py-2 text-xs font-medium rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors flex items-center gap-2"
      on:click={() => showPanels = !showPanels}
    >
      <div class="w-2 h-2 rounded-full {showPanels ? 'bg-[#00FF80]' : 'bg-gray-500'} transition-colors"></div>
      {showPanels ? 'God Mode' : 'Focus Mode'}
    </button>
  </header>

  <!-- Left Telemetry Panel (Hidden by default - Progressive Disclosure) -->
  <aside 
    class="absolute left-6 top-24 bottom-24 w-80 side-panel p-6 flex flex-col z-20"
    style="transform: {showPanels ? 'translateX(0)' : 'translateX(-120%)'}; opacity: {showPanels ? 1 : 0}; pointer-events: {showPanels ? 'auto' : 'none'};"
  >
    <div class="flex items-center gap-2 mb-8">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#00FF80" stroke-width="2"><path d="M22 12h-4l-3 9L9 3l-3 9H2"></path></svg>
      <span class="text-xs font-mono text-gray-400 uppercase tracking-widest">Live Telemetry</span>
    </div>
    
    <div class="space-y-8 flex-1">
      <div>
        <div class="flex justify-between text-xs mb-2"><span class="text-gray-400">Compute Core</span><span>32%</span></div>
        <div class="h-1 bg-white/10 rounded-full overflow-hidden"><div class="h-full w-[32%] bg-[#00FF80]"></div></div>
      </div>
      <div>
        <div class="flex justify-between text-xs mb-2"><span class="text-gray-400">Memory Matrix</span><span>68%</span></div>
        <div class="h-1 bg-white/10 rounded-full overflow-hidden"><div class="h-full w-[68%] bg-[#00FF80]"></div></div>
      </div>
      
      <!-- Mini Chart -->
      <div class="pt-6 border-t border-white/10">
        <span class="text-[10px] font-mono text-gray-500 uppercase">Network Traffic</span>
        <div class="h-16 flex items-end gap-1 mt-2">
          {#each Array(15).fill(0).map(() => Math.random() * 100) as height}
            <div class="flex-1 bg-white/10 rounded-t-sm hover:bg-[#00FF80] transition-colors" style="height: {height}%"></div>
          {/each}
        </div>
      </div>
    </div>
  </aside>

  <!-- Right Audit Panel -->
  <aside 
    class="absolute right-6 top-24 bottom-24 w-80 side-panel p-6 flex flex-col z-20"
    style="transform: {showPanels ? 'translateX(0)' : 'translateX(120%)'}; opacity: {showPanels ? 1 : 0}; pointer-events: {showPanels ? 'auto' : 'none'};"
  >
    <div class="flex items-center gap-2 mb-6">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-gray-400"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
      <span class="text-xs font-mono text-gray-400 uppercase tracking-widest">Security Audit</span>
    </div>
    
    <div class="flex-1 overflow-y-auto space-y-3 font-mono text-[11px]">
      <div class="p-3 bg-white/5 rounded-xl border border-white/5 text-gray-300">
        <span class="text-gray-500 block mb-1">10:42:01</span>
        [SYS] Authentication matrix updated.
      </div>
      <div class="p-3 bg-white/5 rounded-xl border border-white/5 text-gray-300">
        <span class="text-gray-500 block mb-1">10:41:15</span>
        [AI] Context window flushed.
      </div>
      <div class="p-3 bg-[#00FF80]/10 rounded-xl border border-[#00FF80]/20 text-[#00FF80]">
        <span class="text-[#00FF80]/60 block mb-1">10:40:00</span>
        [AGENT] Payload successfully deployed to node Alpha.
      </div>
    </div>
  </aside>

  <!-- Central Canvas (Conversation) -->
  <main class="w-full max-w-3xl flex flex-col justify-center items-center px-8 z-10 flex-1 fade-up" class:active={mounted} style="transition-delay: 0.2s;">
    
    <!-- AI Response Stage -->
    <div class="w-full text-center space-y-6 transform transition-all duration-700" style="transform: {showPanels ? 'scale(0.95) translateY(-20px)' : 'scale(1) translateY(0)'}">
      
      <!-- Origin Label -->
      <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-white/10 bg-white/5 text-xs font-mono text-gray-300 backdrop-blur-md">
        <div class="w-1.5 h-1.5 bg-[#00FF80] rounded-full animate-pulse"></div>
        DeepSeek V3 Agent
      </div>

      <!-- Hero Typography (Design Expert: Emotional Impact) -->
      <h2 class="text-4xl md:text-5xl font-light leading-tight text-transparent bg-clip-text bg-gradient-to-br from-white to-gray-400">
        He optimizado la <span class="font-medium text-white">carga de servidores</span> y purgado <span class="font-medium text-[#00FF80]">14.2GB</span> de logs obsoletos.
      </h2>
      
      <p class="text-lg text-gray-400 font-light max-w-xl mx-auto">
        Todos los sistemas operan a máxima capacidad. No se requieren acciones adicionales en este momento.
      </p>
    </div>

  </main>

  <!-- Bottom Command Dock (Raycast Style) -->
  <footer class="absolute bottom-12 w-full max-w-2xl px-6 z-30 fade-up" class:active={mounted} style="transition-delay: 0.4s;">
    <div class="command-dock flex items-center p-2 pl-6">
      <!-- Icon -->
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#00FF80" stroke-width="2" class="opacity-80"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
      
      <input 
        type="text" 
        placeholder="Pregunta a la IA, invoca una herramienta, o presiona '/' para comandos..." 
        class="w-full bg-transparent border-none outline-none text-white px-4 py-4 font-light text-lg placeholder:text-gray-500"
      >
      
      <!-- Right Side Actions -->
      <div class="flex gap-2 pr-2">
        <kbd class="hidden md:flex items-center justify-center px-2.5 py-1 bg-white/10 rounded-md border border-white/10 text-xs font-mono text-gray-400">Ctrl+K</kbd>
        <button class="w-10 h-10 flex items-center justify-center bg-white text-black rounded-xl hover:scale-105 transition-transform">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
        </button>
      </div>
    </div>
    <div class="text-center mt-4">
      <span class="text-[10px] font-mono text-gray-500 uppercase tracking-wider">Nexus Lite Kernel v2.1.0</span>
    </div>
  </footer>

</div>
