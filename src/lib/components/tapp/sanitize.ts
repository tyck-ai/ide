/** Sanitize CSS style strings from WASM to prevent exfiltration via url() etc. */
export function sanitizeStyle(style: unknown): string {
  if (typeof style !== 'string') return '';
  // Normalize CSS unicode escapes (e.g. \75 for 'u') and backslash escapes,
  // then check for dangerous patterns. Return the NORMALIZED string so that
  // unicode-escaped attacks like \75\72\6c() can't bypass the check.
  const normalized = style.replace(/\\[0-9a-fA-F]{1,6}\s?/g, '_').replace(/\\/g, '');
  if (/url\s*\(/i.test(normalized) ||
      /expression\s*\(/i.test(normalized) ||
      /-moz-binding/i.test(normalized) ||
      /@import/i.test(normalized) ||
      /behavior\s*:/i.test(normalized)) {
    return '';
  }
  return normalized;
}

/** Validate image src — only allow http(s) and relative paths */
export function sanitizeSrc(src: unknown): string {
  if (typeof src !== 'string') return '';
  const trimmed = src.trim().toLowerCase();
  if (
    trimmed.startsWith('javascript:') ||
    trimmed.startsWith('data:') ||
    trimmed.startsWith('blob:') ||
    trimmed.startsWith('//')
  ) {
    return '';
  }
  return src;
}
