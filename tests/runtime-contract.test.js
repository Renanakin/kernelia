import test from 'node:test';
import assert from 'node:assert/strict';
import { tryDirectLocalCommand } from '../src/lib/utils/localDirect.js';

function createStorage() {
  const map = new Map();
  return {
    getItem(key) {
      return map.has(key) ? map.get(key) : null;
    },
    setItem(key, value) {
      map.set(key, String(value));
    },
    removeItem(key) {
      map.delete(key);
    },
    clear() {
      map.clear();
    },
  };
}

global.localStorage = createStorage();

test('set_model accepts snake_case payload', async () => {
  const ok = await tryDirectLocalCommand('set_model', { model_id: 'gemma3-local' });
  assert.equal(ok, true);

  const settings = await tryDirectLocalCommand('get_settings');
  assert.equal(settings.selected_model, 'gemma3-local');
});

test('set_model accepts camelCase payload', async () => {
  const ok = await tryDirectLocalCommand('set_model', { modelId: 'gemma3-local' });
  assert.equal(ok, true);
});

test('set_api_key validates non-empty value', async () => {
  await assert.rejects(
    () => tryDirectLocalCommand('set_api_key', { api_key: '' }),
    /API_VALIDATION/
  );

  const ok = await tryDirectLocalCommand('set_api_key', { api_key: 'token-demo' });
  assert.equal(ok, true);
});
