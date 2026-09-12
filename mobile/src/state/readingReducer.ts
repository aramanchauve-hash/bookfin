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
  transitionType: 'page_flip' | 'fade' | 'none';
}

export type ReadingAction =
  | { type: 'FETCH_START'; transitionType?: 'page_flip' | 'fade' }
  | { type: 'FETCH_SUCCESS'; page: FeedPageDto }
  | { type: 'FETCH_ERROR'; message: string }
  | { type: 'REACT_START'; reaction: ReactionType; eventId: string }
  | { type: 'REACT_SUCCESS'; metadata: PageRevealDto }
  | { type: 'REACT_ERROR'; message: string }
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
  transitionType: 'page_flip',
};

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
      };

    case 'REACT_SUCCESS':
      return {
        ...state,
        status: 'revealed',
        metadata: action.metadata,
        errorMessage: null,
      };

    case 'REACT_ERROR':
      return {
        ...state,
        status: 'error',
        errorMessage: action.message,
      };

    case 'NAVIGATE_START':
      return {
        ...state,
        status: 'navigating',
        lastNavigationAction: action.action,
        transitionType: action.action === 'continue_book' ? 'page_flip' : 'fade',
        errorMessage: null,
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

