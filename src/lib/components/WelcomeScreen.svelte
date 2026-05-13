<script>
  const quickActions = [
    { icon: 'NET', label: 'Diagnostico de Red', message: 'Ejecuta un diagnostico completo de mi conexion a internet.' },
    { icon: 'RPT', label: 'Generar Reporte', message: 'Genera un reporte tecnico de soporte para Hackteck.' },
    { icon: 'CPU', label: 'Procesos activos', message: 'Ejecuta y muestrame los procesos que mas recursos consumen.' },
    { icon: 'DIR', label: 'Explorar archivos', message: 'Ejecuta y lista los archivos del escritorio del usuario actual.' },
  ];

  let { onsend } = $props();
</script>

<div class="flex flex-col items-center justify-center h-full px-6 py-12 select-none relative z-10 w-full">
  <div class="w-full text-center space-y-6 mb-16 transform transition-all duration-700">
    <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-white/10 bg-white/5 text-xs font-mono text-gray-300 backdrop-blur-md mb-4">
      <div class="w-1.5 h-1.5 bg-[#00FF80] rounded-full animate-pulse"></div>
      Kernel IA - Sistemas optimos
    </div>

    <h2 class="text-4xl md:text-5xl font-light leading-tight text-transparent bg-clip-text bg-gradient-to-br from-white to-gray-400">
      Bienvenido a la <span class="font-medium text-white">siguiente generacion</span> de diagnostico inteligente.
    </h2>

    <p class="text-lg text-gray-400 font-light max-w-xl mx-auto">
      Todos los sistemas operan a maxima capacidad. Selecciona una accion rapida o invoca un comando.
    </p>
  </div>

  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl w-full">
    {#each quickActions as action}
      <button
        onclick={() => onsend(action.message)}
        class="flex flex-col items-center justify-center gap-2 p-4 rounded-2xl bg-white/5 border border-white/5 hover:bg-white/10 hover:border-white/10 transition-all duration-300 group"
      >
        <div class="text-xs font-mono group-hover:scale-110 transition-transform duration-300 opacity-80 group-hover:opacity-100">{action.icon}</div>
        <span class="text-xs text-gray-300 font-medium tracking-wide opacity-80 group-hover:opacity-100">{action.label}</span>
      </button>
    {/each}
  </div>
</div>
