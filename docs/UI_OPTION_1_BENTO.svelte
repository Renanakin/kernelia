<script>
  import { onMount } from 'svelte';
  let mounted = false;
  onMount(() => { setTimeout(() => { mounted = true; }, 50); });
</script>

<svelte:head>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=JetBrains+Mono:wght@400;700&display=swap');

    :global(body) {
      font-family: 'Inter', sans-serif;
      background-color: #050505;
      color: #E0E0E0;
    }

    /* Spatial Bento Card Styles */
    .bento-card {
      background: rgba(18, 18, 18, 0.4);
      backdrop-filter: blur(40px);
      -webkit-backdrop-filter: blur(40px);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 24px;
      box-shadow: 0 4px 30px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.1);
      transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.4s ease;
      position: relative;
      overflow: hidden;
    }

    .bento-card:hover {
      border: 1px solid rgba(255, 255, 255, 0.15);
      box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8), inset 0 1px 0 rgba(255, 255, 255, 0.2);
    }

    /* Ambient Glows */
    .glow-accent {
      position: absolute;
      width: 150px;
      height: 150px;
      background: radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%);
      top: -75px;
      left: 50%;
      transform: translateX(-50%);
      pointer-events: none;
    }

    /* Staggered Entrance */
    .fade-in-up {
      opacity: 0;
      transform: translateY(30px);
      transition: all 0.8s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .fade-in-up.visible {
      opacity: 1;
      transform: translateY(0);
    }
  </style>
</svelte:head>

<div class="h-screen w-screen p-6 bg-[#050505] overflow-hidden flex flex-col gap-6">
  
  <!-- Subtle Background Gradient -->
  <div class="fixed inset-0 pointer-events-none z-0" style="background: radial-gradient(circle at 50% 0%, rgba(255, 255, 255, 0.03) 0%, transparent 50%);"></div>

  <!-- Header -->
  <header class="z-10 flex justify-between items-center px-4 fade-in-up" class:visible={mounted} style="transition-delay: 0.1s;">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-full bg-white text-black flex items-center justify-center font-bold font-mono">NX</div>
      <h1 class="text-xl font-medium tracking-tight">Nexus<span class="text-gray-500 font-light">Lite</span></h1>
    </div>
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2">
        <span class="relative flex h-2 w-2">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-40"></span>
          <span class="relative inline-flex rounded-full h-2 w-2 bg-white"></span>
        </span>
        <span class="text-xs font-mono text-gray-400">DeepSeek Core</span>
      </div>
    </div>
  </header>

  <!-- Bento Grid Layout -->
  <main class="z-10 flex-1 grid grid-cols-12 grid-rows-6 gap-6 min-h-0">
    
    <!-- Large Center Piece: The Conversation / AI Input -->
    <div class="bento-card col-span-8 row-span-6 flex flex-col p-8 fade-in-up" class:visible={mounted} style="transition-delay: 0.2s;">
      <div class="glow-accent"></div>
      
      <!-- Chat History -->
      <div class="flex-1 overflow-y-auto flex flex-col gap-8 pb-4 pr-4">
        <!-- User -->
        <div class="flex flex-col gap-2">
          <span class="text-[10px] font-mono text-gray-500 uppercase tracking-wider">User Query</span>
          <p class="text-lg text-gray-300 font-light leading-relaxed">
            Ejecuta un diagnóstico completo de los servicios críticos y muéstrame las alertas.
          </p>
        </div>
        
        <!-- AI Response (High typography contrast) -->
        <div class="flex flex-col gap-3">
          <div class="flex items-center gap-2">
             <div class="w-1.5 h-1.5 bg-white rounded-full"></div>
             <span class="text-[10px] font-mono text-white uppercase tracking-wider">Kernel AI</span>
          </div>
          <h2 class="text-3xl font-normal text-white leading-tight tracking-tight">
            Diagnóstico completado en <span class="text-gray-400">0.42s</span>.
          </h2>
          <p class="text-xl text-gray-400 font-light leading-relaxed max-w-2xl mt-2">
            He detectado latencia inusual en el microservicio de bases de datos. Los demás sistemas operan dentro de los parámetros normales.
          </p>
          
          <!-- Tool Execution Pill -->
          <div class="mt-4 inline-flex items-center gap-3 px-4 py-2 rounded-full border border-white/10 bg-white/5 w-max">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
            <span class="text-xs font-mono text-gray-300">system_diagnostic.exe</span>
            <span class="text-xs text-white bg-white/20 px-2 py-0.5 rounded-full">Success</span>
          </div>
        </div>
      </div>

      <!-- Input Field -->
      <div class="mt-6 relative">
        <input 
          type="text" 
          placeholder="Comanda a Nexus..." 
          class="w-full bg-white/5 border border-white/10 rounded-2xl px-6 py-5 text-lg text-white font-light placeholder:text-gray-600 focus:outline-none focus:bg-white/10 focus:border-white/30 transition-all shadow-inner"
        >
        <button class="absolute right-4 top-1/2 -translate-y-1/2 w-10 h-10 bg-white text-black rounded-xl flex items-center justify-center hover:scale-105 transition-transform shadow-[0_0_20px_rgba(255,255,255,0.3)]">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
        </button>
      </div>
    </div>

    <!-- Top Right: System Status -->
    <div class="bento-card col-span-4 row-span-2 p-6 flex flex-col justify-between fade-in-up" class:visible={mounted} style="transition-delay: 0.3s;">
      <div class="flex justify-between items-start">
        <span class="text-[10px] font-mono text-gray-500 uppercase tracking-wider">System Status</span>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-gray-400"><path d="M22 12h-4l-3 9L9 3l-3 9H2"></path></svg>
      </div>
      
      <div class="flex items-end gap-4 mt-4">
        <div class="flex flex-col">
          <span class="text-4xl font-light text-white">24<span class="text-lg text-gray-500">%</span></span>
          <span class="text-xs text-gray-500 font-mono mt-1">CPU LOAD</span>
        </div>
        <div class="flex flex-col">
          <span class="text-4xl font-light text-white">1.2<span class="text-lg text-gray-500">gb</span></span>
          <span class="text-xs text-gray-500 font-mono mt-1">RAM USAGE</span>
        </div>
      </div>
    </div>

    <!-- Middle Right: Audit Stream -->
    <div class="bento-card col-span-4 row-span-4 p-6 flex flex-col fade-in-up" class:visible={mounted} style="transition-delay: 0.4s;">
      <div class="flex justify-between items-center mb-6">
        <span class="text-[10px] font-mono text-gray-500 uppercase tracking-wider">Audit Stream</span>
        <div class="w-1.5 h-1.5 rounded-full bg-white"></div>
      </div>

      <div class="flex-1 overflow-y-auto flex flex-col gap-4 font-mono text-xs">
        <div class="flex flex-col gap-1">
          <span class="text-gray-600">14:22:01.004</span>
          <span class="text-gray-300 bg-white/5 p-2 rounded-lg border border-white/5">GET /api/v1/metrics [200 OK]</span>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-gray-600">14:21:45.112</span>
          <span class="text-gray-300 bg-white/5 p-2 rounded-lg border border-white/5">Invoking skill: sys_diagnostics</span>
        </div>
        <div class="flex flex-col gap-1 opacity-50">
          <span class="text-gray-600">14:20:10.992</span>
          <span class="text-gray-300 p-2">WebSocket connected to wss://nexus.local</span>
        </div>
      </div>
    </div>

  </main>

</div>
