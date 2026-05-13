<script>
</script>

<div class="absolute inset-0 pointer-events-none flex items-center justify-center overflow-hidden z-0">
  <div class="relative w-[800px] h-[800px] flex items-center justify-center opacity-40">
    <!-- Pulsing aura -->
    <div class="absolute inset-0 bg-[var(--color-brand-primary)] rounded-full blur-[120px] opacity-20 animate-[pulse_8s_ease-in-out_infinite]"></div>
    
    <!-- Rotating outer ring -->
    <svg class="absolute w-full h-full animate-[spin_30s_linear_infinite]" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="50" cy="50" r="48" stroke="var(--color-brand-secondary)" stroke-width="0.3" stroke-dasharray="4 12 2 12" opacity="0.3"/>
      <circle cx="50" cy="50" r="42" stroke="var(--color-brand-primary)" stroke-width="0.1" stroke-dasharray="10 30" opacity="0.5"/>
    </svg>
    
    <!-- Counter-rotating inner ring -->
    <svg class="absolute w-[500px] h-[500px] animate-[spin_20s_linear_infinite_reverse]" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="50" cy="50" r="45" stroke="var(--color-brand-secondary)" stroke-width="0.5" stroke-dasharray="1 10" opacity="0.4"/>
      <circle cx="50" cy="50" r="30" stroke="var(--color-brand-primary)" stroke-width="1" stroke-dasharray="80 40" opacity="0.2"/>
    </svg>
    
    <!-- Central Node -->
    <div class="relative w-40 h-40 bg-gradient-to-br from-[var(--color-brand-primary)] to-[var(--color-brand-secondary)] rounded-full shadow-[0_0_80px_var(--color-brand-primary)] flex items-center justify-center z-10 blur-[4px]">
      <div class="w-20 h-20 bg-white rounded-full opacity-70 blur-[10px]"></div>
      <div class="w-10 h-10 bg-white rounded-full absolute blur-[4px]"></div>
    </div>
  </div>
</div>
