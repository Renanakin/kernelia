const QUOTA_PATTERNS = [
  /429/,
  /quota/i,
  /resource_exhausted/i,
  /too many requests/i,
  /rate limit/i,
];

export function normalizeAppError(error) {
  const raw = String(error ?? '').trim();

  if (QUOTA_PATTERNS.some((rx) => rx.test(raw))) {
    return {
      code: 'quota_exceeded',
      userMessage:
        'Limite de cuota excedido (429). Espera unos segundos y vuelve a intentar.',
      raw,
    };
  }

  if (!raw) {
    return {
      code: 'unknown',
      userMessage: 'Error inesperado. Intenta nuevamente.',
      raw: '',
    };
  }

  return {
    code: 'generic',
    userMessage: raw,
    raw,
  };
}
