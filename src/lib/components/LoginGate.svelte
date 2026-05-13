<script>
  import { onMount } from 'svelte';
  import {
    authStatus,
    authReady,
    refreshAuthStatus,
    login,
    logout,
    unlockTechnicianCritical,
    listSupportUsers,
    createSupportUser,
    deleteSupportUser,
  } from '$lib/stores/auth.js';
  import { loadSettings } from '$lib/stores/settings.js';

  let username = '';
  let password = '';
  let error = '';
  let loading = false;
  let showPassword = false;
  let shakeForm = false;
  let mounted = false;
  let isLightTheme = false;
  let visualStyle = 'cyber';

  let showAdmin = false;
  /** @type {Array<{username:string, profile:string, active?:boolean}>} */
  let users = [];
  let newUser = '';
  let newPass = '';
  let newProfile = 'tecnico';
  let adminError = '';
  let adminMsg = '';

  let criticalPassword = '';
  let criticalMsg = '';
  let criticalErr = '';

  /** @type {Array<{id:number,x:number,y:number,size:number,drift:number,delay:number,opacity:number}>} */
  let particles = [];

  function toggleTheme() {
    isLightTheme = !isLightTheme;
  }

  function setVisualStyle(style) {
    visualStyle = style;
  }

  async function doLogin() {
    error = '';
    loading = true;
    try {
      await login(username.trim(), password);
      await loadSettings();
      username = '';
      password = '';
    } catch (e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      error = err?.message || 'Credenciales inválidas';
      shakeForm = true;
      setTimeout(() => (shakeForm = false), 600);
    } finally {
      loading = false;
    }
  }

  async function doLogout() {
    await logout();
    showAdmin = false;
  }

  async function refreshUsers() {
    adminError = '';
    try {
      users = await listSupportUsers();
    } catch (e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      adminError = err?.message || 'Error cargando usuarios';
    }
  }

  async function addUser() {
    adminError = '';
    adminMsg = '';
    try {
      await createSupportUser(newUser.trim(), newPass, newProfile);
      adminMsg = 'Usuario creado';
      newUser = '';
      newPass = '';
      await refreshUsers();
    } catch (e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      adminError = err?.message || 'Error al crear usuario';
    }
  }

  async function removeUser(user) {
    adminError = '';
    adminMsg = '';
    try {
      await deleteSupportUser(user);
      adminMsg = 'Usuario eliminado';
      await refreshUsers();
    } catch (e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      adminError = err?.message || 'Error al eliminar usuario';
    }
  }

  async function unlockCritical() {
    criticalErr = '';
    criticalMsg = '';
    try {
      const ok = await unlockTechnicianCritical(criticalPassword, 20);
      criticalMsg = ok ? 'Privilegios críticos habilitados por 20 minutos' : '';
      if (!ok) criticalErr = 'Clave crítica inválida';
      criticalPassword = '';
    } catch (e) {
      const err = /** @type {{message?: string} | null | undefined} */ (e);
      criticalErr = err?.message || 'Error al desbloquear';
    }
  }

  onMount(async () => {
    particles = Array.from({ length: 36 }, (_, i) => ({
      id: i,
      x: Math.random() * 100,
      y: Math.random() * 100,
      size: Math.random() * 2 + 1,
      drift: Math.random() * 24 + 12,
      delay: Math.random() * 8,
      opacity: Math.random() * 0.5 + 0.15,
    }));

    setTimeout(() => (mounted = true), 70);
    await refreshAuthStatus();
    await loadSettings();
  });
</script>

