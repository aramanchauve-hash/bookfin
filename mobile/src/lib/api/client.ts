import { Platform } from 'react-native';
import Constants from 'expo-constants';

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public payload?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }

  get isEndOfEdition(): boolean {
    return this.status === 404 && this.payload?.error === 'end_of_edition';
  }

  /** The strict random pool has no unseen eligible page; this is not a network failure. */
  get isFeedExhausted(): boolean {
    return this.status === 404 && this.payload?.error === 'feed_exhausted';
  }

  get isDuplicateReaction(): boolean {
    return this.status === 409;
  }

  get isInvalidCode(): boolean {
    return this.status === 403;
  }

  get isCodeExpired(): boolean {
    return this.status === 410;
  }

  /**
   * 422 de validation des métriques de lecture (temps trop court, scroll insuffisant,
   * délai serveur insuffisant). C'est un état récupérable et attendu du produit,
   * PAS une erreur réseau/serveur : le lecteur n'a simplement pas encore assez lu.
   */
  get isReadingValidationError(): boolean {
    if (this.status !== 422) return false;
    if (this.payload && typeof this.payload === 'object' && 'error' in this.payload) {
      return this.payload.error === 'reading_validation_failed';
    }
    // Repli : sur l'endpoint /api/v1/reactions, 422 n'est utilisé que pour ce cas.
    return true;
  }

  /** Code de raison sanitisé pour les logs dev (ex: "insufficient_scroll"). Jamais de contenu utilisateur. */
  get validationReason(): string | undefined {
    if (this.payload && typeof this.payload === 'object' && 'reason' in this.payload) {
      return this.payload.reason;
    }
    return undefined;
  }
}

/** Détection d'environnement dev/alpha (Expo définit global.__DEV__ au runtime). */
export function isDevBuild(): boolean {
  return typeof __DEV__ !== 'undefined' ? __DEV__ : process.env.NODE_ENV !== 'production';
}

export function getAppVersions(): { appVersion: string; buildVersion: string } {
  const appVersion = Constants.expoConfig?.version || '0.1.0-alpha';
  const buildVersion =
    Constants.expoConfig?.android?.versionCode?.toString() || '1';
  return { appVersion, buildVersion };
}

/**
 * Résolution de l'URL de base de l'API Bookfin :
 * - En build de production/alpha (!__DEV__) : EXPO_PUBLIC_API_BASE_URL est obligatoire
 *   et ne doit JAMAIS pointer vers localhost/10.0.2.2.
 * - En dev local (__DEV__) : repli sur 127.0.0.1 / localhost si la variable n'est pas définie.
 */
export function getApiBaseUrl(): string {
  const envUrl = process.env.EXPO_PUBLIC_API_BASE_URL;
  const isDev = isDevBuild();

  if (!isDev) {
    if (!envUrl) {
      throw new Error(
        'Configuration invalide : EXPO_PUBLIC_API_BASE_URL est obligatoire en build alpha/production'
      );
    }
    if (
      envUrl.includes('localhost') ||
      envUrl.includes('127.0.0.1') ||
      envUrl.includes('10.0.2.2')
    ) {
      throw new Error(
        `Configuration invalide : aucune URL localhost/10.0.2.2 n'est admise en build alpha (${envUrl})`
      );
    }
    return envUrl.replace(/\/+$/, '');
  }

  if (envUrl) {
    return envUrl.replace(/\/+$/, '');
  }

  if (Platform.OS === 'android') {
    return 'http://127.0.0.1:3000';
  }
  return 'http://localhost:3000';
}

export interface RequestOptions extends RequestInit {
  timeoutMs?: number;
}

export async function requestJson<T>(
  endpoint: string,
  options: RequestOptions = {}
): Promise<T> {
  const { timeoutMs = 10000, ...fetchOptions } = options;
  const baseUrl = getApiBaseUrl();
  const url = `${baseUrl}${endpoint.startsWith('/') ? '' : '/'}${endpoint}`;
  const { appVersion, buildVersion } = getAppVersions();

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(url, {
      ...fetchOptions,
      signal: controller.signal,
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'X-App-Version': appVersion,
        'X-Build-Version': buildVersion,
        ...fetchOptions.headers,
      },
    });

    const text = await response.text();
    let data: any = null;
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        data = text;
      }
    }

    if (!response.ok) {
      const msg =
        (typeof data === 'object' && data?.message) ||
        (typeof data === 'string' && data) ||
        `Erreur HTTP ${response.status}`;
      throw new ApiError(response.status, msg, data);
    }

    return data as T;
  } catch (error: any) {
    if (error.name === 'AbortError') {
      throw new ApiError(408, `Délai d'attente dépassé (${timeoutMs} ms)`);
    }
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(0, error.message || 'Impossible de joindre le serveur Bookfin');
  } finally {
    clearTimeout(timer);
  }
}

export interface ClaimInviteResult {
  user_id: string;
  claimed: boolean;
  message: string;
}

export async function claimAlphaInvite(
  code: string,
  deviceSummary?: string
): Promise<ClaimInviteResult> {
  const { appVersion, buildVersion } = getAppVersions();
  return await requestJson<ClaimInviteResult>('/api/v1/alpha/claim', {
    method: 'POST',
    body: JSON.stringify({
      code,
      device_summary: deviceSummary || `${Platform.OS} ${Platform.Version}`,
      app_version: appVersion,
      build_version: buildVersion,
    }),
  });
}

export async function verifyAlphaUser(userId: string): Promise<{ valid: boolean; user_id: string }> {
  return await requestJson<{ valid: boolean; user_id: string }>(
    `/api/v1/alpha/verify?user_id=${encodeURIComponent(userId)}`,
    { method: 'GET' }
  );
}
