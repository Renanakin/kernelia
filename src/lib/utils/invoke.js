import { invoke } from '@tauri-apps/api/core';
import { normalizeAppError } from './errors.js';
import { tryDirectLocalCommand } from './localDirect.js';

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function hasTauriRuntime() {
  try {
    return typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__;
  } catch {
    return false;
  }
}

export async function invokeWithPolicy(command, payload = {}, options = {}) {
  const {
    timeoutMs = 15000,
    retries = 1,
    retryDelayMs = 500,
  } = options;

  let attempt = 0;
  let lastErr;

  // In browser mode without Tauri runtime, skip invoke() to avoid pending calls.
  if (!hasTauriRuntime()) {
    try {
      return await tryDirectLocalCommand(command, payload);
    } catch (error) {
      const normalized = normalizeAppError(error);
      const err = new Error(normalized.userMessage);
      err.code = normalized.code;
      err.raw = normalized.raw;
      throw err;
    }
  }

  while (attempt <= retries) {
    try {
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error(`Timeout (${timeoutMs}ms) en ${command}`)), timeoutMs);
      });
      return await Promise.race([invoke(command, payload), timeoutPromise]);
    } catch (error) {
      lastErr = error;
      if (attempt >= retries) break;
      await sleep(retryDelayMs * (attempt + 1));
    }
    attempt += 1;
  }

  if (String(lastErr?.message || '').includes("undefined (reading 'invoke')")) {
    try {
      return await tryDirectLocalCommand(command, payload);
    } catch {
      const err = new Error('Este flujo requiere runtime de Tauri. Ejecuta la app con pnpm tauri dev.');
      err.code = 'TAURI_RUNTIME_REQUIRED';
      err.raw = lastErr;
      throw err;
    }
  }

  try {
    return await tryDirectLocalCommand(command, payload);
  } catch {
    // No local fallback available for this command.
  }

  const normalized = normalizeAppError(lastErr);
  const err = new Error(normalized.userMessage);
  err.code = normalized.code;
  err.raw = normalized.raw;
  throw err;
}

export async function invokeTool(name, args = {}, options = {}) {
  return invokeWithPolicy('execute_tool', { name, args }, options);
}
