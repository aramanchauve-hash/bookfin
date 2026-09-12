import {
  canSubmitReaction,
  initialReadingState,
  readingReducer,
  ReadingState,
} from '../src/state/readingReducer';
import { resolveReactionErrorAction, READING_VALIDATION_HINT } from '../src/state/reactionErrorPolicy';
import { ApiError } from '../src/lib/api/client';
import { ReadingTrackerCore } from '../src/lib/metrics/useReadingTracker';
import { toServerScrollDepth } from '../src/lib/metrics/useReadingTracker';
import { FeedPageDto } from '../src/types/api';

/**
 * Régression bug alpha : 422 "Métriques de lecture invalides ou délai insuffisant"
 * bloquait l'écran de lecture, Réessayer ne faisait rien.
 */
describe('Récupération après 422 de validation de lecture', () => {
  const page: FeedPageDto = {
    impression_id: 'imp-001',
    id: 'imp-001',
    page_id: 'page-001',
    page_sequence_number: 1,
    page_number: 1,
    source_page_number: '1',
    text: 'Longtemps je me suis couché de bonne heure.',
    content: 'Longtemps je me suis couché de bonne heure.',
    language_tag: 'fr',
    served_at: '2026-09-12T10:00:00Z',
    token_count: 120,
  };

  function readingValidationError(reason = 'insufficient_scroll'): ApiError {
    return new ApiError(422, 'Métriques de lecture invalides ou délai insuffisant', {
      error: 'reading_validation_failed',
      reason,
      message: 'Métriques de lecture invalides ou délai insuffisant',
    });
  }

  // A. reading -> reaction -> 422 validation => retour à un état lisible/non bloquant.
  test('A. un 422 de validation renvoie vers un état lisible, pas bloqué', () => {
    let state: ReadingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page,
    });
    state = readingReducer(state, { type: 'REACT_START', reaction: 'like', eventId: 'ev-1' });
    expect(state.status).toBe('reacting');

    const action = resolveReactionErrorAction(readingValidationError());
    state = readingReducer(state, action);

    expect(state.status).toBe('reading');
    expect(state.currentPage).toEqual(page); // page conservée
    expect(state.validationHint).toBe(READING_VALIDATION_HINT);
    expect(state.errorMessage).toBeNull(); // pas la bannière d'erreur bloquante
    // La réaction redevient immédiatement disponible : le guard l'autorise depuis 'reading'.
    expect(canSubmitReaction(state.status)).toBe(true);
  });

  // B. après 422, le temps actif continue à augmenter (le tracker n'est jamais mis en pause
  // par une erreur de soumission : il ne dépend que de l'AppState et du scroll).
  test('B. le temps actif continue à s’accumuler après un 422', () => {
    jest.useFakeTimers();
    try {
      const tracker = new ReadingTrackerCore();
      jest.advanceTimersByTime(4000);
      const before = tracker.getSnapshot(100).dwell_time_ms;

      // Un 422 survient ici (hors du tracker, qui n'en a pas connaissance) : la lecture continue.
      jest.advanceTimersByTime(3000);
      const after = tracker.getSnapshot(100).dwell_time_ms;

      expect(before).toBe(4000);
      expect(after).toBe(7000);
      expect(after).toBeGreaterThan(before);
    } finally {
      jest.useRealTimers();
    }
  });

  // C. après nouveau scroll, les nouvelles métriques sont prises en compte.
  test('C. une nouvelle progression de scroll après un 422 est bien reflétée dans le prochain snapshot', () => {
    const tracker = new ReadingTrackerCore();

    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 100 },
        contentSize: { width: 360, height: 1200 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);
    const firstAttempt = tracker.getSnapshot(200);
    expect(firstAttempt.scroll_depth_percent).toBeLessThan(75);

    // Le lecteur continue de faire défiler après le 422 (bas de page atteint).
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 590 },
        contentSize: { width: 360, height: 1200 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);
    const secondAttempt = tracker.getSnapshot(200);

    expect(secondAttempt.scroll_depth_percent).toBeGreaterThan(firstAttempt.scroll_depth_percent);
    expect(secondAttempt.bottom_reached).toBe(true);
    // La fraction envoyée au serveur dépasse maintenant le seuil de 0.75.
    expect(toServerScrollDepth(secondAttempt.scroll_depth_percent)).toBeGreaterThanOrEqual(0.75);
  });

  // D. une deuxième tentative construit un snapshot différent/frais (jamais figé).
  test('D. deux tentatives successives produisent des snapshots distincts', () => {
    jest.useFakeTimers();
    try {
      const tracker = new ReadingTrackerCore();
      jest.advanceTimersByTime(4000);
      const attempt1 = tracker.getSnapshot(150);

      jest.advanceTimersByTime(5000);
      tracker.onScroll({
        nativeEvent: {
          contentOffset: { x: 0, y: 590 },
          contentSize: { width: 360, height: 1200 },
          layoutMeasurement: { width: 360, height: 600 },
        },
      } as any);
      const attempt2 = tracker.getSnapshot(150);

      expect(attempt2).not.toEqual(attempt1);
      expect(attempt2.dwell_time_ms).toBeGreaterThan(attempt1.dwell_time_ms);
      expect(attempt2.scroll_depth_percent).toBeGreaterThan(attempt1.scroll_depth_percent);
    } finally {
      jest.useRealTimers();
    }
  });

  // E. une vraie erreur réseau conserve son comportement de retry approprié.
  test('E. une erreur réseau reste une erreur bloquante avec retry, distincte du 422', () => {
    const networkError = new ApiError(0, 'Impossible de joindre le serveur Bookfin');

    const action = resolveReactionErrorAction(networkError);
    expect(action.type).toBe('REACT_ERROR');

    let state: ReadingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page,
    });
    state = readingReducer(state, { type: 'REACT_START', reaction: 'like', eventId: 'ev-net' });
    state = readingReducer(state, action);

    expect(state.status).toBe('error');
    expect(state.errorMessage).toBeTruthy();
    // L'event_id est conservé pour permettre un retry idempotent côté serveur.
    expect(state.pendingEventId).toBe('ev-net');
    // Le guard doit permettre de retenter depuis 'error' (c'était le bug : Réessayer ne faisait rien).
    expect(canSubmitReaction(state.status)).toBe(true);
  });

  test('E bis. un 409 (réaction déjà enregistrée) est traité comme une erreur dure, pas une validation', () => {
    const alreadyReacted = new ApiError(409, 'Cette impression a déjà reçu une réaction');
    expect(resolveReactionErrorAction(alreadyReacted).type).toBe('REACT_ERROR');
  });

  // F. double tap reste protégé : la state machine ne permet une (nouvelle) tentative
  // que depuis 'reading' ou 'error', jamais pendant 'reacting'.
  test('F. le guard de soumission bloque toute tentative pendant reacting/loading/navigating/revealed', () => {
    expect(canSubmitReaction('reading')).toBe(true);
    expect(canSubmitReaction('error')).toBe(true);
    expect(canSubmitReaction('reacting')).toBe(false);
    expect(canSubmitReaction('loading')).toBe(false);
    expect(canSubmitReaction('navigating')).toBe(false);
    expect(canSubmitReaction('revealed')).toBe(false);
    expect(canSubmitReaction('end_of_edition')).toBe(false);
  });

  // G. aucune réaction n'est enregistrée localement comme réussie avant confirmation serveur.
  test('G. le statut ne passe à revealed qu’après REACT_SUCCESS, jamais avant', () => {
    let state: ReadingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page,
    });
    state = readingReducer(state, { type: 'REACT_START', reaction: 'like', eventId: 'ev-g' });

    // Tant que le serveur n'a pas confirmé, on reste en 'reacting' (jamais 'revealed').
    expect(state.status).toBe('reacting');
    expect(state.metadata).toBeNull();

    // Un 422 ne doit jamais non plus faire passer par 'revealed'.
    const softErrorState = readingReducer(state, resolveReactionErrorAction(readingValidationError()));
    expect(softErrorState.status).not.toBe('revealed');

    // Seule une confirmation serveur explicite marque la réaction comme acquise.
    const confirmed = readingReducer(state, {
      type: 'REACT_SUCCESS',
      metadata: {
        page_id: 'page-001',
        title: 'Du côté de chez Swann',
        author: 'Marcel Proust',
        edition_title: null,
        translator: null,
        publication_year: 1913,
        source_name: null,
      },
    });
    expect(confirmed.status).toBe('revealed');
  });

  test('resolveReactionErrorAction : fallback 422 sans corps JSON structuré est tout de même traité comme récupérable', () => {
    // Cas défensif : si un proxy/CDN intercale une réponse 422 sans le JSON structuré attendu.
    const rawError = new ApiError(422, 'Metriques de lecture invalides ou delai insuffisant');
    expect(resolveReactionErrorAction(rawError).type).toBe('REACT_VALIDATION_RETRY');
  });

  test('resolveReactionErrorAction : erreur générique non-ApiError retombe sur REACT_ERROR', () => {
    const action = resolveReactionErrorAction(new Error('Boom'));
    expect(action).toEqual({ type: 'REACT_ERROR', message: 'Boom' });
  });
});
