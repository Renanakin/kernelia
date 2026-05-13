import { writable } from 'svelte/store';
import {
  createSupportUser as apiCreateSupportUser,
  deleteSupportUser as apiDeleteSupportUser,
  getAuthStatus,
  listSupportUsers as apiListSupportUsers,
  loginUser,
  logoutUser,
  unlockTecnicoCritical,
} from '$lib/api/runtime/client.js';
import { loadSettings } from '$lib/stores/settings.js';

const LOCKED_AUTH_STATE = {
  is_authenticated: false,
  username: null,
  profile: null,
  role: 'Viewer',
  tecnico_critical_unlocked: false,
  tecnico_unlock_until_epoch: null,
};

export const authStatus = writable({ ...LOCKED_AUTH_STATE });

export const authReady = writable(false);

export async function refreshAuthStatus() {
  try {
    const status = await getAuthStatus();
    authStatus.set(status);
    authReady.set(true);
    return status;
  } catch (e) {
    authStatus.set({ ...LOCKED_AUTH_STATE });
    authReady.set(true);
    console.error('Auth status failed:', e);
    return { ...LOCKED_AUTH_STATE };
  }
}

export async function login(username, password) {
  const status = await loginUser(username, password);
  if (!status?.is_authenticated) {
    throw new Error('Credenciales invalidas o sesion no iniciada.');
  }
  authStatus.set(status);
  await loadSettings();
  return status;
}

export async function logout() {
  await logoutUser();
  authStatus.set({ ...LOCKED_AUTH_STATE });
}

export async function unlockTechnicianCritical(password, minutes = 15) {
  const ok = await unlockTecnicoCritical(password, minutes);
  await refreshAuthStatus();
  await loadSettings();
  return ok;
}

export async function listSupportUsers() {
  return apiListSupportUsers();
}

export async function createSupportUser(username, password, profile) {
  await apiCreateSupportUser(username, password, profile);
}

export async function deleteSupportUser(username) {
  await apiDeleteSupportUser(username);
}
