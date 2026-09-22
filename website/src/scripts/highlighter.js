// markdown_renderer.js
import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/es/highlight.min.js';

export function highlightCode(code, lang) {
  try {
    // Use highlight.js for code syntax highlighting
    const language = hljs.getLanguage(lang) ? lang : 'plaintext';
    return hljs.highlight(code, { language }).value;
  } catch (err) {
    console.error("Error highlighting code:", err);
    return code; // Return original code on error
  }
}