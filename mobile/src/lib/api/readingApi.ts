import { requestJson } from './client';
import { getCurrentUserId } from '../auth/userContext';
import {
  ContinueReadingRequestDto,
  FeedPageDto,
  PageRevealDto,
  SubmitReactionRequestDto,
  SubmitReactionResponseDto,
} from '../../types/api';

/**
 * Récupère la page suivante du flux anonyme (RANDOM_PAGE).
 * Garanti strictement anonyme sans titre ni auteur avant réaction.
 */
export async function fetchNextRandomPage(userId?: string): Promise<FeedPageDto> {
  const uid = userId || getCurrentUserId();
  return requestJson<FeedPageDto>(`/api/v1/feed/next?user_id=${encodeURIComponent(uid)}`, {
    method: 'GET',
  });
}

/**
 * Soumet une réaction (Like / Dislike) avec métriques de lecture.
 * Utilise un event_id client unique garantissant l'idempotence réseau.
 */
export async function submitReaction(
  payload: SubmitReactionRequestDto
): Promise<SubmitReactionResponseDto> {
  return requestJson<SubmitReactionResponseDto>('/api/v1/reactions', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

/**
 * Révèle l'auteur et le titre après réaction confirmée.
 * Protégé côté serveur (renvoie 403 si aucune réaction enregistrée).
 */
export async function revealPageMetadata(
  pageId: string,
  userId?: string
): Promise<PageRevealDto> {
  const uid = userId || getCurrentUserId();
  return requestJson<PageRevealDto>(
    `/api/v1/pages/${encodeURIComponent(pageId)}/reveal?user_id=${encodeURIComponent(uid)}`,
    {
      method: 'GET',
    }
  );
}

/**
 * Demande la page suivante dans la même édition (CONTINUE_BOOK).
 * Gère le retour 404 avec 'end_of_edition' si dernière page de l'œuvre.
 */
export async function continueReading(
  pageId: string,
  parentImpressionId?: string,
  userId?: string,
  idempotencyKey?: string
): Promise<FeedPageDto> {
  const uid = userId || getCurrentUserId();
  const body: ContinueReadingRequestDto = {
    user_id: uid,
    parent_impression_id: parentImpressionId,
    idempotency_key: idempotencyKey,
  };

  return requestJson<FeedPageDto>(
    `/api/v1/pages/${encodeURIComponent(pageId)}/continue`,
    {
      method: 'POST',
      body: JSON.stringify(body),
    }
  );
}

/**
 * Démarre une œuvre depuis le début (page 1) et l'ajoute à la bibliothèque personnelle.
 * Sur une œuvre déjà commencée, le serveur refuse (409 `work_already_started`) sans
 * toucher à la progression, sauf redémarrage explicite (`restart: true`).
 */
export async function startWork(
  workId: string,
  editionId: string,
  options: { restart?: boolean } = {},
  userId?: string
): Promise<FeedPageDto> {
  const uid = userId || getCurrentUserId();
  return requestJson<FeedPageDto>(`/api/v1/works/${encodeURIComponent(workId)}/start`, {
    method: 'POST',
    body: JSON.stringify({ user_id: uid, edition_id: editionId, restart: options.restart === true }),
  });
}

/**
 * Reprend la lecture suivie d'une œuvre à sa page actuelle sauvegardée.
 */
export async function resumeWork(
  workId: string,
  userId?: string
): Promise<FeedPageDto> {
  const uid = userId || getCurrentUserId();
  return requestJson<FeedPageDto>(`/api/v1/works/${encodeURIComponent(workId)}/resume`, {
    method: 'POST',
    body: JSON.stringify({ user_id: uid }),
  });
}

/**
 * Récupère la bibliothèque personnelle des œuvres commencées par le lecteur.
 */
export async function fetchUserLibrary(
  userId?: string
): Promise<import('../../types/api').UserLibraryItemDto[]> {
  const uid = userId || getCurrentUserId();
  return requestJson<import('../../types/api').UserLibraryItemDto[]>(
    `/api/v1/me/library?user_id=${encodeURIComponent(uid)}`,
    {
      method: 'GET',
    }
  );
}


