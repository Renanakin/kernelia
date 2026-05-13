import test from 'node:test';
import assert from 'node:assert/strict';
import { normalizeAppError } from '../src/lib/utils/errors.js';

test('maps quota/rate-limit errors to friendly message', () => {
  const err = normalizeAppError('API error (429 Too Many Requests): quota exceeded');
  assert.equal(err.code, 'quota_exceeded');
  assert.match(err.userMessage, /cuota excedido|cuota excedida|429/i);
});

test('maps empty error to unknown fallback', () => {
  const err = normalizeAppError('');
  assert.equal(err.code, 'unknown');
  assert.match(err.userMessage, /inesperado/i);
});

test('keeps generic message for non-quota errors', () => {
  const err = normalizeAppError('No se pudo conectar al backend');
  assert.equal(err.code, 'generic');
  assert.equal(err.userMessage, 'No se pudo conectar al backend');
});
