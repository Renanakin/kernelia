export const MS_ALLOWED_DOMAINS = [
  'learn.microsoft.com',
  'support.microsoft.com',
  'answers.microsoft.com',
  'techcommunity.microsoft.com',
];

export function is_whitelisted_microsoft_url(url) {
  const lowercase = String(url || '').toLowerCase();
  return MS_ALLOWED_DOMAINS.some((d) => lowercase.includes(d));
}

export function build_microsoft_search_query(userQuery) {
  const clean = String(userQuery || '')
    .replace(/site:/gi, '')
    .trim();
  
  return `${clean} (site:learn.microsoft.com OR site:support.microsoft.com OR site:answers.microsoft.com OR site:techcommunity.microsoft.com)`;
}
