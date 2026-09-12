import { initialReadingState, readingReducer } from '../src/state/readingReducer';
import { FeedPageDto, PageRevealDto } from '../src/types/api';

describe('Cycle complet de lecture mobile Bookfin', () => {
  const page1: FeedPageDto = {
    impression_id: 'imp-001',
    id: 'imp-001',
    page_id: 'page-001',
    page_sequence_number: 1,
    page_number: 1,
    source_page_number: '1',
    text: 'Dans un village de la Manche, dont je ne veux pas me rappeler le nom...',
    content: 'Dans un village de la Manche, dont je ne veux pas me rappeler le nom...',
    language_tag: 'fr',
    served_at: '2026-09-12T10:00:00Z',
    token_count: 180,
  };

  const page2: FeedPageDto = {
    impression_id: 'imp-002',
    id: 'imp-002',
    page_id: 'page-002',
    page_sequence_number: 2,
    page_number: 2,
    source_page_number: '2',
    text: 'Un gentilhomme de ceux qui ont lance au râtelier, rondache antique...',
    content: 'Un gentilhomme de ceux qui ont lance au râtelier, rondache antique...',
    language_tag: 'fr',
    served_at: '2026-09-12T10:05:00Z',
    token_count: 175,
  };

  const pageRandom: FeedPageDto = {
    impression_id: 'imp-003',
    id: 'imp-003',
    page_id: 'page-099',
    page_sequence_number: 55,
    page_number: 55,
    source_page_number: null,
    text: 'Aujourd’hui, maman est morte. Ou peut-être hier, je ne sais pas.',
    content: 'Aujourd’hui, maman est morte. Ou peut-être hier, je ne sais pas.',
    language_tag: 'fr',
    served_at: '2026-09-12T10:10:00Z',
    token_count: 140,
  };

  const metadataDonQuichotte: PageRevealDto = {
    page_id: 'page-001',
    edition_id: 'edition-cervantes',
    work_id: 'work-don-quichotte',
    title: 'Don Quichotte de la Manche',
    author: 'Miguel de Cervantes',
    edition_title: 'Traduction Louis Viardot',
    translator: 'Louis Viardot',
    publication_year: 1605,
    source_name: 'Bibliothèque Numérique',
  };

  test('Cycle 1 : Ouverture -> Lecture anonyme -> Like -> Reveal -> ContinueBook', () => {
    // 1. Ouverture : état initial
    let state = initialReadingState;
    expect(state.status).toBe('loading');

    // 2. Première page reçue
    state = readingReducer(state, { type: 'FETCH_SUCCESS', page: page1 });
    expect(state.status).toBe('reading');
    expect(state.currentPage?.content).toContain('Dans un village');
    expect((state.currentPage as any).title).toBeUndefined(); // Anonymat garanti

    // 3. Like
    const eventId1 = 'event-like-1';
    state = readingReducer(state, { type: 'REACT_START', reaction: 'like', eventId: eventId1 });
    expect(state.status).toBe('reacting');
    expect(state.userReaction).toBe('like');
    expect(state.pendingEventId).toBe(eventId1);

    // 4. Reveal post-réaction
    state = readingReducer(state, { type: 'REACT_SUCCESS', metadata: metadataDonQuichotte });
    expect(state.status).toBe('revealed');
    expect(state.metadata?.title).toBe('Don Quichotte de la Manche');
    expect(state.metadata?.author).toBe('Miguel de Cervantes');

    // 5. Navigation ContinueBook
    state = readingReducer(state, { type: 'NAVIGATE_START', action: 'continue_book' });
    expect(state.status).toBe('navigating');
    expect(state.transitionType).toBe('page_flip');

    // 6. Page suivante reçue (recommencer le cycle)
    state = readingReducer(state, { type: 'FETCH_SUCCESS', page: page2 });
    expect(state.status).toBe('reading');
    expect(state.currentPage?.page_number).toBe(2);
    expect(state.metadata).toBeNull(); // Re-anonymisé pour la nouvelle page
  });

  test('Cycle 2 : Lecture -> Dislike -> Reveal -> RandomPage', () => {
    let state = readingReducer(initialReadingState, { type: 'FETCH_SUCCESS', page: page1 });

    // Dislike
    state = readingReducer(state, {
      type: 'REACT_START',
      reaction: 'dislike',
      eventId: 'event-dislike-1',
    });
    expect(state.userReaction).toBe('dislike');

    // Reveal
    state = readingReducer(state, { type: 'REACT_SUCCESS', metadata: metadataDonQuichotte });
    expect(state.status).toBe('revealed');

    // Choix : Une autre page au hasard
    state = readingReducer(state, { type: 'NAVIGATE_START', action: 'random_page' });
    expect(state.status).toBe('navigating');
    expect(state.transitionType).toBe('fade');

    // Nouvelle page aléatoire reçue
    state = readingReducer(state, { type: 'FETCH_SUCCESS', page: pageRandom });
    expect(state.status).toBe('reading');
    expect(state.currentPage?.page_number).toBe(55);
  });

  test('Cycle 3 : ContinueBook sur la dernière page -> EndOfEdition -> RandomPage', () => {
    // État révélé sur la dernière page
    let state = readingReducer(initialReadingState, { type: 'FETCH_SUCCESS', page: page2 });
    state = readingReducer(state, { type: 'REACT_START', reaction: 'like', eventId: 'ev-last' });
    state = readingReducer(state, { type: 'REACT_SUCCESS', metadata: metadataDonQuichotte });

    // Tentative de continuer
    state = readingReducer(state, { type: 'NAVIGATE_START', action: 'continue_book' });

    // Réponse backend 404 : End of edition
    state = readingReducer(state, {
      type: 'END_OF_EDITION',
      message: 'Fin du livre atteinte : aucune page suivante disponible dans cette édition',
    });
    expect(state.status).toBe('end_of_edition');
    expect(state.errorMessage).toContain('Fin du livre atteinte');

    // L'utilisateur choisit : Une autre page au hasard
    state = readingReducer(state, { type: 'NAVIGATE_START', action: 'random_page' });
    state = readingReducer(state, { type: 'FETCH_SUCCESS', page: pageRandom });
    expect(state.status).toBe('reading');
    expect(state.currentPage?.id).toBe('imp-003');
  });

  test('Protection anti-double tap et retry idempotent', () => {
    let state = readingReducer(initialReadingState, { type: 'FETCH_SUCCESS', page: page1 });

    // Premier tap sur Like
    const initialEventId = 'ev-retry-123';
    state = readingReducer(state, {
      type: 'REACT_START',
      reaction: 'like',
      eventId: initialEventId,
    });
    expect(state.status).toBe('reacting');

    // Erreur réseau simulée (timeout / coupure wifi)
    state = readingReducer(state, {
      type: 'REACT_ERROR',
      message: 'Délai d’attente dépassé (10000 ms)',
    });
    expect(state.status).toBe('error');
    expect(state.pendingEventId).toBe(initialEventId); // L'eventId est précieusement conservé

    // Réessai : le retry réutilise exactement le même eventId
    state = readingReducer(state, {
      type: 'REACT_START',
      reaction: 'like',
      eventId: state.pendingEventId!,
    });
    expect(state.pendingEventId).toBe(initialEventId);

    // Confirmation serveur
    state = readingReducer(state, { type: 'REACT_SUCCESS', metadata: metadataDonQuichotte });
    expect(state.status).toBe('revealed');
  });
});
