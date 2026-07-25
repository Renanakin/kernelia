import { marked } from 'marked';
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';

// Configurar marked con highlight.js
marked.setOptions({
  breaks: true,
  gfm: true,
});

// Renderer personalizado para code blocks con botón de copiar
const renderer = new marked.Renderer();

renderer.code = function ({ text, lang }) {
  const language = lang && hljs.getLanguage(lang) ? lang : 'plaintext';
  const highlighted = hljs.highlight(text, { language }).value;
  const langLabel = lang || 'text';
  const encodedText = encodeURIComponent(text);
  return `<div class="code-block-wrapper" data-code="${encodedText}">
    <div class="code-block-header">
      <span class="code-lang">${langLabel}</span>
      <button class="copy-btn" data-action="copy-code">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"></path>
        </svg>
        Copiar
      </button>
    </div>
    <pre><code class="hljs language-${language}">${highlighted}</code></pre>
  </div>`;
};

marked.use({ renderer });

/**
 * Renderiza Markdown a HTML seguro
 * @param {string} text - Texto en Markdown
 * @returns {string} HTML sanitizado
 */
export function renderMarkdown(text) {
  if (!text) return '';
  
  try {
    const html = marked.parse(text);
    // FLUJO SEGURO XSS: Sanitizar HTML con prohibición explícita de atributos ejecutables
    return DOMPurify.sanitize(html, {
      ADD_TAGS: ['svg', 'path', 'rect'],
      ADD_ATTR: ['viewBox', 'fill', 'stroke', 'stroke-width', 'd', 'x', 'y', 'width', 'height', 'rx', 'ry', 'data-action', 'data-code'],
      FORBID_ATTR: ['onclick', 'onerror', 'onload', 'onmouseover'],
    });
  } catch (e) {
    console.error('Markdown render error:', e);
    return DOMPurify.sanitize(text, { FORBID_ATTR: ['onclick', 'onerror', 'onload'] });
  }
}
