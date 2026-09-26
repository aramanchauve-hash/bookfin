import { detectSystemLanguage, formatString, SUPPORTED_LANGUAGES_LIST } from '../src/i18n';
import { fr } from '../src/i18n/fr';
import { en } from '../src/i18n/en';
import { es } from '../src/i18n/es';
import { ru } from '../src/i18n/ru';
import { zh } from '../src/i18n/zh';
import { ja } from '../src/i18n/ja';
import { getStoredAppLanguage, setStoredAppLanguage } from '../src/lib/storage/secureStore';

describe('Internationalisation UX (i18n)', () => {
  const translations = { fr, en, es, ru, zh, ja };

  test('toutes les 6 langues prises en charge possèdent les clés requises', () => {
    expect(SUPPORTED_LANGUAGES_LIST).toEqual(['fr', 'en', 'es', 'ru', 'zh', 'ja']);

    for (const code of SUPPORTED_LANGUAGES_LIST) {
      const t = translations[code];
      expect(t).toBeDefined();

      // common
      expect(t.common.loading).toBeTruthy();
      expect(t.common.close).toBeTruthy();
      expect(t.common.back).toBeTruthy();
      expect(t.common.error).toBeTruthy();
      expect(t.common.retry).toBeTruthy();

      // identity
      expect(t.identity.title).toBeTruthy();
      expect(t.identity.continue_without_account).toBeTruthy();

      // languages
      expect(t.languages.onboarding_title).toBeTruthy();
      expect(t.languages.continue_button).toBeTruthy();

      // reading
      expect(t.reading.start_from_beginning).toBeTruthy();
      expect(t.reading.back_to_discovery).toBeTruthy();
      expect(t.reading.profile_button).toBeTruthy();
      expect(t.reading.page_progress).toContain('{current}');

      // library
      expect(t.library.title).toBeTruthy();
      expect(t.library.empty_title).toBeTruthy();
      expect(t.library.resume).toBeTruthy();

      // profile
      expect(t.profile.title).toBeTruthy();
      expect(t.profile.library).toBeTruthy();
      expect(t.profile.settings).toBeTruthy();
      expect(t.profile.anonymous_reader).toBeTruthy();

      // settings
      expect(t.settings.title).toBeTruthy();
      expect(t.settings.app_language_section).toBeTruthy();
      expect(t.settings.reading_languages_section).toBeTruthy();
    }
  });

  test('formatString interpole correctement les variables', () => {
    const formatted = formatString('Page {current} / {total}', { current: 12, total: 243 });
    expect(formatted).toBe('Page 12 / 243');

    const multi = formatString('{title} par {author}', { title: 'Candide', author: 'Voltaire' });
    expect(multi).toBe('Candide par Voltaire');
  });

  test('detectSystemLanguage utilise la locale ou retombe sur en', () => {
    const lang = detectSystemLanguage();
    expect(SUPPORTED_LANGUAGES_LIST).toContain(lang);
  });

  test('la langue sélectionnée manuellement est persistée dans SecureStore', async () => {
    await setStoredAppLanguage('es');
    const stored = await getStoredAppLanguage();
    expect(stored).toBe('es');

    await setStoredAppLanguage('fr');
    const updated = await getStoredAppLanguage();
    expect(updated).toBe('fr');
  });
});
