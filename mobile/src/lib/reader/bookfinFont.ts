/**
 * WebView font configuration. Poliphili is licensed for mobile-app embedding;
 * data URIs are injected into the persistent WebView shell exactly once.
 */
import { POLIPHILI_ROMAN_DATA_URI, POLIPHILI_ITALIC_DATA_URI } from './poliphiliDataUri';

export type ReaderFontMode = 'poliphili' | 'system';

export type EmbeddedFontSource = {
  family: string;
  romanDataUri?: string;
  italicDataUri?: string;
};

export const POLIPHILI_FAMILY = 'Bookfin Poliphili';
export const POLIPHILI_LICENSE_STATUS = 'POLIPHILI NOT INSTALLED — LICENSE REQUIRED';
export const BOOKFIN_FONT_FALLBACK = '"Times New Roman", Georgia, serif';
export const BOOKFIN_FONT_FAMILY_STACK = `"${POLIPHILI_FAMILY}", ${BOOKFIN_FONT_FALLBACK}`;

// Licensed Poliphili font source
export const poliphiliFontSource: EmbeddedFontSource = {
  family: POLIPHILI_FAMILY,
  romanDataUri: POLIPHILI_ROMAN_DATA_URI,
  italicDataUri: POLIPHILI_ITALIC_DATA_URI,
};

function detectFormat(dataUri: string): string {
  if (dataUri.includes('font/ttf') || dataUri.includes('font/truetype')) return 'truetype';
  if (dataUri.includes('font/otf') || dataUri.includes('font/opentype')) return 'opentype';
  return 'woff2';
}

export function createFontFaceCss(source: EmbeddedFontSource): string {
  const rules: string[] = [];
  if (source.romanDataUri) {
    const fmt = detectFormat(source.romanDataUri);
    rules.push(`@font-face{font-family:"${source.family}";src:url("${source.romanDataUri}") format("${fmt}");font-style:normal;font-weight:400;font-display:block}`);
  }
  if (source.italicDataUri) {
    const fmt = detectFormat(source.italicDataUri);
    rules.push(`@font-face{font-family:"${source.family}";src:url("${source.italicDataUri}") format("${fmt}");font-style:italic;font-weight:400;font-display:block}`);
  }
  return rules.join('');
}

export const BOOKFIN_POLIPHILI_FONT_FACE_CSS = createFontFaceCss(poliphiliFontSource);
export const poliphiliRomanIsInstalled = Boolean(poliphiliFontSource.romanDataUri);
export const poliphiliItalicIsInstalled = Boolean(poliphiliFontSource.italicDataUri);
export const poliphiliIsInstalled = poliphiliRomanIsInstalled;
