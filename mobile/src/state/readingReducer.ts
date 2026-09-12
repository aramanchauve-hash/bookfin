import { FeedPageDto, NavigationAction, PageRevealDto, ReactionType } from '../types/api';

export type ReadingStateStatus =
  | 'loading'
  | 'reading'
  | 'reacting'
  | 'revealed'
  | 'navigating'
  | 'end_of_edition'
  | 'error';

export interface ReadingState {
  status: ReadingStateStatus;
  currentPage: FeedPageDto | null;
  metadata: PageRevealDto | null;
  userReaction: ReactionType | null;
  pendingEventId: string | null;
  lastNavigationAction: NavigationAction | null;
  errorMessage: string | null;
  /**
   * Message discret affiché en état 'reading' suite à un 422 de validation de lecture
   * (ex: scroll/temps insuffisant). Distinct de errorMessage : ce n'est PAS un état
   * bloquant, la lecture et la réaction restent immédiatement disponibles.
   */
  validationHint: string | null;
  transitionType: 'page_flip' | 'fade' | 'none';
}

export type ReadingAction =
  | { type: 'FETCH_START'; transitionType?: 'page_flip' | 'fade' }
  | { type: 'FETCH_SUCCESS'; page: FeedPageDto }
  | { type: 'FETCH_ERROR'; message: string }
  | { type: 'REACT_START'; reaction: ReactionType; eventId: string }
  | { type: 'REACT_SUCCESS'; metadata: PageRevealDto }
  | { type: 'REACT_ERROR'; message: string }
  | { type: 'REACT_VALIDATION_RETRY'; message: string }
  | { type: 'NAVIGATE_START'; action: NavigationAction }
  | { type: 'END_OF_EDITION'; message?: string }
  | { type: 'RESET' };

export const initialReadingState: ReadingState = {
  status: 'loading',
  currentPage: null,
  metadata: null,
  userReaction: null,
  pendingEventId: null,
  lastNavigationAction: null,
  errorMessage: null,
  validationHint: null,
  transitionType: 'page_flip',
};

/**
 * États depuis lesquels une tentative de réaction (initiale ou réessai) est autorisée.
 * 'error' est inclus pour que le bouton Réessayer d'une vraie erreur réseau
 * fonctionne réellement (voir bug alpha : écran figé après un REACT_ERROR).
 */
export function canSubmitReaction(status: ReadingStateStatus): boolean {
  return status === 'reading' || status === 'error';
}

export function readingReducer(
  state: ReadingState,
  action: ReadingAction
): ReadingState {
  switch (action.type) {
    case 'FETCH_START':
      return {
        ...state,
        status: state.currentPage ? 'navigating' : 'loading',
        transitionType: action.transitionType || 'fade',
        errorMessage: null,
        validationHint: null,
      };

    case 'FETCH_SUCCESS':
      return {
        ...state,
        status: 'reading',
        currentPage: action.page,
        metadata: null,
        userReaction: null,
        pendingEventId: null,
        errorMessage: null,
        validationHint: null,
      };

    case 'FETCH_ERROR':
      return {
        ...state,
        status: 'error',
        errorMessage: action.message,
      };

    case 'REACT_START':
      return {
        ...state,
        status: 'reacting',
        userReaction: action.reaction,
        pendingEventId: action.eventId,
        errorMessage: null,
        validationHint: null,
      };

    case 'REACT_SUCCESS':
      return {
        ...state,
        status: 'revealed',
        metadata: action.metadata,
        errorMessage: null,
        validationHint: null,
      };

    case 'REACT_ERROR':
      return {
        ...state,
        status: 'error',
        errorMessage: action.message,
      };

    case 'REACT_VALIDATION_RETRY':
      // 422 de validation de lecture : ce n'est pas une erreur fatale. On revient
      // immédiatement à un état lisible et non bloquant, sans toucher à la page ni
      // au tracker en cours. Un nouvel event_id sera généré au prochain essai.
      return {
        ...state,
        status: 'reading',
        userReaction: null,
        pendingEventId: null,
        errorMessage: null,
        validationHint: action.message,
      };

    case 'NAVIGATE_START':
      return {
        ...state,
        status: 'navigating',
        lastNavigationAction: action.action,
        transitionType: action.action === 'continue_book' ? 'page_flip' : 'fade',
        errorMessage: null,
        validationHint: null,
      };

    case 'END_OF_EDITION':
      return {
        ...state,
        status: 'end_of_edition',
        errorMessage: action.message || 'Fin de cette édition',
      };

    case 'RESET':
      return initialReadingState;

    default:
      return state;
  }
}