{#if $authReady && !$authStatus.is_authenticated}
  <div class="login-root" class:mounted class:light-theme={isLightTheme} class:style-cyber={visualStyle === 'cyber'} class:style-corporate={visualStyle === 'corporate'} class:style-minimal={visualStyle === 'minimal'}>
    <div class="ambient-gradient" aria-hidden="true"></div>
    <div class="ambient-grid" aria-hidden="true"></div>
    <div class="ambient-sweep" aria-hidden="true"></div>

    <div class="particle-layer" aria-hidden="true">
      {#each particles as p}
        <span
          class="particle"
          style="left:{p.x}%; top:{p.y}%; width:{p.size}px; height:{p.size}px; opacity:{p.opacity}; animation-duration:{p.drift}s; animation-delay:-{p.delay}s"
        ></span>
      {/each}
    </div>

    <section class="hero-left">
      <img class="kernelia-logo" src="/KERNELIA_LOGO.svg" alt="KernelIA by Hackteck" />
    </section>

    <section class="login-card" class:shake={shakeForm} aria-label="Autenticación KernelIA">
      <header class="card-head">
        <div class="shield">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M12 3l7 4v6c0 5-3.5 7.8-7 9-3.5-1.2-7-4-7-9V7l7-4z"/></svg>
        </div>
        <div>
          <h1>Acceso al núcleo</h1>
          <p>Identifícate para iniciar sesión segura</p>
        </div>
        <button class="theme-toggle" type="button" on:click={toggleTheme} aria-label="Cambiar tema">
          {#if isLightTheme}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/></svg>
          {/if}
        </button>
      </header>

      <div class="chips">
        <span class="chip active">Superusuario</span>
        <span class="chip active">Soporte</span>
        <span class="chip">Técnico</span>
      </div>

      <div class="style-switcher" role="group" aria-label="Estilo visual del login">
        <button type="button" class:active={visualStyle === 'cyber'} on:click={() => setVisualStyle('cyber')}>Cyber</button>
        <button type="button" class:active={visualStyle === 'corporate'} on:click={() => setVisualStyle('corporate')}>Corporate</button>
        <button type="button" class:active={visualStyle === 'minimal'} on:click={() => setVisualStyle('minimal')}>Minimal</button>
      </div>

      <div class="field">
        <label for="lg-user">Identificador</label>
        <div class="input-wrap">
          <svg class="ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          <input id="lg-user" type="text" bind:value={username} placeholder="usuario de acceso" autocomplete="username" on:keydown={(e) => e.key === 'Enter' && doLogin()} disabled={loading} />
        </div>
      </div>

      <div class="field">
        <label for="lg-pass">Contraseña</label>
        <div class="input-wrap">
          <svg class="ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="3" y="11" width="18" height="10" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
          <input id="lg-pass" type={showPassword ? 'text' : 'password'} bind:value={password} placeholder="••••••••••••" autocomplete="current-password" on:keydown={(e) => e.key === 'Enter' && doLogin()} disabled={loading} />
          <button class="eye" type="button" on:click={() => (showPassword = !showPassword)} aria-label="Mostrar u ocultar contraseña">
            {#if showPassword}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"/><path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
            {/if}
          </button>
        </div>
      </div>

      {#if error}
        <div class="err">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          <span>{error}</span>
        </div>
      {/if}

      <button class="auth-btn" on:click={doLogin} disabled={loading || !username || !password}>
        {#if loading}
          <span class="spinner"></span>
          <span>Conectando...</span>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="13 17 18 12 13 7"/><line x1="18" y1="12" x2="3" y2="12"/></svg>
          <span>Iniciar enlace seguro</span>
        {/if}
      </button>

      <footer class="card-foot">
        <div class="status"><span></span>AES-256-GCM activo</div>
        <small>RBAC · SEC-L3</small>
      </footer>
    </section>
  </div>
{/if}

{#if $authReady && $authStatus.is_authenticated}
  <div class="session-pill">
    <span class="s-dot"></span>
    <strong>{$authStatus.profile}</strong>
    <span>{$authStatus.username}</span>
    <button on:click={doLogout}>Salir</button>
  </div>

  {#if $authStatus.profile === 'tecnico'}
    <div class="tech-panel">
      <h3>Elevación de privilegios críticos</h3>
      <p>Ingresa la clave crítica para habilitar acciones sensibles (20 min).</p>
      <input type="password" bind:value={criticalPassword} placeholder="Clave crítica" on:keydown={(e) => e.key === 'Enter' && unlockCritical()} />
      <button on:click={unlockCritical} disabled={!criticalPassword}>Desbloquear 20 min</button>
      {#if criticalMsg}<div class="ok">{criticalMsg}</div>{/if}
      {#if criticalErr}<div class="err-inline">{criticalErr}</div>{/if}
    </div>
  {/if}

  {#if $authStatus.profile === 'superusuario'}
    <div class="admin-launcher">
      <button on:click={async () => { showAdmin = !showAdmin; if (showAdmin) await refreshUsers(); }}>
        {showAdmin ? 'Cerrar' : 'Gestionar usuarios'}
      </button>
    </div>
    {#if showAdmin}
      <div class="admin-panel">
        <h3>Administración de usuarios</h3>
        <div class="create-row">
          <input bind:value={newUser} placeholder="Nuevo usuario" />
          <input type="password" bind:value={newPass} placeholder="Contraseña" />
          <select bind:value={newProfile}>
            <option value="soporte1">soporte1</option>
            <option value="tecnico">tecnico</option>
            <option value="superusuario">superusuario</option>
          </select>
          <button on:click={addUser} disabled={!newUser || !newPass}>Crear</button>
        </div>
        {#if adminMsg}<div class="ok">{adminMsg}</div>{/if}
        {#if adminError}<div class="err-inline">{adminError}</div>{/if}
        <div class="user-list">
          {#each users as u}
            <div class="user-item">
              <span>{u.username}</span>
              <span class="u-profile">{u.profile}</span>
              <button on:click={() => removeUser(u.username)} disabled={u.username === 'superadmin'}>Eliminar</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
{/if}

<style>
  .login-root {
    --bg-a: #030816;
    --bg-b: #040b1d;
    --bg-c: #020611;
    --grid: rgba(0, 255, 140, 0.06);
    --card-bg-a: rgba(8, 14, 28, 0.84);
    --card-bg-b: rgba(5, 10, 22, 0.74);
    --card-border: rgba(109, 250, 210, 0.24);
    --text-main: #f4fbff;
    --text-sub: rgba(205, 220, 239, 0.72);
    --accent-a: #00f59f;
    --accent-b: #00d9ff;
    --input-bg: rgba(9, 17, 31, 0.88);
    --input-border: rgba(140, 175, 208, 0.2);
    --hero-h2-size: 34px;
    --hero-p-size: 12px;
    --card-radius: 22px;
    --card-shadow: 0 28px 70px rgba(0, 0, 0, 0.6), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    --btn-letter: 0.08em;
    --particle-opacity: 1;
    position: fixed;
    inset: 0;
    display: grid;
    grid-template-columns: 1.1fr 0.9fr;
    align-items: center;
    overflow: hidden;
    background: var(--bg-a);
    font-family: 'Outfit', sans-serif;
    opacity: 0;
    transition: opacity 0.5s ease;
    z-index: 2000;
  }
  .login-root.mounted { opacity: 1; }

  .ambient-gradient {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(circle at 18% 40%, rgba(0, 255, 140, 0.16), transparent 42%),
      radial-gradient(circle at 82% 20%, rgba(0, 176, 255, 0.14), transparent 35%),
      linear-gradient(130deg, var(--bg-b) 0%, var(--bg-c) 45%, var(--bg-a) 100%);
  }
  .ambient-grid {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(var(--grid) 1px, transparent 1px),
      linear-gradient(90deg, color-mix(in srgb, var(--grid) 85%, transparent) 1px, transparent 1px);
    background-size: 56px 56px;
    mask-image: radial-gradient(circle at 35% 45%, black 20%, transparent 85%);
    opacity: 0.6;
  }
  .ambient-sweep {
    position: absolute;
    inset: -20% -10% auto -10%;
    height: 50%;
    background: linear-gradient(to bottom, rgba(0, 255, 140, 0.12), transparent 70%);
    filter: blur(40px);
    animation: sweep 8s ease-in-out infinite alternate;
  }
  @keyframes sweep { from { transform: translateY(-20px); } to { transform: translateY(20px); } }

  .particle-layer { position: absolute; inset: 0; pointer-events: none; }
  .particle {
    position: absolute;
    border-radius: 999px;
    background: radial-gradient(circle, rgba(0, 255, 160, 1) 0%, rgba(0, 180, 255, 0.5) 55%, transparent 100%);
    box-shadow: 0 0 14px rgba(0, 255, 160, 0.35);
    animation-name: float-dot;
    animation-timing-function: linear;
    animation-iteration-count: infinite;
    opacity: calc(var(--particle-opacity) * 1);
  }
  @keyframes float-dot {
    0% { transform: translate3d(0, 0, 0); }
    50% { transform: translate3d(16px, -26px, 0); }
    100% { transform: translate3d(0, -52px, 0); }
  }

  .hero-left {
    position: relative;
    z-index: 2;
    display: grid;
    place-items: center;
    gap: 28px;
    padding: 40px;
  }

  .kernelia-logo {
    width: min(720px, 82%);
    height: auto;
    display: block;
    animation: logo-reveal 0.9s cubic-bezier(0.16, 1, 0.3, 1) both;
  }
  @keyframes logo-reveal {
    from { opacity: 0; transform: translateY(18px) scale(0.97); filter: blur(10px); }
    to { opacity: 1; transform: translateY(0) scale(1); filter: blur(0); }
  }

  .login-card {
    position: relative;
    z-index: 2;
    width: min(450px, 90%);
    padding: 30px;
    border-radius: var(--card-radius);
    background: linear-gradient(165deg, var(--card-bg-a), var(--card-bg-b));
    border: 1px solid var(--card-border);
    box-shadow: var(--card-shadow);
    backdrop-filter: blur(14px);
  }
  .login-card.shake { animation: shake 0.55s cubic-bezier(.36,.07,.19,.97) both; }
  @keyframes shake {
    10%,90% { transform: translateX(-4px); }
    20%,80% { transform: translateX(6px); }
    30%,50%,70% { transform: translateX(-8px); }
    40%,60% { transform: translateX(8px); }
  }

  .card-head { display: flex; gap: 12px; align-items: center; margin-bottom: 18px; }
  .shield {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: #00ffb3;
    background: rgba(0, 255, 179, 0.1);
    border: 1px solid rgba(0, 255, 179, 0.32);
  }
  .shield svg { width: 18px; height: 18px; }

  h1 { margin: 0; color: var(--text-main); font-size: 27px; letter-spacing: -0.02em; text-transform: uppercase; }
  .card-head p { margin: 4px 0 0; color: var(--text-sub); font-size: 13px; }
  .theme-toggle {
    margin-left: auto;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    border: 1px solid rgba(146, 200, 228, 0.25);
    background: rgba(255, 255, 255, 0.04);
    color: rgba(197, 232, 255, 0.85);
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .theme-toggle:hover { transform: translateY(-1px); border-color: rgba(0, 255, 184, 0.42); }
  .theme-toggle svg { width: 16px; height: 16px; }

  .chips { display: flex; gap: 8px; margin: 4px 0 18px; }
  .chip {
    border-radius: 999px;
    padding: 4px 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    color: rgba(227, 242, 255, 0.55);
    font-size: 10px;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }
  .chip.active {
    color: #07f7ad;
    border-color: rgba(0, 255, 173, 0.35);
    background: rgba(0, 255, 173, 0.08);
  }
  .style-switcher {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    margin: -4px 0 14px;
  }
  .style-switcher button {
    border: 1px solid rgba(153, 191, 221, 0.25);
    background: rgba(255, 255, 255, 0.03);
    color: rgba(199, 220, 241, 0.82);
    border-radius: 9px;
    height: 30px;
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .style-switcher button:hover { border-color: rgba(0, 255, 184, 0.45); }
  .style-switcher button.active {
    color: #041722;
    background: linear-gradient(135deg, rgba(0, 246, 173, 0.92), rgba(0, 208, 255, 0.92));
    border-color: transparent;
    font-weight: 700;
  }

  .field { margin-bottom: 12px; }
  .field label {
    display: block;
    margin: 0 0 7px;
    color: rgba(193, 213, 236, 0.75);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  .input-wrap { position: relative; }
  .ico {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 16px;
    height: 16px;
    color: rgba(189, 208, 226, 0.45);
  }
  .input-wrap input {
    width: 100%;
    box-sizing: border-box;
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    color: #f0f8ff;
    border-radius: 12px;
    padding: 12px 44px 12px 40px;
    font-size: 14px;
    outline: none;
    transition: all 0.2s ease;
  }
  .input-wrap input:focus {
    border-color: rgba(0, 255, 184, 0.48);
    box-shadow: 0 0 0 3px rgba(0, 255, 184, 0.12);
  }
  .input-wrap::after {
    content: '';
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: 3px;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(0, 255, 184, 0.8), transparent);
    transform: scaleX(0);
    transition: transform 0.22s ease;
  }
  .input-wrap:focus-within::after { transform: scaleX(1); }
  .input-wrap input::placeholder { color: rgba(180, 203, 230, 0.36); }

  .eye {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: rgba(188, 210, 233, 0.55);
    cursor: pointer;
  }
  .eye svg { width: 16px; height: 16px; }

  .err {
    margin: 10px 0 6px;
    display: flex;
    gap: 8px;
    align-items: center;
    border-radius: 10px;
    border: 1px solid rgba(255, 78, 122, 0.34);
    background: rgba(255, 64, 112, 0.1);
    color: #ff9aba;
    padding: 9px 11px;
    font-size: 12px;
  }
  .err svg { width: 14px; height: 14px; flex-shrink: 0; }

  .auth-btn {
    width: 100%;
    margin-top: 12px;
    border: none;
    border-radius: 12px;
    padding: 13px 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: var(--btn-letter);
    font-weight: 700;
    color: #06131b;
    background: linear-gradient(135deg, #00f59f, #00d9ff);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.2s ease, opacity 0.2s ease;
    position: relative;
    overflow: hidden;
  }
  .auth-btn:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 16px 30px rgba(0, 243, 170, 0.26);
  }
  .auth-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .auth-btn svg { width: 16px; height: 16px; }
  .auth-btn::after {
    content: '';
    position: absolute;
    top: -120%;
    left: -40%;
    width: 40%;
    height: 320%;
    transform: rotate(22deg);
    background: linear-gradient(180deg, transparent, rgba(255, 255, 255, 0.45), transparent);
    transition: left 0.45s ease;
  }
  .auth-btn:hover::after { left: 120%; }

  .spinner {
    width: 14px;
    height: 14px;
    border-radius: 999px;
    border: 2px solid rgba(10, 20, 28, 0.3);
    border-top-color: rgba(8, 22, 30, 0.9);
    animation: spin 0.75s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .card-foot {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid rgba(173, 201, 227, 0.16);
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: rgba(178, 201, 225, 0.72);
    font-size: 11px;
  }

  .login-card,
  .kernelia-logo,
  .chip,
  .field,
  .auth-btn {
    animation: rise-in 0.6s ease both;
  }
  .kernelia-logo { animation-delay: 0.05s; }
  .chip:nth-child(1) { animation-delay: 0.08s; }
  .chip:nth-child(2) { animation-delay: 0.12s; }
  .chip:nth-child(3) { animation-delay: 0.16s; }
  .field:nth-of-type(1) { animation-delay: 0.18s; }
  .field:nth-of-type(2) { animation-delay: 0.24s; }
  .auth-btn { animation-delay: 0.3s; }
  @keyframes rise-in {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .login-root.light-theme {
    --bg-a: #eaf3ff;
    --bg-b: #d9ecff;
    --bg-c: #f4f9ff;
    --grid: rgba(0, 124, 188, 0.18);
    --card-bg-a: rgba(255, 255, 255, 0.86);
    --card-bg-b: rgba(239, 248, 255, 0.82);
    --card-border: rgba(0, 149, 199, 0.3);
    --text-main: #0d2238;
    --text-sub: rgba(19, 44, 70, 0.68);
    --accent-a: #18c5ff;
    --accent-b: #1de8a8;
    --input-bg: rgba(255, 255, 255, 0.94);
    --input-border: rgba(39, 96, 145, 0.22);
  }
  .login-root.light-theme .field label { color: rgba(18, 53, 87, 0.74); }
  .login-root.light-theme .input-wrap input { color: #11304f; }
  .login-root.light-theme .input-wrap input::placeholder { color: rgba(19, 50, 80, 0.4); }
  .login-root.light-theme .chip { color: rgba(16, 58, 87, 0.62); border-color: rgba(17, 85, 129, 0.22); }
  .login-root.light-theme .chip.active { color: #0e6b87; background: rgba(0, 175, 148, 0.12); border-color: rgba(0, 150, 126, 0.35); }
  .login-root.light-theme .status,
  .login-root.light-theme .card-foot small { color: rgba(14, 51, 80, 0.72); }
  .login-root.light-theme .auth-btn { color: #062032; background: linear-gradient(135deg, var(--accent-a), var(--accent-b)); }

  .login-root.style-cyber {
    --hero-h2-size: 34px;
    --hero-p-size: 12px;
    --card-radius: 22px;
    --card-shadow: 0 28px 70px rgba(0, 0, 0, 0.6), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    --btn-letter: 0.08em;
    --particle-opacity: 1;
  }

  .login-root.style-corporate {
    --hero-h2-size: 30px;
    --hero-p-size: 11px;
    --card-radius: 14px;
    --card-shadow: 0 14px 32px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    --btn-letter: 0.04em;
    --particle-opacity: 0.35;
  }
  .login-root.style-corporate .ambient-grid { opacity: 0.28; }

  .login-root.style-minimal {
    --hero-h2-size: 28px;
    --hero-p-size: 10px;
    --card-radius: 10px;
    --card-shadow: 0 8px 24px rgba(0, 0, 0, 0.28);
    --btn-letter: 0.03em;
    --particle-opacity: 0;
  }
  .login-root.style-minimal .ambient-grid { opacity: 0.08; }
  .login-root.style-minimal .ambient-sweep { display: none; }
  .login-root.style-minimal .chips { margin-bottom: 10px; }
  .login-root.style-minimal .chip { font-size: 9px; }
  .status { display: flex; align-items: center; gap: 6px; }
  .status span {
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: #00ffbc;
    box-shadow: 0 0 10px rgba(0, 255, 188, 0.8);
    animation: blink 1.8s ease infinite;
  }
  @keyframes blink { 50% { opacity: 0.3; } }

  .session-pill {
    position: fixed; top: 12px; right: 14px; z-index: 1200;
    display: flex; gap: .5rem; align-items: center;
    background: rgba(9, 14, 20, .88); border: 1px solid rgba(255, 255, 255, .15);
    border-radius: 999px; padding: .4rem .6rem; color: #e7f4ff;
    backdrop-filter: blur(12px); font-size: 13px;
  }
  .s-dot { width: 8px; height: 8px; border-radius: 50%; background: #00ff85; box-shadow: 0 0 10px #00ff85; }
  .session-pill button {
    border: none; border-radius: 8px;
    background: linear-gradient(135deg, #00e3ff, #00ff85);
    color: #041015; font-weight: 700; padding: .3rem .7rem; cursor: pointer; font-size: 12px;
  }
  .tech-panel, .admin-panel {
    position: fixed; bottom: 16px; right: 16px; width: min(460px, 92vw); z-index: 1300;
    padding: 1rem; border-radius: 14px;
    background: rgba(8, 13, 20, .94); border: 1px solid rgba(255, 255, 255, .16);
    color: #e7edff; display: grid; gap: .6rem;
  }
  .tech-panel h3, .admin-panel h3 { margin: 0; font-size: 14px; }
  .tech-panel p { margin: 0; font-size: 12px; color: #aeb8d0; }
  .tech-panel input, .admin-panel input, .admin-panel select {
    width: 100%; border-radius: 8px; border: 1px solid rgba(255, 255, 255, .18);
    background: rgba(255, 255, 255, .06); color: #f5f8ff; padding: .6rem .8rem; outline: none; font-size: 13px;
  }
  .tech-panel button, .admin-panel button {
    border: none; border-radius: 8px;
    background: linear-gradient(135deg, #00e3ff, #00ff85);
    color: #041015; font-weight: 700; padding: .6rem .9rem; cursor: pointer; font-size: 13px;
  }
  .tech-panel button:disabled, .admin-panel button:disabled { opacity: .5; cursor: not-allowed; }
  .admin-launcher { position: fixed; bottom: 16px; left: 16px; z-index: 1300; }
  .admin-launcher button {
    border: none; border-radius: 10px;
    background: linear-gradient(135deg, #00e3ff, #00ff85);
    color: #041015; font-weight: 700; padding: .65rem 1rem; cursor: pointer; font-size: 13px;
  }
  .create-row { display: grid; grid-template-columns: 1fr 1fr auto auto; gap: .4rem; }
  .user-list { max-height: 220px; overflow: auto; display: grid; gap: .3rem; }
  .user-item {
    display: grid; grid-template-columns: 1fr auto auto; gap: .4rem; align-items: center;
    border: 1px solid rgba(255, 255, 255, .1); border-radius: 8px; padding: .4rem; font-size: 13px;
  }
  .u-profile { font-size: .72rem; color: #a9badf; }
  .ok { font-size: .8rem; color: #9affc7; }
  .err-inline { font-size: .8rem; color: #ff8d8d; }

  @media (max-width: 980px) {
    .login-root { grid-template-columns: 1fr; }
    .hero-left { display: none; }
    .login-card { margin: 0 auto; }
    .ambient-grid { mask-image: none; }
  }

  @media (max-width: 560px) {
    .login-card { width: calc(100% - 24px); padding: 22px; border-radius: 16px; }
    h1 { font-size: 22px; }
    .card-foot { flex-direction: column; align-items: flex-start; gap: 6px; }
    .create-row { grid-template-columns: 1fr; }
  }
</style>
