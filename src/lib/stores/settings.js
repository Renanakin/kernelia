import { writable, derived } from 'svelte/store';
import { getModels, getSettings, updateSettings } from '$lib/api/runtime/client.js';

export const models = writable([]);
export const selectedModelId = writable('gemma3-local');
export const userRole = writable('Viewer');
export const isFirstRun = writable(true);
export const settingsOpen = writable(false);
export const auditOpen = writable(false);
export const sidebarTab = writable('telemetry');
export const theme = writable('dark');

export const selectedModel = derived(
  [models, selectedModelId],
  ([$models, $selectedModelId]) =>
    $models.find((m) => m.id === $selectedModelId) || $models[0] || null
);

export const configuredModels = derived(models, ($models) =>
  $models.filter((m) => m.hasApiKey || m.isLocal)
);

export async function loadSettings() {
  try {
    const s = await getSettings();
    selectedModelId.set(s.selected_model);
    userRole.set(s.user_role);
    isFirstRun.set(s.first_run);
    theme.set(s.theme);

    const modelList = await getModels();
    models.set(modelList);

    return s;
  } catch (e) {
    console.error('Failed to load settings:', e);
  }
}

export async function saveSettings(updates) {
  try {
    const current = await getSettings();
    const newSettings = { ...current, ...updates };
    await updateSettings(newSettings);

    if (updates.selected_model) selectedModelId.set(updates.selected_model);
    if (updates.user_role) userRole.set(updates.user_role);
    if (updates.theme) theme.set(updates.theme);

    return true;
  } catch (e) {
    console.error('Failed to save settings:', e);
    return false;
  }
}
