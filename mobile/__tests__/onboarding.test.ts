import {
  canConfirmLanguages,
  normalizeLanguages,
  ReadingLanguage,
} from '../src/lib/onboarding/languages';
import {
  getStoredLanguages,
  setStoredLanguages,
} from '../src/lib/storage/secureStore';
import { syncReadingLanguages } from '../src/lib/api/identityApi';

const mockFetch = jest.fn();
(global as any).fetch = mockFetch;

describe('onboarding langues et identité anonyme', () => {
  test('multi-sélection normalisée : conserve les langues prises en charge et retire les doublons', () => {
    expect(normalizeLanguages(['fr', 'en', 'fr', 'invalid', 'ja'])).toEqual(['fr', 'en', 'ja']);
    expect(canConfirmLanguages([])).toBe(false);
    expect(canConfirmLanguages(['zh'])).toBe(true);
  });

  test('les langues persistent localement entre deux lancements', async () => {
    const languages: ReadingLanguage[] = ['fr', 'es'];
    await setStoredLanguages(languages);
    expect(await getStoredLanguages()).toEqual(languages);
  });

  test('la synchronisation serveur porte exactement les langues choisies', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({ language_tags: ['fr', 'en'] }),
    });
    await syncReadingLanguages('22222222-2222-2222-2222-222222222222', ['fr', 'en']);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toContain('/api/v1/users/22222222-2222-2222-2222-222222222222/languages');
    expect(JSON.parse(init.body)).toEqual({ language_tags: ['fr', 'en'] });
  });
});
