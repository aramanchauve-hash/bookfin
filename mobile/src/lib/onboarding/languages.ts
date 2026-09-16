export const SUPPORTED_READING_LANGUAGES = [
  { code: 'fr', label: 'Fran\u00e7ais' },
  { code: 'en', label: 'English' },
  { code: 'es', label: 'Espa\u00f1ol' },
  { code: 'ru', label: '\u0420\u0443\u0441\u0441\u043a\u0438\u0439' },
  { code: 'zh', label: '\u4e2d\u6587' },
  { code: 'ja', label: '\u65e5\u672c\u8a9e' },
] as const;

export type ReadingLanguage = (typeof SUPPORTED_READING_LANGUAGES)[number]['code'];

export function normalizeLanguages(languages: readonly string[]): ReadingLanguage[] {
  const allowed = new Set<string>(SUPPORTED_READING_LANGUAGES.map((language) => language.code));
  return [...new Set(languages)].filter((language): language is ReadingLanguage => allowed.has(language));
}

export function canConfirmLanguages(languages: readonly string[]): boolean {
  return normalizeLanguages(languages).length > 0;
}
