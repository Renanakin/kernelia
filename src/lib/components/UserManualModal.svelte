<script>
  export let show = false;
  export let onClose = () => {};

  const quickPrompts = [
    'Ejecuta un diagnóstico de red completo y resume hallazgos.',
    'Muéstrame procesos que consumen más CPU y RAM.',
    'Analiza controladores con error y dame plan de actualización.',
    'Genera un reporte técnico de soporte para este equipo.',
    'Revisa estado de seguridad local y puertos abiertos.',
    'Crea una rutina de mantenimiento semanal automática.'
  ];
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-[1000] bg-black/80 backdrop-blur-md flex items-center justify-center p-4" on:click|self={onClose}>
    <div class="w-full max-w-5xl max-h-[88vh] overflow-hidden rounded-2xl border border-[var(--glass-border)] bg-[var(--color-bg-card)] shadow-2xl flex flex-col">
      <div class="px-6 py-4 border-b border-white/10 flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-white">Manual de Usuario - KernelIA</h2>
          <p class="text-xs text-[var(--color-text-dim)]">Guía rápida para operar sin estrés</p>
        </div>
        <button on:click={onClose} class="px-3 py-1.5 text-xs rounded-lg border border-white/10 text-[var(--color-text-dim)] hover:text-white hover:bg-white/5 transition-colors">Cerrar</button>
      </div>

      <div class="overflow-y-auto p-6 space-y-6 text-sm text-[var(--color-text-main)]">
        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">1) Qué hace KernelIA</h3>
          <ul class="list-disc pl-5 space-y-1 text-[var(--color-text-dim)]">
            <li>Diagnóstico técnico guiado con IA (red, sistema, procesos, drivers, seguridad).</li>
            <li>Ejecución de herramientas del sistema con control de permisos (RBAC + MegaBoss).</li>
            <li>Telemetría del equipo, auditoría de acciones y generación de reportes.</li>
            <li>Mantenimiento y automatización operativa con flujos de auto-healing.</li>
            <li>Soporte en modelos cloud y modelos locales.</li>
          </ul>
        </section>

        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">2) Flujo recomendado de uso</h3>
          <ol class="list-decimal pl-5 space-y-1 text-[var(--color-text-dim)]">
            <li>Selecciona modelo IA en el selector superior.</li>
            <li>Si es cloud, configura API key desde Configuración.</li>
            <li>Empieza con una consulta concreta o usa una acción rápida.</li>
            <li>Revisa resultados y evidencia en Auditoría.</li>
            <li>Para acciones sensibles, desbloquea MegaBoss temporalmente.</li>
          </ol>
        </section>

        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">3) Funciones por módulo</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-[var(--color-text-dim)]">
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Telemetría</strong>
              <p>CPU, RAM, discos, red, uptime, procesos activos, estado del equipo.</p>
            </div>
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Red</strong>
              <p>Diagnóstico, DNS, gateway, adaptadores, latencia, reset de pila de red.</p>
            </div>
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Mantenimiento</strong>
              <p>Limpieza, análisis del sistema, tareas programadas, rutinas automáticas.</p>
            </div>
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Drivers</strong>
              <p>Detección de fallas, problemas de dispositivos, apertura de actualización.</p>
            </div>
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Seguridad</strong>
              <p>Firewall, Defender, puertos/conexiones, validaciones de seguridad local.</p>
            </div>
            <div class="rounded-xl border border-white/10 p-3 bg-black/20">
              <strong class="text-white">Auditoría</strong>
              <p>Trazabilidad de cada herramienta ejecutada con estado y resultado.</p>
            </div>
          </div>
        </section>

        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">4) Prompts listos para usar</h3>
          <div class="space-y-2">
            {#each quickPrompts as p}
              <div class="rounded-lg border border-white/10 bg-black/30 px-3 py-2 font-mono text-xs text-[#9fd8ff]">{p}</div>
            {/each}
          </div>
        </section>

        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">5) Permisos y seguridad</h3>
          <ul class="list-disc pl-5 space-y-1 text-[var(--color-text-dim)]">
            <li><strong class="text-white">Viewer:</strong> lectura y diagnóstico básico.</li>
            <li><strong class="text-white">PowerUser:</strong> mantenimiento y acciones operativas extendidas.</li>
            <li><strong class="text-white">Owner:</strong> acciones sensibles del sistema.</li>
            <li><strong class="text-white">MegaBoss:</strong> desbloqueo temporal para máximo privilegio.</li>
          </ul>
        </section>

        <section>
          <h3 class="text-sm font-semibold mb-2 text-[var(--color-brand-secondary)]">6) Buenas prácticas anti-estrés</h3>
          <ul class="list-disc pl-5 space-y-1 text-[var(--color-text-dim)]">
            <li>Haz una sola solicitud clara por mensaje.</li>
            <li>Pide siempre “resumen + próximos pasos” para acciones complejas.</li>
            <li>Antes de aplicar cambios críticos, solicita “modo simulación / validación previa”.</li>
            <li>Usa Auditoría para verificar exactamente qué se ejecutó.</li>
          </ul>
        </section>
      </div>
    </div>
  </div>
{/if}

