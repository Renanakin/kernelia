<script>
  import { onMount } from 'svelte';
  import { login } from '$lib/stores/auth.js';

  let username = '';
  let password = '';
  let loading = false;
  let error = '';
  let mounted = false;
  let showPassword = false;
  let shakeForm = false;
  let phase = 'idle'; // idle | connecting | authenticated | denied
  let connectionProgress = 0;
  let nodeCount = 0;
  let dots = [];
  let rings = [
    { r: 60, speed: 12, offset: 0 },
    { r: 100, speed: -18, offset: 45 },
    { r: 145, speed: 9, offset: 120 },
  ];
  /** @type {number | null} */
  let raf = null;
  let time = 0;

  function genDots() {
    dots = Array.from({ length: 28 }, (_, i) => ({
      id: i,
      angle: (i / 28) * 360,
      ring: Math.floor(i / 9),
      r: [60, 100, 145][Math.floor(i / 9)] ?? 60,
      pulse: Math.random(),
      active: Math.random() > 0.4,
    }));
  }

  /** @param {number} ts */
  function animate(ts) {
    time = ts / 1000;
    raf = requestAnimationFrame(animate);
  }

  onMount(() => {
    genDots();
    raf = requestAnimationFrame(animate);
    setTimeout(() => { mounted = true; }, 80);
    return () => {
      if (raf !== null) cancelAnimationFrame(raf);
    };
  });

  async function handleLogin() {
    if (!username || !password) { error = 'Credenciales requeridas'; triggerShake(); return; }
    if (loading) return;
    loading = true;
    error = '';
    phase = 'connecting';
    connectionProgress = 0;

    // Animate progress
    const interval = setInterval(() => {
      connectionProgress = Math.min(connectionProgress + Math.random() * 18, 92);
    }, 120);

    try {
      const status = await login(username, password);
      clearInterval(interval);
      if (status.is_authenticated) {
        connectionProgress = 100;
        phase = 'authenticated';
      } else {
        connectionProgress = 0;
        phase = 'denied';
        error = 'Acceso denegado — credenciales inválidas';
        triggerShake();
        setTimeout(() => { phase = 'idle'; }, 1400);
      }
    } catch(e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      clearInterval(interval);
      connectionProgress = 0;
      phase = 'denied';
      error = err?.message || 'Error de conexión con el núcleo';
      triggerShake();
      setTimeout(() => { phase = 'idle'; }, 1400);
    } finally {
      loading = false;
    }
  }

  function triggerShake() { shakeForm = true; setTimeout(() => shakeForm = false, 600); }
  /** @param {KeyboardEvent} e */
  function handleKeydown(e) { if (e.key === 'Enter') handleLogin(); }

  $: coreColor = phase === 'authenticated' ? '#00FF80' : phase === 'denied' ? '#FF3060' : phase === 'connecting' ? '#00B8FF' : '#00FF80';
  $: statusLabel = phase === 'connecting' ? 'ESTABLECIENDO ENLACE...' : phase === 'authenticated' ? 'ENLACE ESTABLECIDO' : phase === 'denied' ? 'ACCESO DENEGADO' : 'NÚCLEO ACTIVO';
</script>

<svelte:head>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=JetBrains+Mono:wght@300;400;500&display=swap');
  </style>
</svelte:head>

