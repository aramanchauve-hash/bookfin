import { initialReadingState, readingReducer } from '../src/state/readingReducer';
import { FeedPageDto } from '../src/types/api';

describe('Lecture suivie et Bibliothèque', () => {
  const dummyRandomPage: FeedPageDto = {
    impression_id: 'imp-random-1',
    page_id: 'p-random-1',
    page_sequence_number: 42,
    source_page_number: '42',
    text: 'Extrait au hasard découvert dans le feed...',
    language_tag: 'fr',
    served_at: '2026-09-21T20:00:00Z',
    token_count: 120,
  };

  const dummyFirstPageOfWork: FeedPageDto = {
    impression_id: 'imp-seq-1',
    page_id: 'p-seq-1',
    page_sequence_number: 1,
    source_page_number: '1',
    text: 'Il était une fois au commencement...',
    language_tag: 'fr',
    served_at: '2026-09-21T20:01:00Z',
    token_count: 150,
  };

  test('ENTER_SEQUENTIAL_MODE bascule en lecture suivie sans toucher aux invariants', () => {
    const stateWithRandom = {
      ...initialReadingState,
      status: 'revealed' as const,
      currentPage: dummyRandomPage,
      metadata: {
        page_id: 'p-random-1',
        title: 'Candide',
        author: 'Voltaire',
        translator: null,
        edition_title: 'Édition originale',
        publication_year: 1759,
        source_name: 'BNF',
        work_id: 'w-candide',
        edition_id: 'ed-candide',
      },
    };

    const nextState = readingReducer(stateWithRandom, {
      type: 'ENTER_SEQUENTIAL_MODE',
      work: {
        workId: 'w-candide',
        editionId: 'ed-candide',
        title: 'Candide',
        author: 'Voltaire',
      },
      page: dummyFirstPageOfWork,
    });

    expect(nextState.readingMode).toBe('sequential');
    expect(nextState.activeWork).toEqual({
      workId: 'w-candide',
      editionId: 'ed-candide',
      title: 'Candide',
      author: 'Voltaire',
    });
    expect(nextState.currentPage).toEqual(dummyFirstPageOfWork);
    expect(nextState.status).toBe('reading');
    expect(nextState.metadata).toBeNull();
    expect(nextState.userReaction).toBeNull();
  });

  test('EXIT_SEQUENTIAL_MODE restaure le mode aléatoire et la page sauvegardée', () => {
    const sequentialState = {
      ...initialReadingState,
      status: 'reading' as const,
      readingMode: 'sequential' as const,
      activeWork: {
        workId: 'w-candide',
        editionId: 'ed-candide',
        title: 'Candide',
        author: 'Voltaire',
      },
      currentPage: dummyFirstPageOfWork,
    };

    const exitedState = readingReducer(sequentialState, {
      type: 'EXIT_SEQUENTIAL_MODE',
      page: dummyRandomPage,
    });

    expect(exitedState.readingMode).toBe('random');
    expect(exitedState.activeWork).toBeNull();
    expect(exitedState.currentPage).toEqual(dummyRandomPage);
    expect(exitedState.status).toBe('reading');
  });

  test('EXIT_SEQUENTIAL_MODE restaure le signet révélé avec sa réaction et son scroll', () => {
    const metadata = {
      page_id: 'p-random-1',
      title: 'Candide',
      author: 'Voltaire',
      translator: null,
      edition_title: 'Édition originale',
      publication_year: 1759,
      source_name: 'BNF',
      work_id: 'w-candide',
      edition_id: 'ed-candide',
    };
    const sequentialState = {
      ...initialReadingState,
      status: 'reading' as const,
      readingMode: 'sequential' as const,
      activeWork: { workId: 'w-candide', editionId: 'ed-candide', title: 'Candide', author: 'Voltaire' },
      currentPage: dummyFirstPageOfWork,
    };

    const exitedState = readingReducer(sequentialState, {
      type: 'EXIT_SEQUENTIAL_MODE',
      page: dummyRandomPage,
      metadata: metadata as any,
      reaction: 'like',
      scrollOffset: 320,
    });

    // La page déjà réagie ne doit pas réafficher la barre de réaction (sinon 409).
    expect(exitedState.status).toBe('revealed');
    expect(exitedState.metadata).toEqual(metadata);
    expect(exitedState.userReaction).toBe('like');
    expect(exitedState.restoreScrollOffset).toBe(320);
    expect(exitedState.isBufferedPage).toBe(false);
  });

  test('RESET réinitialise le mode à random', () => {
    const sequentialState = {
      ...initialReadingState,
      readingMode: 'sequential' as const,
      activeWork: {
        workId: 'w-1',
        editionId: 'ed-1',
        title: 'Livre',
        author: 'Auteur',
      },
    };

    const resetState = readingReducer(sequentialState, { type: 'RESET' });
    expect(resetState.readingMode).toBe('random');
    expect(resetState.activeWork).toBeNull();
    expect(resetState.status).toBe('loading');
  });
});
