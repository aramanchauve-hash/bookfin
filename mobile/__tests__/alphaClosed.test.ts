import {
  claimAlphaInvite,
  getApiBaseUrl,
  getAppVersions,
  requestJson,
  verifyAlphaUser,
} from '../src/lib/api/client';
import {
  clearStoredAlphaIdentity,
  getStoredConsentGiven,
  getStoredUserId,
  setStoredConsentGiven,
  setStoredUserId,
} from '../src/lib/storage/secureStore';
import { getCurrentUserId, setCurrentUserId, DEFAULT_DEV_USER_ID } from '../src/lib/auth/userContext';

const mockFetch = jest.fn();
(global as any).fetch = mockFetch;

describe('Bookfin Closed Alpha - Mobile Core Tests', () => {
  beforeEach(() => {
    mockFetch.mockReset();
    setCurrentUserId(null);
  });

  describe('Stockage sécurisé & anonymat', () => {
    test('Persistance et lecture du userId alpha sans fuite', async () => {
      await clearStoredAlphaIdentity();
      expect(await getStoredUserId()).toBeNull();

      const testId = '22222222-2222-2222-2222-222222222222';
      await setStoredUserId(testId);
      expect(await getStoredUserId()).toBe(testId);

      await clearStoredAlphaIdentity();
      expect(await getStoredUserId()).toBeNull();
    });

    test('Persistance et lecture du consentement explicite', async () => {
      await clearStoredAlphaIdentity();
      expect(await getStoredConsentGiven()).toBe(false);

      await setStoredConsentGiven(true);
      expect(await getStoredConsentGiven()).toBe(true);
    });

    test('userContext : utilise memoryUserId si défini, sinon repli dev', () => {
      expect(getCurrentUserId()).toBe(DEFAULT_DEV_USER_ID);

      const customUser = '33333333-3333-3333-3333-333333333333';
      setCurrentUserId(customUser);
      expect(getCurrentUserId()).toBe(customUser);

      setCurrentUserId(null);
      expect(getCurrentUserId()).toBe(DEFAULT_DEV_USER_ID);
    });
  });

  describe('Règles d\'URL d\'environnement alpha & Versioning', () => {
    const originalEnv = process.env;

    beforeEach(() => {
      process.env = { ...originalEnv };
    });

    afterAll(() => {
      process.env = originalEnv;
    });

    test('Récupération automatique de la version et build_version', () => {
      const { appVersion, buildVersion } = getAppVersions();
      expect(appVersion).toBe('0.1.0-alpha');
      expect(buildVersion).toBe('1');
    });

    test('En production/alpha : refuse strictement localhost et 10.0.2.2', () => {
      // Simulation environnement de production
      (global as any).__DEV__ = false;
      process.env.NODE_ENV = 'production';

      process.env.EXPO_PUBLIC_API_BASE_URL = 'http://localhost:3000';
      expect(() => getApiBaseUrl()).toThrow(/aucune URL localhost\/10\.0\.2\.2 n'est admise/);

      process.env.EXPO_PUBLIC_API_BASE_URL = 'http://10.0.2.2:3000';
      expect(() => getApiBaseUrl()).toThrow(/aucune URL localhost\/10\.0\.2\.2 n'est admise/);

      process.env.EXPO_PUBLIC_API_BASE_URL = 'https://alpha-api.bookfin.app';
      expect(getApiBaseUrl()).toBe('https://alpha-api.bookfin.app');

      (global as any).__DEV__ = true;
    });

    test('Headers X-App-Version et X-Build-Version systématiquement injectés', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: async () => JSON.stringify({ ok: true }),
      });

      await requestJson('/api/v1/test');

      expect(mockFetch).toHaveBeenCalledTimes(1);
      const [, init] = mockFetch.mock.calls[0];
      expect(init.headers['X-App-Version']).toBe('0.1.0-alpha');
      expect(init.headers['X-Build-Version']).toBe('1');
    });
  });

  describe('Validation d\'invitation Alpha (Endpoints)', () => {
    test('claimAlphaInvite appelle /api/v1/alpha/claim avec le code et la version', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        text: async () =>
          JSON.stringify({
            user_id: '44444444-4444-4444-4444-444444444444',
            claimed: true,
            message: 'Invitation validée',
          }),
      });

      const res = await claimAlphaInvite('ALPHA-TEST-01', 'Android Test Device');
      expect(res.claimed).toBe(true);
      expect(res.user_id).toBe('44444444-4444-4444-4444-444444444444');

      const [url, init] = mockFetch.mock.calls[0];
      expect(url).toContain('/api/v1/alpha/claim');
      expect(init.method).toBe('POST');
      const body = JSON.parse(init.body);
      expect(body.code).toBe('ALPHA-TEST-01');
      expect(body.app_version).toBe('0.1.0-alpha');
    });

    test('verifyAlphaUser appelle /api/v1/alpha/verify avec user_id', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: async () =>
          JSON.stringify({
            valid: true,
            user_id: '44444444-4444-4444-4444-444444444444',
          }),
      });

      const res = await verifyAlphaUser('44444444-4444-4444-4444-444444444444');
      expect(res.valid).toBe(true);
      expect(res.user_id).toBe('44444444-4444-4444-4444-444444444444');

      const [url] = mockFetch.mock.calls[0];
      expect(url).toContain('/api/v1/alpha/verify?user_id=44444444-4444-4444-4444-444444444444');
    });
  });
});

