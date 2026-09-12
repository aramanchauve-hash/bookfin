import { useCallback, useEffect, useReducer, useRef } from 'react';
import { canSubmitReaction, initialReadingState, readingReducer } from './readingReducer';
import { resolveReactionErrorAction } from './reactionErrorPolicy';
import {
  continueReading,
  fetchNextRandomPage,
  revealPageMetadata,
  submitReaction,
} from '../lib/api/readingApi';
import { ApiError, isDevBuild } from '../lib/api/client';
import { generateClientRequestId, getCurrentUserId } from '../lib/auth/userContext';
import { toServerScrollDepth, useReadingTracker } from '../lib/metrics/useReadingTracker';
import { ReactionType } from '../types/api';

export function useReadingSession() {
  const [state, dispatch] = useReducer(readingReducer, initialReadingState);
  const tracker = useReadingTracker(state.currentPage?.page_id);
  const isActionLockedRef = useRef<boolean>(false);

  // 1. Chargement d'une page aléatoire
  const loadRandomPage = useCallback(
    async (transition: 'page_flip' | 'fade' = 'fade') => {
      if (isActionLockedRef.current) return;
      isActionLockedRef.current = true;
      dispatch({ type: 'FETCH_START', transitionType: transition });

      try {
        const page = await fetchNextRandomPage();
        dispatch({ type: 'FETCH_SUCCESS', page });
      } catch (err: any) {
        dispatch({
          type: 'FETCH_ERROR',
          message: err.message || 'Impossible de charger la page littéraire.',
        });
      } finally {
        isActionLockedRef.current = false;
      }
    },
    []
  );

  // Chargement initial au montage de l'application
  useEffect(() => {
    loadRandomPage('page_flip');
  }, [loadRandomPage]);

  // 2. Soumission de réaction (Like / Dislike) avec métriques locales
  const handleReaction = useCallback(
    async (reaction: ReactionType) => {
      // Verrou anti-double-tap. 'error' est autorisé pour que Réessayer fonctionne
      // réellement après une vraie erreur réseau (voir bug alpha : écran figé).
      if (!canSubmitReaction(state.status) || !state.currentPage || isActionLockedRef.current) {
        return;
      }
      isActionLockedRef.current = true;

      const eventId = state.pendingEventId || generateClientRequestId();
      dispatch({ type: 'REACT_START', reaction, eventId });

      // Snapshot pris au moment de CE clic : chaque tentative (initiale ou après un
      // 422 de validation) reflète le temps actif et le scroll accumulés jusqu'ici,
      // jamais une valeur figée d'un essai précédent.
      const snapshot = tracker.getSnapshot(state.currentPage.token_count);

      try {
        const userId = getCurrentUserId();

        await submitReaction({
          user_id: userId,
          page_id: state.currentPage.page_id,
          impression_id: state.currentPage.impression_id || state.currentPage.id!,
          reaction: reaction,
          reading_time_ms: snapshot.dwell_time_ms,
          scroll_depth: toServerScrollDepth(snapshot.scroll_depth_percent),
          bottom_reached: snapshot.bottom_reached,
          content_overflows: snapshot.content_overflows,
          event_id: eventId,
          served_at: state.currentPage.served_at,
        });

        // Révélation sobre de l'auteur et du titre post-réaction confirmée
        const metadata = await revealPageMetadata(state.currentPage.page_id, userId);
        dispatch({ type: 'REACT_SUCCESS', metadata });
      } catch (err: any) {
        if (isDevBuild() && err instanceof ApiError && err.isReadingValidationError) {
          // Log sanitisé dev/alpha uniquement : aucun token, aucune URL, aucun contenu
          // littéraire, aucune donnée personnelle. Juste les métriques numériques et le motif.
          // eslint-disable-next-line no-console
          console.log('[Bookfin][dev] 422 validation de lecture', {
            reason: err.validationReason,
            reading_time_ms: snapshot.dwell_time_ms,
            scroll_depth_percent: snapshot.scroll_depth_percent,
            bottom_reached: snapshot.bottom_reached,
            content_overflows: snapshot.content_overflows,
          });
        }
        dispatch(resolveReactionErrorAction(err));
      } finally {
        isActionLockedRef.current = false;
      }
    },
    [state.status, state.currentPage, state.pendingEventId, tracker]
  );

  // 3. Navigation : Page suivante dans l'œuvre (CONTINUE_BOOK)
  const handleContinueBook = useCallback(async () => {
    if (state.status !== 'revealed' || !state.currentPage || isActionLockedRef.current) {
      return;
    }
    isActionLockedRef.current = true;
    dispatch({ type: 'NAVIGATE_START', action: 'continue_book' });

    try {
      const parentImpressionId = state.currentPage.impression_id || state.currentPage.id;
      const nextPage = await continueReading(
        state.currentPage.page_id,
        parentImpressionId,
        getCurrentUserId(),
        generateClientRequestId()
      );
      dispatch({ type: 'FETCH_SUCCESS', page: nextPage });
    } catch (err: any) {
      if (err instanceof ApiError && err.isEndOfEdition) {
        dispatch({
          type: 'END_OF_EDITION',
          message: err.payload?.message || 'Fin de cette édition',
        });
      } else {
        dispatch({
          type: 'FETCH_ERROR',
          message: err.message || 'Erreur lors du chargement de la suite.',
        });
      }
    } finally {
      isActionLockedRef.current = false;
    }
  }, [state.status, state.currentPage]);

  // 4. Navigation : Autre page au hasard (RANDOM_PAGE)
  const handleRandomPage = useCallback(async () => {
    if (
      (state.status !== 'revealed' && state.status !== 'end_of_edition') ||
      !state.currentPage ||
      isActionLockedRef.current
    ) {
      return;
    }
    dispatch({ type: 'NAVIGATE_START', action: 'random_page' });
    await loadRandomPage('fade');
  }, [state.status, state.currentPage, loadRandomPage]);

  // 5. Réessai en cas d'erreur
  const handleRetry = useCallback(async () => {
    if (state.pendingEventId && state.userReaction) {
      // Réessai de réaction avec strictement le même event_id pour préserver l'idempotence
      await handleReaction(state.userReaction);
    } else {
      await loadRandomPage('fade');
    }
  }, [state.pendingEventId, state.userReaction, handleReaction, loadRandomPage]);

  return {
    state,
    tracker,
    handleReaction,
    handleContinueBook,
    handleRandomPage,
    handleRetry,
  };
}
