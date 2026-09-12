import {
  initialReadingState,
  readingReducer,
  ReadingState,
} from '../src/state/readingReducer';
import { FeedPageDto, PageRevealDto } from '../src/types/api';

describe('readingReducer - Machine d\'état de lecture', () => {
  const mockPage: FeedPageDto = {
    impression_id: 'impression-1111',
    id: 'impression-1111',
    page_id: 'page-2222',
    page_sequence_number: 42,
    page_number: 42,
    source_page_number: '84',
    text: 'Longtemps je me suis couché de bonne heure.',
    content: 'Longtemps je me suis couché de bonne heure.',
    language_tag: 'fr',
    served_at: '2026-09-12T10:00:00Z',
    token_count: 120,
    continuation_depth: 0,
  };

  const mockMetadata: PageRevealDto = {
    page_id: 'page-2222',
    edition_id: 'edition-3333',
    work_id: 'work-4444',
    title: 'Du côté de chez Swann',
    author: 'Marcel Proust',
    edition_title: 'Édition originale Grasset',
    translator: null,
    publication_year: 1913,
    source_name: 'Bibliothèque Nationale',
  };

  test('État initial : ouverture et animation de page tournée', () => {
    expect(initialReadingState.status).toBe('loading');
    expect(initialReadingState.currentPage).toBeNull();
    expect(initialReadingState.transitionType).toBe('page_flip');
  });

  test('Page reçue : anonymat garanti et passage en état reading', () => {
    const next = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page: mockPage,
    });

    expect(next.status).toBe('reading');
    expect(next.currentPage).toEqual(mockPage);
    expect(next.metadata).toBeNull();
    expect(next.userReaction).toBeNull();

    // Vérification stricte du contrat anonyme
    const pageKeys = Object.keys(next.currentPage!);
    expect(pageKeys).not.toContain('title');
    expect(pageKeys).not.toContain('author');
    expect(pageKeys).not.toContain('work_id');
    expect(pageKeys).not.toContain('edition_title');
  });

  test('Soumission de Like : passage en état reacting et conservation event_id', () => {
    const readingState: ReadingState = {
      ...initialReadingState,
      status: 'reading',
      currentPage: mockPage,
    };

    const reacting = readingReducer(readingState, {
      type: 'REACT_START',
      reaction: 'like',
      eventId: 'event-uuid-999',
    });

    expect(reacting.status).toBe('reacting');
    expect(reacting.userReaction).toBe('like');
    expect(reacting.pendingEventId).toBe('event-uuid-999');
  });

  test('Soumission de Dislike : passage en état reacting', () => {
    const readingState: ReadingState = {
      ...initialReadingState,
      status: 'reading',
      currentPage: mockPage,
    };

    const reacting = readingReducer(readingState, {
      type: 'REACT_START',
      reaction: 'dislike',
      eventId: 'event-uuid-888',
    });

    expect(reacting.status).toBe('reacting');
    expect(reacting.userReaction).toBe('dislike');
  });

  test('Révélation après réaction réussie : passage en état revealed avec titre et auteur', () => {
    const reactingState: ReadingState = {
      ...initialReadingState,
      status: 'reacting',
      currentPage: mockPage,
      userReaction: 'like',
      pendingEventId: 'event-uuid-999',
    };

    const revealed = readingReducer(reactingState, {
      type: 'REACT_SUCCESS',
      metadata: mockMetadata,
    });

    expect(revealed.status).toBe('revealed');
    expect(revealed.metadata).toEqual(mockMetadata);
    expect(revealed.metadata?.title).toBe('Du côté de chez Swann');
    expect(revealed.metadata?.author).toBe('Marcel Proust');
  });

  test('Navigation ContinueBook : transition page_flip sélectionnée', () => {
    const revealedState: ReadingState = {
      ...initialReadingState,
      status: 'revealed',
      currentPage: mockPage,
      metadata: mockMetadata,
      userReaction: 'like',
    };

    const navigating = readingReducer(revealedState, {
      type: 'NAVIGATE_START',
      action: 'continue_book',
    });

    expect(navigating.status).toBe('navigating');
    expect(navigating.lastNavigationAction).toBe('continue_book');
    expect(navigating.transitionType).toBe('page_flip');
  });

  test('Navigation RandomPage : transition fade sélectionnée', () => {
    const revealedState: ReadingState = {
      ...initialReadingState,
      status: 'revealed',
      currentPage: mockPage,
      metadata: mockMetadata,
      userReaction: 'dislike',
    };

    const navigating = readingReducer(revealedState, {
      type: 'NAVIGATE_START',
      action: 'random_page',
    });

    expect(navigating.status).toBe('navigating');
    expect(navigating.lastNavigationAction).toBe('random_page');
    expect(navigating.transitionType).toBe('fade');
  });

  test('EndOfEdition : passage en état end_of_edition avec message explicite', () => {
    const navigatingState: ReadingState = {
      ...initialReadingState,
      status: 'navigating',
      currentPage: mockPage,
    };

    const endState = readingReducer(navigatingState, {
      type: 'END_OF_EDITION',
      message: 'Fin du livre atteinte : aucune page suivante',
    });

    expect(endState.status).toBe('end_of_edition');
    expect(endState.errorMessage).toContain('Fin du livre');
  });
});
