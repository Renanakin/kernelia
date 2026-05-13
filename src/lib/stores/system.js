import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<string|null>} */
export const systemInfo = writable(null);

/** @type {import('svelte/store').Writable<boolean>} */
export const systemPanelOpen = writable(false);

/** @type {import('svelte/store').Writable<string>} */
export const appVersion = writable('0.1.0');
