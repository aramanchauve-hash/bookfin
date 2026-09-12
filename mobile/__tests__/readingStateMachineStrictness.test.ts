import fs from 'fs';
import path from 'path';
import {
  canSubmitReaction,
  initialReadingState,
  readingReducer,
  ReadingState,
} from '../src/state/readingReducer';
import { resolveReactionErrorAction } from '../src/state/reactionErrorPolicy';
import { ApiError } from '../src/lib/api/client';
import { FeedPageDto, PageRevealDto } from '../src/types/api';

describe('Bookfin UI Alpha - Étanchéité de la State Machine & Rendu Strict', () => {
  const mockPage: FeedPageDto = {
    impression_id: 'imp-alpha-001',
    id: 'imp-alpha-001',
    page_id: 'page-alpha-001',
    page_sequence_number: 12,
    page_number: 12,
    source_page_number: '12',
    text: 'Un jeune géomètre vint de la ville pour dresser une carte du marais...',
    content: 'Un jeune géomètre vint de la ville pour dresser une carte du marais...',
    language_tag: 'fr',
    served_at: '2026-09-12T10:00:00Z',
    token_count: 200,
  };

  const mockMetadata: PageRevealDto = {
    page_id: 'page-alpha-001',
    edition_id: 'ed-001',
    work_id: 'work-001',
    title: 'Le Marais',
    author: 'Auteur Inconnu',
    edition_title: 'Édition originale',
    translator: null,
    publication_year: 1905,
    source_name: 'Bibliothèque Publique',
  };

  // Prédicats de rendu conformes au JSX de ReadingScreen.tsx
  const shouldRenderReactionToolbar = (state: ReadingState): boolean =>
    state.status === 'reading' || state.status === 'reacting';

  const shouldRenderNavigationToolbar = (state: ReadingState): boolean =>
    state.status === 'revealed' ||
    state.status === 'navigating' ||
    state.status === 'end_of_edition';

  const shouldRenderRevealBanner = (state: ReadingState): boolean =>
    (state.status === 'revealed' || state.status === 'end_of_edition') &&
    state.metadata !== null;

  // 1. État READING => Like / Dislike visibles, aucune navigation
  test('1. État reading : Like/Dislike visibles, aucune navigation vers une autre page', () => {
    const state = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page: mockPage,
    });

    expect(state.status).toBe('reading');
    expect(state.currentPage).toEqual(mockPage);
    expect(shouldRenderReactionToolbar(state)).toBe(true);
    expect(canSubmitReaction(state.status)).toBe(true);
    expect(shouldRenderNavigationToolbar(state)).toBe(false);
    expect(shouldRenderRevealBanner(state)).toBe(false);
    expect(state.metadata).toBeNull();
  });

  // 2. État REACTING => double tap protégé, pas de navigation prématurée
  test('2. État reacting : double tap protégé, Like/Dislike en transmission, aucune navigation', () => {
    const readingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page: mockPage,
    });

    const reactingState = readingReducer(readingState, {
      type: 'REACT_START',
      reaction: 'like',
      eventId: 'evt-001',
    });

    expect(reactingState.status).toBe('reacting');
    expect(reactingState.userReaction).toBe('like');
    expect(shouldRenderReactionToolbar(reactingState)).toBe(true);
    expect(canSubmitReaction(reactingState.status)).toBe(false);
    expect(shouldRenderNavigationToolbar(reactingState)).toBe(false);
    expect(shouldRenderRevealBanner(reactingState)).toBe(false);
  });

  // 3. État REVEALED => Navigation visible, Like / Dislike masqués
  test('3. État revealed : auteur/titre révélés, navigation visible, Like/Dislike masqués', () => {
    const readingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page: mockPage,
    });

    const reactingState = readingReducer(readingState, {
      type: 'REACT_START',
      reaction: 'like',
      eventId: 'evt-001',
    });

    const revealedState = readingReducer(reactingState, {
      type: 'REACT_SUCCESS',
      metadata: mockMetadata,
    });

    expect(revealedState.status).toBe('revealed');
    expect(revealedState.metadata).toEqual(mockMetadata);
    expect(shouldRenderRevealBanner(revealedState)).toBe(true);
    expect(shouldRenderNavigationToolbar(revealedState)).toBe(true);
    expect(shouldRenderReactionToolbar(revealedState)).toBe(false);
    expect(canSubmitReaction(revealedState.status)).toBe(false);
  });

  // 4. Erreur 422 => Retour reading + Like / Dislike visibles, aucune navigation
  test('4. Erreur 422 de validation : retour immédiat à reading, Like/Dislike visibles, aucune navigation', () => {
    const readingState = readingReducer(initialReadingState, {
      type: 'FETCH_SUCCESS',
      page: mockPage,
    });

    const reactingState = readingReducer(readingState, {
      type: 'REACT_START',
      reaction: 'dislike',
      eventId: 'evt-002',
    });

    const err422 = new ApiError(422, 'Métriques de lecture insuffisantes', {
      error: 'reading_validation_failed',
      reason: 'insufficient_scroll',
    });

    expect(err422.isReadingValidationError).toBe(true);
    const action = resolveReactionErrorAction(err422);
    expect(action.type).toBe('REACT_VALIDATION_RETRY');

    const recoveredState = readingReducer(reactingState, action);

    expect(recoveredState.status).toBe('reading');
    expect(recoveredState.currentPage).toEqual(mockPage);
    expect(shouldRenderReactionToolbar(recoveredState)).toBe(true);
    expect(canSubmitReaction(recoveredState.status)).toBe(true);
    expect(shouldRenderNavigationToolbar(recoveredState)).toBe(false);
    expect(shouldRenderRevealBanner(recoveredState)).toBe(false);
    expect(recoveredState.metadata).toBeNull();
    expect(recoveredState.userReaction).toBeNull();
    expect(recoveredState.validationHint).toBeTruthy();
  });

  // 5. Aucune navigation visible avant réaction confirmée
  test('5. Invariant d anonymat : aucune navigation vers une autre page avant réaction confirmée', () => {
    const statusesBeforeReaction: ReadingState['status'][] = [
      'loading',
      'reading',
      'reacting',
    ];

    statusesBeforeReaction.forEach((status) => {
      const state: ReadingState = {
        ...initialReadingState,
        status,
        currentPage: mockPage,
      };
      expect(shouldRenderNavigationToolbar(state)).toBe(false);
      expect(shouldRenderRevealBanner(state)).toBe(false);
    });
  });

  // 6. Absence absolue de bouton debug Next dans l UI Alpha
  test('6. Absence absolue d artefact de debug Next dans le code source mobile', () => {
    const mobileSrcDir = path.resolve(__dirname, '../src');
    const appTsxPath = path.resolve(__dirname, '../App.tsx');
    const filesToCheck: string[] = [appTsxPath];

    function collectFiles(dir: string) {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          collectFiles(fullPath);
        } else if (entry.name.endsWith('.tsx') || entry.name.endsWith('.ts')) {
          filesToCheck.push(fullPath);
        }
      }
    }

    collectFiles(mobileSrcDir);

    const forbiddenPatterns = [
      /Next \(/i,
      /Next\(/i,
      /btn-next/i,
      /HarnessScreen/i,
      /styles\.fab\b/i,
      /fabText\b/i,
    ];

    for (const filePath of filesToCheck) {
      const fileContent = fs.readFileSync(filePath, 'utf-8');
      for (const pattern of forbiddenPatterns) {
        const match = fileContent.match(pattern);
        expect(match).toBeNull();
      }
    }
  });

  // 7. Double-tap toujours protégé
  test('7. Double-tap toujours protégé : canSubmitReaction rejette reacting, revealed, navigating, end_of_edition', () => {
    expect(canSubmitReaction('reading')).toBe(true);
    expect(canSubmitReaction('error')).toBe(true);
    expect(canSubmitReaction('reacting')).toBe(false);
    expect(canSubmitReaction('revealed')).toBe(false);
    expect(canSubmitReaction('navigating')).toBe(false);
    expect(canSubmitReaction('end_of_edition')).toBe(false);
    expect(canSubmitReaction('loading')).toBe(false);
  });
});