<div class="root" class:mounted>

  <!-- LEFT PANEL — KERNELIA CORE -->
  <div class="panel-left">
    <div class="noise" aria-hidden="true"></div>
    <div class="grid-bg" aria-hidden="true"></div>

    <!-- Ambient glows -->
    <div class="glow g1" style="background: radial-gradient(circle, {coreColor}22 0%, transparent 70%); transition: background 0.8s;"></div>
    <div class="glow g2"></div>

    <!-- KERNELIA brand logo -->
    <div class="core-wrap">
      <img class="kernelia-logo" src="/KERNELIA_LOGO.svg" alt="KernelIA by Hackteck" />

      <!-- Status label below core -->
      <div class="core-status" style="color: {coreColor}; transition: color 0.8s;">
        <span class="core-status-dot" style="background: {coreColor}; box-shadow: 0 0 8px {coreColor}; transition: background 0.8s, box-shadow 0.8s;"></span>
        <span class="core-status-text">{statusLabel}</span>
      </div>

      <!-- Connection progress bar -->
      {#if phase === 'connecting' || phase === 'authenticated'}
        <div class="progress-wrap">
          <div class="progress-bar" style="width: {connectionProgress}%; background: {coreColor}; transition: width 0.15s linear, background 0.8s;"></div>
        </div>
      {/if}
    </div>

    <!-- Bottom left identity -->
    <div class="left-brand">
      <span class="left-brand-name">KERNELIA</span>
      <span class="left-brand-sub">HACKTECK SYSTEMS · v2.1.0</span>
    </div>
  </div>

  <!-- RIGHT PANEL — AUTH FORM -->
  <div class="panel-right">
    <div class="form-wrap" class:shake={shakeForm}>

      <!-- Header -->
      <div class="form-header">
        <div class="form-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        </div>
        <div>
          <h1 class="form-title">Acceso al Núcleo</h1>
          <p class="form-subtitle">Identifíquese para conectar con KERNELIA</p>
        </div>
      </div>

      <!-- Role pills -->
      <div class="role-row">
        <span class="role-pill active">OWNER</span>
        <span class="role-pill">TÉCNICO</span>
        <span class="role-pill dim">VIEWER</span>
      </div>

      <!-- Fields -->
      <div class="fields">
        <div class="field">
          <label class="field-label" for="u">Identificador</label>
          <div class="field-inner">
            <svg class="field-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>
            </svg>
            <input id="u" type="text" class="field-input" bind:value={username}
              placeholder="usuario de acceso" autocomplete="username"
              on:keydown={handleKeydown} disabled={loading}/>
          </div>
        </div>

        <div class="field">
          <label class="field-label" for="p">Contraseña</label>
          <div class="field-inner">
            <svg class="field-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
            </svg>
            <input id="p" type={showPassword ? 'text' : 'password'} class="field-input" bind:value={password}
              placeholder="••••••••••••" autocomplete="current-password"
              on:keydown={handleKeydown} disabled={loading}/>
            <button class="vis-toggle" type="button" on:click={() => showPassword = !showPassword} tabindex="-1">
              {#if showPassword}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"/><path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
              {:else}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
              {/if}
            </button>
          </div>
        </div>

        {#if error}
          <div class="err-box">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
            <span>{error}</span>
          </div>
        {/if}

        <button class="auth-btn" class:loading on:click={handleLogin} disabled={loading}>
          {#if loading}
            <span class="spinner"></span>
            <span>Conectando con KERNELIA...</span>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
            <span>Iniciar Enlace Seguro</span>
          {/if}
        </button>
      </div>

      <!-- Footer -->
      <div class="form-footer">
        <div class="status-row">
          <span class="s-dot"></span>
          <span>Sistema operativo · AES-256-GCM activo</span>
        </div>
        <span class="version">RBAC · SEC-L3</span>
      </div>
    </div>

    <!-- Watermark -->
    <div class="watermark">
      <img src="/LOGO_HACKTECK.png" alt="Hackteck" class="wm-logo"/>
      <span class="wm-txt">HACKTECK © 2026</span>
    </div>
  </div>

</div>

<style>
  .root {
    position: fixed; inset: 0;
    display: flex;
    font-family: 'Outfit', sans-serif;
    background: #020813;
    opacity: 0;
    transition: opacity 0.9s ease;
    overflow: hidden;
  }
  .root.mounted { opacity: 1; }

  /* ── LEFT PANEL ── */
  .panel-left {
    position: relative;
    width: 52%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-right: 1px solid rgba(0,255,128,0.07);
  }
  .noise {
    position: absolute; inset: 0;
    background-image: url('data:image/svg+xml;utf8,%3Csvg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="n"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.75" numOctaves="4" stitchTiles="stitch"/%3E%3C/filter%3E%3Crect width="100%25" height="100%25" filter="url(%23n)"/%3E%3C/svg%3E');
    opacity: 0.025; mix-blend-mode: overlay; pointer-events: none; z-index: 1;
  }
  .grid-bg {
    position: absolute; inset: 0;
    background-image:
      linear-gradient(rgba(0,255,128,0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0,255,128,0.03) 1px, transparent 1px);
    background-size: 52px 52px;
    mask-image: radial-gradient(ellipse 70% 70% at 50% 50%, black 20%, transparent 100%);
  }
  .glow {
    position: absolute; border-radius: 50%; pointer-events: none; filter: blur(90px);
  }
  .g1 { width: 520px; height: 520px; top: 50%; left: 50%; transform: translate(-50%,-50%); }
  .g2 { width: 300px; height: 300px; bottom: -100px; left: -80px;
    background: radial-gradient(circle, rgba(0,184,255,0.08) 0%, transparent 70%); }

  /* ── ORBITAL CORE ── */
  .core-wrap {
    position: relative; z-index: 5;
    display: flex; flex-direction: column; align-items: center; gap: 24px;
    animation: core-in 1s cubic-bezier(0.16,1,0.3,1) both; animation-delay: 0.3s;
  }
  @keyframes core-in {
    from { opacity:0; transform: scale(0.9); }
    to   { opacity:1; transform: scale(1); }
  }
  .kernelia-logo {
    width: min(680px, 84vw);
    max-width: 84%;
    height: auto;
    display: block;
  }

  .core-status {
    display: flex; align-items: center; gap: 8px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px; letter-spacing: 0.15em; font-weight: 500;
  }
  .core-status-dot {
    width: 7px; height: 7px; border-radius: 50%;
    animation: blink 2s ease-in-out infinite;
  }
  @keyframes blink { 0%,100% { opacity:1; } 50% { opacity:0.25; } }

  .progress-wrap {
    width: 260px; height: 2px;
    background: rgba(255,255,255,0.06); border-radius: 2px; overflow: hidden;
  }
  .progress-bar { height: 100%; border-radius: 2px; }

  /* ── LEFT BRAND ── */
  .left-brand {
    position: absolute; bottom: 28px; left: 32px;
    display: flex; flex-direction: column; gap: 3px; z-index: 5;
  }
  .left-brand-name {
    font-family: 'JetBrains Mono', monospace;
    font-size: 19px; font-weight: 500; letter-spacing: 0.25em;
    color: #00FF80;
    text-shadow: 0 0 20px rgba(0,255,128,0.4);
  }
  .left-brand-sub {
    font-size: 10px; letter-spacing: 0.14em;
    color: rgba(255,255,255,0.25);
    font-family: 'JetBrains Mono', monospace;
  }

  /* ── RIGHT PANEL ── */
  .panel-right {
    width: 48%;
    display: flex; align-items: center; justify-content: center;
    position: relative;
    background: rgba(2,8,19,0.6);
  }

  .form-wrap {
    width: 100%; max-width: 400px;
    padding: 0 44px;
    animation: form-in 1s cubic-bezier(0.16,1,0.3,1) both; animation-delay: 0.5s;
  }
  @keyframes form-in {
    from { opacity:0; transform: translateX(24px); }
    to   { opacity:1; transform: translateX(0); }
  }
  .form-wrap.shake { animation: shake 0.55s cubic-bezier(.36,.07,.19,.97) both; }
  @keyframes shake {
    10%,90%  { transform: translateX(-4px); }
    20%,80%  { transform: translateX(6px); }
    30%,50%,70% { transform: translateX(-8px); }
    40%,60%  { transform: translateX(8px); }
  }

  /* Header */
  .form-header {
    display: flex; align-items: center; gap: 16px; margin-bottom: 28px;
  }
  .form-badge {
    width: 44px; height: 44px; border-radius: 12px; flex-shrink: 0;
    background: rgba(0,255,128,0.1); border: 1px solid rgba(0,255,128,0.25);
    display: flex; align-items: center; justify-content: center;
    color: #00FF80;
  }
  .form-title { font-size: 22px; font-weight: 600; color: #fff; margin: 0 0 4px; letter-spacing: -0.02em; }
  .form-subtitle { font-size: 12px; color: rgba(255,255,255,0.35); margin: 0; }

  /* Role pills */
  .role-row { display: flex; gap: 6px; margin-bottom: 28px; }
  .role-pill {
    padding: 4px 12px; border-radius: 999px; font-size: 10px;
    font-weight: 600; letter-spacing: 0.1em;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.08);
    color: rgba(255,255,255,0.25);
  }
  .role-pill.active {
    background: rgba(0,255,128,0.1); border-color: rgba(0,255,128,0.3); color: #00FF80;
  }
  .role-pill.dim { opacity: 0.5; }

  /* Fields */
  .fields { display: flex; flex-direction: column; gap: 16px; }
  .field { display: flex; flex-direction: column; gap: 7px; }
  .field-label {
    font-size: 11px; font-weight: 500; letter-spacing: 0.12em;
    text-transform: uppercase; color: rgba(255,255,255,0.4);
  }
  .field-inner { position: relative; display: flex; align-items: center; }
  .field-icon {
    position: absolute; left: 14px; width: 15px; height: 15px;
    color: rgba(255,255,255,0.25); pointer-events: none; z-index: 1;
    transition: color 0.3s;
  }
  .field-inner:focus-within .field-icon { color: #00FF80; }
  .field-input {
    width: 100%; padding: 12px 42px 12px 42px;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 12px; color: #fff;
    font-family: 'Outfit', sans-serif; font-size: 14px;
    outline: none; box-sizing: border-box;
    transition: border-color 0.3s, box-shadow 0.3s, background 0.3s;
  }
  .field-input::placeholder { color: rgba(255,255,255,0.18); }
  .field-input:focus {
    border-color: rgba(0,255,128,0.45);
    background: rgba(0,255,128,0.04);
    box-shadow: 0 0 0 3px rgba(0,255,128,0.08);
  }
  .vis-toggle {
    position: absolute; right: 12px; background: none; border: none;
    cursor: pointer; padding: 2px; color: rgba(255,255,255,0.25); z-index: 2;
    display: flex; transition: color 0.2s;
  }
  .vis-toggle svg { width: 15px; height: 15px; }
  .vis-toggle:hover { color: rgba(255,255,255,0.6); }

  /* Error */
  .err-box {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 14px;
    background: rgba(255,48,96,0.08); border: 1px solid rgba(255,48,96,0.25);
    border-radius: 10px; color: #ff6b8a; font-size: 12px;
    animation: err-in 0.3s ease;
  }
  .err-box svg { width: 14px; height: 14px; flex-shrink: 0; }
  @keyframes err-in { from { opacity:0; transform: translateY(-4px); } to { opacity:1; transform: translateY(0); } }

  /* Auth button */
  .auth-btn {
    position: relative; display: flex; align-items: center; justify-content: center;
    gap: 10px; padding: 13px 24px; margin-top: 4px;
    background: linear-gradient(135deg, #00c96b, #00FF80 50%, #00d4ff);
    border: none; border-radius: 12px; color: #020202;
    font-family: 'Outfit', sans-serif; font-size: 14px; font-weight: 700;
    letter-spacing: 0.04em; cursor: pointer; overflow: hidden;
    transition: transform 0.2s, box-shadow 0.3s, opacity 0.3s; width: 100%;
  }
  .auth-btn svg { width: 17px; height: 17px; }
  .auth-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 10px 40px rgba(0,255,128,0.45), 0 0 0 1px rgba(0,255,128,0.3);
  }
  .auth-btn:active:not(:disabled) { transform: translateY(0); }
  .auth-btn:disabled { opacity: 0.65; cursor: not-allowed; }

  .spinner {
    width: 16px; height: 16px; border-radius: 50%;
    border: 2px solid rgba(2,8,19,0.3); border-top-color: #020202;
    animation: spin 0.7s linear infinite; flex-shrink: 0;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Footer */
  .form-footer {
    display: flex; align-items: center; justify-content: space-between;
    margin-top: 24px; padding-top: 18px;
    border-top: 1px solid rgba(255,255,255,0.05);
  }
  .status-row { display: flex; align-items: center; gap: 7px; font-size: 11px; color: rgba(255,255,255,0.28); }
  .s-dot {
    width: 6px; height: 6px; border-radius: 50%;
    background: #00FF80; box-shadow: 0 0 6px #00FF80;
    animation: blink 2s infinite;
  }
  .version { font-size: 10px; font-family: 'JetBrains Mono', monospace; color: rgba(255,255,255,0.18); letter-spacing: 0.06em; }

  /* Watermark */
  .watermark {
    position: absolute; bottom: 20px; right: 24px;
    display: flex; align-items: center; gap: 8px; opacity: 0.35; z-index: 5;
  }
  .wm-logo { height: 16px; filter: brightness(200%); }
  .wm-txt { font-size: 10px; letter-spacing: 0.14em; color: rgba(255,255,255,0.5); font-family: 'JetBrains Mono', monospace; }
</style>
