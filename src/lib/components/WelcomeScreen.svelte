<script>
  import { authStatus } from '$lib/stores/auth.js';

  let { onsend } = $props();

  const viewerActions = [
    { icon: 'HP', label: 'Salud del PC', message: '¿Cual es el estado general de salud de mi equipo?' },
    { icon: 'DISK', label: 'Espacio Libre', message: '¿Cuanto espacio libre tengo en mis discos?' },
    { icon: 'UPD', label: 'Actualizaciones', message: '¿Hay actualizaciones de Windows pendientes?' },
    { icon: 'DOC', label: 'Reporte Sistema', message: 'Genera un resumen accesible de este equipo.' },
  ];

  const techActions = [
    { icon: 'NET', label: 'Diagnostico de Red', message: 'Ejecuta un diagnostico completo de la conexion de red.' },
    { icon: 'SVC', label: 'Servicio Spooler', message: 'Verifica el estado del servicio spooler de impresoras.' },
    { icon: 'CPU', label: 'Top Procesos', message: 'Muestra los procesos con mayor consumo de CPU y RAM.' },
    { icon: 'DEV', label: 'Revisar Drivers', message: 'Busca dispositivos de hardware con errores de controlador (Code 43).' },
  ];

  const megabossActions = [
    { icon: 'DISM', label: 'Reparacion DISM', message: 'Ejecuta chequeo de salud y reparacion de imagen de Windows con DISM.' },
    { icon: 'DNS', label: 'Flush DNS', message: 'Limpia la cache de DNS y reinicia adaptadores de red.' },
    { icon: 'SPOOL', label: 'Limpiar Spooler', message: 'Elimina trabajos colgados de impresion y reinicia el servicio Spooler.' },
    { icon: 'AUDIT', label: 'Kernel Audit', message: 'Muestra el registro completo de auditoria de seguridad de KernelIA.' },
  ];

  const isMegaboss = $derived(
    $authStatus?.role === 'MegaBoss' || $authStatus?.role === 'Owner' || $authStatus?.profile === 'Superusuario'
  );

  const isTech = $derived(
    $authStatus?.role === 'Admin' || $authStatus?.role === 'Operator' || $authStatus?.profile === 'Tecnico'
  );

  const currentActions = $derived(
    isMegaboss
      ? megabossActions
      : isTech
      ? techActions
      : viewerActions
  );

  const roleBadge = $derived(
    isMegaboss
      ? { text: 'MegaBoss Admin (R4)', color: 'border-red-500/30 bg-red-500/10 text-red-400' }
      : isTech
      ? { text: 'Tecnico TI (R2-R3)', color: 'border-green-500/30 bg-green-500/10 text-green-400' }
      : { text: 'Usuario Estandar (R1)', color: 'border-cyan-500/30 bg-cyan-500/10 text-cyan-400' }
  );
</script>

<div class="flex flex-col items-center justify-center h-full px-6 py-12 select-none relative z-10 w-full">
  <div class="w-full text-center space-y-6 mb-12 transform transition-all duration-700">
    <div class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full border text-xs font-mono backdrop-blur-md mb-2 {roleBadge.color}">
      <div class="w-2 h-2 rounded-full animate-pulse bg-current"></div>
      {roleBadge.text}
    </div>

    <h2 class="text-4xl md:text-5xl font-light leading-tight text-transparent bg-clip-text bg-gradient-to-br from-white to-gray-400">
      Bienvenido a la <span class="font-medium text-white">siguiente generacion</span> de diagnostico inteligente.
    </h2>

    <p class="text-lg text-gray-400 font-light max-w-xl mx-auto">
      Experiencia optimizada para perfil <strong class="text-white">{roleBadge.text}</strong>. Selecciona una accion rapida o escribe una consulta.
    </p>
  </div>

  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl w-full">
    {#each currentActions as action}
      <button
        onclick={() => onsend(action.message)}
        class="flex flex-col items-center justify-center gap-2 p-4 rounded-2xl bg-white/5 border border-white/5 hover:bg-white/10 hover:border-white/15 transition-all duration-300 group"
      >
        <div class="text-xs font-mono group-hover:scale-110 transition-transform duration-300 opacity-80 group-hover:opacity-100">{action.icon}</div>
        <span class="text-xs text-gray-300 font-medium tracking-wide opacity-80 group-hover:opacity-100">{action.label}</span>
      </button>
    {/each}
  </div>
</div>
