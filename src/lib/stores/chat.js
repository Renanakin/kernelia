import { writable, derived } from 'svelte/store';

/**
 * @typedef {Object} Message
 * @property {string} id
 * @property {'user' | 'assistant' | 'system' | 'tool'} role
 * @property {string} content
 * @property {string} timestamp
 * @property {Array<ToolUse>} [toolsUsed]
 * @property {string} [model]
 * @property {string} [error]
 * @property {boolean} [isLoading]
 */

/**
 * @typedef {Object} ToolUse
 * @property {string} name
 * @property {string} arguments
 */

/** @type {import('svelte/store').Writable<Message[]>} */
export const messages = writable([]);

/** @type {import('svelte/store').Writable<boolean>} */
export const isLoading = writable(false);

/** @type {import('svelte/store').Writable<string>} */
export const inputText = writable('');

/** Genera un ID único para cada mensaje */
function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

/** Obtiene timestamp formateado */
function getTimestamp() {
  return new Date().toLocaleTimeString('es-CL', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Agrega un mensaje del usuario
 * @param {string} content
 * @returns {Message}
 */
export function addUserMessage(content) {
  /** @type {Message} */
  const msg = {
    id: generateId(),
    role: 'user',
    content,
    timestamp: getTimestamp(),
  };
  messages.update((msgs) => [...msgs, msg]);
  return msg;
}

/**
 * Agrega un mensaje de carga (placeholder mientras el modelo responde)
 * @returns {string} ID del mensaje de carga
 */
export function addLoadingMessage() {
  const id = generateId();
  /** @type {Message} */
  const msg = {
    id,
    role: 'assistant',
    content: '',
    timestamp: getTimestamp(),
    isLoading: true,
  };
  messages.update((msgs) => [...msgs, msg]);
  return id;
}

/**
 * Reemplaza el mensaje de carga con la respuesta real
 * @param {string} loadingId
 * @param {string} content
 * @param {Array<ToolUse>} [toolsUsed]
 * @param {string} [model]
 * @param {string} [error]
 */
export function resolveLoadingMessage(loadingId, content, toolsUsed, model, error) {
  messages.update((msgs) =>
    msgs.map((msg) =>
      msg.id === loadingId
        ? {
            ...msg,
            content,
            toolsUsed: toolsUsed || [],
            model,
            error,
            isLoading: false,
          }
        : msg
    )
  );
}

/**
 * Agrega un mensaje del sistema (bienvenida, etc.)
 * @param {string} content
 */
export function addSystemMessage(content) {
  /** @type {Message} */
  const msg = {
    id: generateId(),
    role: 'system',
    content,
    timestamp: getTimestamp(),
  };
  messages.update((msgs) => [...msgs, msg]);
}

/**
 * Actualiza un mensaje existente
 * @param {string} id
 * @param {Partial<Message>} update
 */
export function updateMessage(id, update) {
  messages.update((msgs) =>
    msgs.map((msg) => (msg.id === id ? { ...msg, ...update } : msg))
  );
}

/**
 * Agrega texto al final de un mensaje
 * @param {string} id
 * @param {string} delta
 */
export function appendToMessage(id, delta) {
  messages.update((msgs) =>
    msgs.map((msg) =>
      msg.id === id ? { ...msg, content: msg.content + delta, isLoading: false } : msg
    )
  );
}

/** Limpia todos los mensajes */
export function clearMessages() {
  messages.set([]);
}

/** Contador de mensajes */
export const messageCount = derived(messages, ($messages) => $messages.length);
