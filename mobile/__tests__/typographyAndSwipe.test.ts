import fs from 'fs';
import path from 'path';
import {
  isRecognizedLeftSwipe,
  SWIPE_ANIMATION_DURATION_MS,
  SWIPE_DX_THRESHOLD,
} from '../src/components/SwipeableReadingContainer';
import { initialReadingState, readingReducer, ReadingState } from '../src/state/readingReducer';
import { FeedPageDto, PageRevealDto } from '../src/types/api';

describe('Bookfin Polish Mobile - Typographie & Swipe Page Turn', () => {
  const mockPage: FeedPageDto = {
    impression_id: 'imp-swipe-001',
    id: 'imp-swipe-001',
    page_id: 'page-swipe-001',
    page_sequence_number: 1,
    page_number: 1,
    source_page_number: '1',
    text: 'Longtemps, je me suis couché de bonne heure...',
    content: 'Longtemps, je me suis couché de bonne heure...',
    language_tag: 'fr',
    served_at: '2026-09-12T10:00:00Z',
    token_count: 150,
  };

  const mockMetadata: PageRevealDto = {
    page_id: 'page-swipe-001',
    edition_id: 'ed-proust',
    work_id: 'work-recherche',
    title: "Du côté de chez Swann",
    author: 'Marcel Proust',
    edition_title: 'Grasset 1913',
    translator: null,
    publication_year: 1913,
    source_name: 'Bibliothèque Nationale',
  };

  describe('1. Typographie & Justification iOS', () => {
    test('ReadingContent applique la justification aux langues à espaces sans imposer une direction LTR', () => {
      const readingContentSource = fs.readFileSync(
        path.resolve(__dirname, '../src/components/ReadingContent.tsx'),
        'utf-8'
      );

      expect(readingContentSource).toContain('usesJustifiedReadingLayout(page.language_tag)');
      expect(readingContentSource).toContain("? 'justify' : 'left'");
      expect(readingContentSource).not.toMatch(/writingDirection:\s*['"]ltr['"]/);
    });

    test('Aucune modification intempestive des métriques typographiques (fontSize, lineHeight, marges, police)', () => {
      const typographySource = fs.readFileSync(
        path.resolve(__dirname, '../src/lib/theme/typography.ts'),
        'utf-8'
      );

      // fontSize: 18.5, lineHeight: 30, letterSpacing: 0.2
      expect(typographySource).toMatch(/fontSize:\s*18\.5/);
      expect(typographySource).toMatch(/lineHeight:\s*30/);
      expect(typographySource).toMatch(/letterSpacing:\s*0\.2/);
    });

    test('Aucun harness/test screen parasite résiduel dans le code source', () => {
      const appSource = fs.readFileSync(path.resolve(__dirname, '../App.tsx'), 'utf-8');
      expect(appSource).not.toMatch(/HarnessScreen/i);
      expect(appSource).not.toMatch(/Next\s*\(/i);
    });
  });

  describe('2. Gestuelle Swipe Page Turn (SwipeableReadingContainer)', () => {
    // isRecognizedLeftSwipe est la fonction réellement utilisée par le
    // PanResponder (claim ET relâchement) : un seul seuil, pas de suivi du
    // doigt, pas de chemin alternatif basé sur la vélocité.

    test('État REVEALED : un swipe gauche franc est reconnu', () => {
      expect(isRecognizedLeftSwipe(-60, 0)).toBe(true);
      expect(isRecognizedLeftSwipe(-51, 2)).toBe(true);
    });

    test('Sanctuaire du scroll vertical : un mouvement vertical ne déclenche JAMAIS le swipe horizontal', () => {
      // Scroll vertical pur vers le haut ou le bas
      expect(isRecognizedLeftSwipe(0, 50)).toBe(false);
      expect(isRecognizedLeftSwipe(0, -50)).toBe(false);

      // Scroll vertical avec dérive horizontale insuffisamment dominante
      expect(isRecognizedLeftSwipe(-60, 30)).toBe(false); // ratio 60/30 = 2 < 2.5
      expect(isRecognizedLeftSwipe(-10, 40)).toBe(false);
    });

    test('Swipe vers la droite (page précédente) : ignoré', () => {
      expect(isRecognizedLeftSwipe(60, 0)).toBe(false);
    });

    test('Mouvement horizontal sous le seuil (-50) : ignoré, même franchement horizontal', () => {
      expect(isRecognizedLeftSwipe(-30, 0)).toBe(false);
      expect(isRecognizedLeftSwipe(-49, 0)).toBe(false);
    });

    test('Durée de l animation configurée sobrement (160 ms)', () => {
      expect(SWIPE_ANIMATION_DURATION_MS).toBe(160);
      expect(SWIPE_ANIMATION_DURATION_MS).toBeGreaterThanOrEqual(140);
      expect(SWIPE_ANIMATION_DURATION_MS).toBeLessThanOrEqual(180);
    });

    test('Seuil de reconnaissance fixé à -50px', () => {
      expect(SWIPE_DX_THRESHOLD).toBe(-50);
    });
  });

  describe('3. Intégration de la State Machine & ReadingScreen', () => {
    // Prédicat de validation de l activation du swipe selon l état du réducteur
    const isSwipeEnabledInState = (state: ReadingState): boolean =>
      state.status === 'revealed';

    test('Cycle complet : reading (désactivé) -> revealed (activé) -> navigating (désactivé)', () => {
      let state = readingReducer(initialReadingState, {
        type: 'FETCH_SUCCESS',
        page: mockPage,
      });
      expect(state.status).toBe('reading');
      expect(isSwipeEnabledInState(state)).toBe(false);

      // Like
      state = readingReducer(state, {
        type: 'REACT_START',
        reaction: 'like',
        eventId: 'evt-swipe-01',
      });
      expect(state.status).toBe('reacting');
      expect(isSwipeEnabledInState(state)).toBe(false);

      // Reveal
      state = readingReducer(state, {
        type: 'REACT_SUCCESS',
        metadata: mockMetadata,
      });
      expect(state.status).toBe('revealed');
      expect(isSwipeEnabledInState(state)).toBe(true);

      // Déclenchement navigation
      state = readingReducer(state, {
        type: 'NAVIGATE_START',
        action: 'continue_book',
      });
      expect(state.status).toBe('navigating');
      expect(isSwipeEnabledInState(state)).toBe(false);
    });

    test('Bouton aléatoire RANDOM_PAGE préservé parallèlement au swipe', () => {
      let state = readingReducer(initialReadingState, {
        type: 'FETCH_SUCCESS',
        page: mockPage,
      });
      state = readingReducer(state, {
        type: 'REACT_START',
        reaction: 'dislike',
        eventId: 'evt-swipe-02',
      });
      state = readingReducer(state, {
        type: 'REACT_SUCCESS',
        metadata: mockMetadata,
      });

      // L utilisateur peut toujours cliquer sur "Une autre page au hasard"
      state = readingReducer(state, {
        type: 'NAVIGATE_START',
        action: 'random_page',
      });
      expect(state.status).toBe('navigating');
      expect(state.lastNavigationAction).toBe('random_page');
    });

    test('ReadingScreen autorise le swipe gauche au bas de la page et le branche sur l’action composée', () => {
      const readingScreenSource = fs.readFileSync(
        path.resolve(__dirname, '../src/screens/ReadingScreen.tsx'),
        'utf-8'
      );

      expect(readingScreenSource).toMatch(/SwipeableReadingContainer/);
      expect(readingScreenSource).toMatch(/enabledForward=/);
      expect(readingScreenSource).toContain('tracker.bottomReached');
      expect(readingScreenSource).toMatch(/onSwipeLeft=\{handleSwipeLikeContinue\}/);
      expect(readingScreenSource).toMatch(/onSwipeRight=\{handleSwipeBack\}/);
    });
  });
});
