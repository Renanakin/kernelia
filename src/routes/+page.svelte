<script>
  import ChatWindow from '$lib/components/ChatWindow.svelte';
  import TelemetryPanel from '$lib/components/TelemetryPanel.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import AuditDashboard from '$lib/components/AuditDashboard.svelte';
  import { auditOpen } from '$lib/stores/settings.js';
  import { authStatus } from '$lib/stores/auth.js';
  import { onMount } from 'svelte';

  let mounted = false;
  let godMode = false;

  $: roleInfo = ($authStatus?.role === 'MegaBoss' || $authStatus?.role === 'Owner' || $authStatus?.profile === 'Superusuario')
    ? { name: 'MegaBoss Admin', color: 'border-red-500/40 bg-red-500/10 text-red-400', dot: 'bg-red-500' }
    : ($authStatus?.role === 'Admin' || $authStatus?.role === 'Operator' || $authStatus?.profile === 'Tecnico')
    ? { name: 'Técnico TI', color: 'border-green-500/40 bg-green-500/10 text-green-400', dot: 'bg-green-400' }
    : { name: 'Usuario Estándar', color: 'border-cyan-500/40 bg-cyan-500/10 text-cyan-400', dot: 'bg-cyan-400' };

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

    @keyframes spin { to { transform: rotate(360deg); } }

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

    .side-panel {
      background: rgba(10, 10, 12, 0.7);
      backdrop-filter: blur(40px);
      -webkit-backdrop-filter: blur(40px);
      border: 1px solid rgba(255, 255, 255, 0.05);
      border-radius: 24px;
      transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.6s ease;
      overflow: hidden;
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
    
    .noise {
      position: absolute;
      inset: 0;
      background-image: url('data:image/svg+xml;utf8,%3Csvg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noiseFilter"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.65" numOctaves="3" stitchTiles="stitch"/%3E%3C/filter%3E%3Crect width="100%25" height="100%25" filter="url(%23noiseFilter)"/%3E%3C/svg%3E');
      opacity: 0.03;
      mix-blend-mode: overlay;
      pointer-events: none;
      z-index: 1;
    }

    /* Force ChatWindow to be transparent and adapt to Zen Canvas */
    :global(.glass-panel) {
      background: transparent !important;
      border: none !important;
      box-shadow: none !important;
      backdrop-filter: none !important;
    }
  </style>
</svelte:head>


<div class="h-screen w-screen relative flex flex-col items-center justify-center bg-[#020202]">
  
  <!-- Dynamic Background -->
  <div class="aura-background"></div>
  <div class="noise"></div>

  <!-- Top Navigation (Minimal) -->
  <header class="absolute top-0 left-0 w-full p-6 flex justify-between items-center z-30 fade-up" class:active={mounted}>
    <div class="flex items-center gap-3">
      <img src="/KERNELIA_LOGO.svg" alt="KernelIA by Hackteck" class="h-8 w-auto object-contain" />
      <span class="font-medium tracking-wide">Kernel<span class="text-gray-400">IA</span></span>
    </div>
    
    <div class="flex items-center gap-3">
      <!-- Role Badge -->
      <div class="px-3 py-1.5 text-xs font-mono rounded-full border flex items-center gap-2 backdrop-blur-md transition-colors {roleInfo.color}">
        <div class="w-1.5 h-1.5 rounded-full animate-pulse {roleInfo.dot}"></div>
        <span>{roleInfo.name}</span>
      </div>

      <!-- God Mode Toggle -->
      <button 
        class="px-4 py-2 text-xs font-medium rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors flex items-center gap-2 z-50"
        on:click={() => godMode = !godMode}
      >
        <div class="w-2 h-2 rounded-full {godMode ? 'bg-[#00FF80]' : 'bg-gray-500'} transition-colors"></div>
        {godMode ? 'God Mode' : 'Focus Mode'}
      </button>
    </div>
  </header>

  <!-- Left Telemetry Panel (Progressive Disclosure) -->
  <aside 
    class="absolute left-6 top-24 bottom-24 w-[340px] side-panel z-20 flex flex-col"
    style="transform: {godMode ? 'translateX(0)' : 'translateX(-120%)'}; opacity: {godMode ? 1 : 0}; pointer-events: {godMode ? 'auto' : 'none'};"
  >
    <!-- We inject the TelemetryPanel here. It will stretch to fit the aside. -->
    <div class="w-full h-full p-2 overflow-hidden">
      <TelemetryPanel />
    </div>
  </aside>

  <!-- Central Canvas (Conversation) -->
  <!-- Modifying layout to center the chat window and give it space -->
  <main class="w-full max-w-4xl flex flex-col justify-center items-center px-4 z-10 flex-1 min-h-0 pt-24 pb-6 transition-all duration-700 fade-up" class:active={mounted} style="transition-delay: 0.2s;">
    <div class="w-full h-full min-h-0 relative transition-all duration-700" style="transform: {godMode ? 'scale(0.98)' : 'scale(1)'}">
      <!-- We use the existing ChatWindow here. CSS overrides make it frameless. -->
      <ChatWindow />
    </div>
  </main>

  <!-- Modals and external overlays -->
  <div class="absolute inset-0 z-[100] pointer-events-none">
    <div class="pointer-events-auto">
      <SettingsModal />
      <AuditDashboard show={$auditOpen} onClose={() => ($auditOpen = false)} />
    </div>
  </div>

  <!-- Brand Footer -->
  <div class="absolute right-6 bottom-5 z-40 pointer-events-none fade-up" class:active={mounted} style="transition-delay: 0.4s;">
    <div class="flex items-center gap-2 px-3 py-2 rounded-xl bg-white/5 border border-white/10 backdrop-blur-md shadow-lg">
      <img src="/LOGO_HACKTECK.png" alt="Hackteck" class="h-5 w-auto object-contain opacity-95 filter brightness-200" />
      <span class="text-[11px] tracking-[0.14em] uppercase text-white/60 font-mono">KernelIA</span>
    </div>
  </div>

</div>
