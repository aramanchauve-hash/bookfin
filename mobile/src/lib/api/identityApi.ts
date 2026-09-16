import { requestJson } from './client';
import { ReadingLanguage } from '../onboarding/languages';

export interface AnonymousIdentityDto {
  user_id: string;
  identity_kind: 'anonymous';
}

export async function createAnonymousIdentity(): Promise<AnonymousIdentityDto> {
  return requestJson<AnonymousIdentityDto>('/api/v1/identities/anonymous', { method: 'POST' });
}

export async function syncReadingLanguages(
  userId: string,
  languageTags: readonly ReadingLanguage[]
): Promise<{ language_tags: ReadingLanguage[] }> {
  return requestJson<{ language_tags: ReadingLanguage[] }>(
    `/api/v1/users/${encodeURIComponent(userId)}/languages`,
    {
      method: 'PUT',
      body: JSON.stringify({ language_tags: languageTags }),
    }
  );
}
