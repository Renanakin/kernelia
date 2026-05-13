import { writable } from 'svelte/store';

function createToastStore() {
  const { subscribe, update } = writable([]);

  function add(message, type = 'info', duration = 3000) {
    const id = Math.random().toString(36).substring(2, 9);
    update(all => [{ id, message, type }, ...all]);
    
    if (duration > 0) {
      setTimeout(() => {
        remove(id);
      }, duration);
    }
    return id;
  }

  function remove(id) {
    update(all => all.filter(t => t.id !== id));
  }

  return {
    subscribe,
    add,
    remove,
    success: (m, d) => add(m, 'success', d),
    error: (m, d) => add(m, 'error', d),
    info: (m, d) => add(m, 'info', d)
  };
}

export const toasts = createToastStore();
