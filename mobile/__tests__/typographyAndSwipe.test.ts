import fs from 'fs';
import path from 'path';
import {
  HORIZONTAL_TO_VERTICAL_RATIO,
  SWIPE_ANIMATION_DURATION_MS,
  SWIPE_HORIZONTAL_THRESHOLD,
  SWIPE_VELOCITY_THRESHOLD,
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
    test('ReadingContent applique explicitement textAlign: justify et writingDirection: ltr', () => {
      const readingContentSource = fs.readFileSync(
        path.resolve(__dirname, '../src/components/ReadingContent.tsx'),
        'utf-8'
      );

      // Vérifie que le style bodyText contient les directives exactes
      expect(readingContentSource).toMatch(/textAlign:\s*['"]justify['"]/);
      expect(readingContentSource).toMatch(/writingDirection:\s*['"]ltr['"]/);
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
    // Fonctions pures simulant la logique de décision gestuelle de PanResponder
    const shouldSetResponder = (
      enabled: boolean,
      dx: number,
      dy: number
    ): boolean => {
      if (!enabled) return false;
      return dx < -20 && Math.abs(dx) > Math.abs(dy) * HORIZONTAL_TO_VERTICAL_RATIO;
    };

    const isEligibleSwipeRelease = (
      enabled: boolean,
      dx: number,
      vx: number
    ): boolean => {
      if (!enabled) return false;
      return (
        dx <= SWIPE_HORIZONTAL_THRESHOLD ||
        (dx < -20 && vx <= SWIPE_VELOCITY_THRESHOLD)
      );
    };

    test('État READING (avant réaction) : tout swipe horizontal est strictement ignoré', () => {
      const enabled = false; // reading / reacting
      // Même avec un grand geste horizontal gauche, le geste n'est pas capturé
      expect(shouldSetResponder(enabled, -100, 0)).toBe(false);
      expect(isEligibleSwipeRelease(enabled, -100, -0.5)).toBe(false);
    });

    test('État REACTING : tout swipe horizontal reste strictement ignoré', () => {
      const enabled = false;
      expect(shouldSetResponder(enabled, -80, 0)).toBe(false);
    });

    test('État REVEALED : un swipe gauche franc est capturé', () => {
      const enabled = true;
      expect(shouldSetResponder(enabled, -40, 2)).toBe(true);
      expect(isEligibleSwipeRelease(enabled, -60, 0)).toBe(true);
    });

    test('État REVEALED : un flick / swipe rapide par vélocité est capturé', () => {
      const enabled = true;
      expect(shouldSetResponder(enabled, -30, 2)).toBe(true);
      expect(isEligibleSwipeRelease(enabled, -25, -0.4)).toBe(true);
    });

    test('Sanctuaire du scroll vertical : un mouvement vertical ne déclenche JAMAIS le swipe horizontal', () => {
      const enabled = true;

      // Scroll vertical pur vers le haut ou le bas
      expect(shouldSetResponder(enabled, 0, 50)).toBe(false);
      expect(shouldSetResponder(enabled, 0, -50)).toBe(false);

      // Scroll vertical avec légère dérive horizontale
      expect(shouldSetResponder(enabled, -10, 40)).toBe(false);
      expect(shouldSetResponder(enabled, -25, 30)).toBe(false); // ratio 25/30 < 2.5
    });

    test('Swipe vers la droite (page précédente) : ignoré', () => {
      const enabled = true;
      expect(shouldSetResponder(enabled, 60, 0)).toBe(false);
      expect(isEligibleSwipeRelease(enabled, 60, 0.5)).toBe(false);
    });

    test('Mouvement horizontal minime (inférieur au seuil) : ignoré et relâché sans navigation', () => {
      const enabled = true;
      // Moins de 20px
      expect(shouldSetResponder(enabled, -15, 0)).toBe(false);
      // Au-dessus de 20px mais inférieur à 50px sans vélocité suffisante
      expect(isEligibleSwipeRelease(enabled, -30, -0.1)).toBe(false);
    });

    test('Durée de l animation configurée sobrement (160 ms)', () => {
      expect(SWIPE_ANIMATION_DURATION_MS).toBe(160);
      expect(SWIPE_ANIMATION_DURATION_MS).toBeGreaterThanOrEqual(140);
      expect(SWIPE_ANIMATION_DURATION_MS).toBeLessThanOrEqual(180);
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

    test('ReadingScreen connecte SwipeableReadingContainer avec enabled={state.status === "revealed"}', () => {
      const readingScreenSource = fs.readFileSync(
        path.resolve(__dirname, '../src/screens/ReadingScreen.tsx'),
        'utf-8'
      );

      expect(readingScreenSource).toMatch(/SwipeableReadingContainer/);
      expect(readingScreenSource).toMatch(/enabled=\{state\.status\s*===\s*['"]revealed['"]\}/);
      expect(readingScreenSource).toMatch(/onSwipeLeft=\{handleContinueBook\}/);
    });
  });
});
