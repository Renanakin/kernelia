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
export const ragEngineEnabled = writable(true);
export const ragCompareMode = writable(false);
export const ragDebugPanel = writable(false);
export const ragShowConfidenceBadge = writable(true);

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
    ragEngineEnabled.set(s.rag_engine_enabled);
    ragCompareMode.set(s.rag_compare_mode);
    ragDebugPanel.set(s.rag_debug_panel);
    ragShowConfidenceBadge.set(s.rag_show_confidence_badge);

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
    if (typeof updates.rag_engine_enabled === 'boolean') ragEngineEnabled.set(updates.rag_engine_enabled);
    if (typeof updates.rag_compare_mode === 'boolean') ragCompareMode.set(updates.rag_compare_mode);
    if (typeof updates.rag_debug_panel === 'boolean') ragDebugPanel.set(updates.rag_debug_panel);
    if (typeof updates.rag_show_confidence_badge === 'boolean') ragShowConfidenceBadge.set(updates.rag_show_confidence_badge);

    return true;
  } catch (e) {
    console.error('Failed to save settings:', e);
    return false;
  }
}
