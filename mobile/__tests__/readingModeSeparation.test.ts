import fs from 'fs';
import path from 'path';
import { resolveWorkEntryActions, swipeForwardIntent } from '../src/state/readingModePolicy';
import { startWork } from '../src/lib/api/readingApi';
import { ApiError } from '../src/lib/api/client';
import { PageRevealDto } from '../src/types/api';

const mockFetch = jest.fn();
(global as any).fetch = mockFetch;

const read = (rel: string) => fs.readFileSync(path.resolve(__dirname, '..', rel), 'utf-8');

/** Corps d'un useCallback du hook, de sa déclaration à la suivante. */
function hookSection(source: string, name: string): string {
  const start = source.indexOf(`const ${name} = useCallback`);
  expect(start).toBeGreaterThan(-1);
  const next = source.indexOf('useCallback', start + `const ${name} = useCallback`.length);
  return source.slice(start, next === -1 ? undefined : next);
}

const revealed = (extra: Partial<PageRevealDto> = {}): PageRevealDto => ({
  page_id: 'p-103',
  title: 'Niebla',
  author: 'Miguel de Unamuno',
  edition_title: null,
  translator: null,
  publication_year: 1914,
  source_name: null,
  work_id: 'w-niebla',
  edition_id: 'ed-niebla',
  ...extra,
});

describe('Découverte et lecture suivie : deux contextes distincts', () => {
  describe('sémantique du swipe', () => {
    test('découverte : swipe = LIKE + CONTINUE_READING', () => {
      expect(swipeForwardIntent('random')).toBe('like_and_continue');
    });

    test('lecture suivie : swipe = page suivante uniquement', () => {
      expect(swipeForwardIntent('sequential')).toBe('next_page_only');
    });

    test('le swipe en lecture suivie est aiguillé avant toute réaction', () => {
      const source = read('src/state/useReadingSession.ts');
      const swipe = hookSection(source, 'handleSwipeLikeContinue');
      const guard = swipe.indexOf("swipeForwardIntent(state.readingMode) === 'next_page_only'");
      expect(guard).toBeGreaterThan(-1);
      expect(guard).toBeLessThan(swipe.indexOf('await handleSequentialPageTurn()'));
      expect(swipe.indexOf('await handleSequentialPageTurn()')).toBeLessThan(swipe.indexOf('await submitReaction'));
    });

    test('la page suivante en lecture suivie n’émet ni réaction ni révélation', () => {
      const next = hookSection(read('src/state/useReadingSession.ts'), 'handleSequentialPageTurn');
      expect(next).toContain('continueReading(');
      expect(next).not.toContain('submitReaction');
      expect(next).not.toContain("'like'");
      expect(next).not.toContain('revealPageMetadata');
    });

    test('l’écran n’affiche pas les pouces en lecture suivie et y garde la page suivante', () => {
      const screen = read('src/screens/ReadingScreen.tsx');
      expect(screen).toContain("{!isSequential && (state.status === 'reading' || state.status === 'reacting') && (");
      expect(screen).toContain('onContinue={isSequential ? handleSequentialPageTurn : handleContinueBook}');
    });
  });

  describe('Commencer / Reprendre / Recommencer', () => {
    test('œuvre jamais commencée : « Commencer depuis le début » seulement', () => {
      expect(resolveWorkEntryActions(revealed(), 'random', null)).toEqual({
        primary: 'start',
        canRestart: false,
        resumePage: null,
      });
    });

    test('œuvre déjà commencée : « Reprendre » en principal, « Recommencer » en secondaire', () => {
      expect(
        resolveWorkEntryActions(revealed({ work_progress_page_sequence_number: 47 }), 'random', null)
      ).toEqual({ primary: 'resume', canRestart: true, resumePage: 47 });
    });

    test('aucune action pendant la lecture suivie de cette même œuvre', () => {
      const activeWork = { workId: 'w-niebla', editionId: 'ed-niebla', title: 'Niebla', author: 'Unamuno' };
      expect(
        resolveWorkEntryActions(revealed({ work_progress_page_sequence_number: 47 }), 'sequential', activeWork)
      ).toEqual({ primary: null, canRestart: false, resumePage: null });
    });

    test('aucune action sans identifiants d’œuvre (backend antérieur à 0012)', () => {
      expect(resolveWorkEntryActions(revealed({ work_id: undefined }), 'random', null).primary).toBeNull();
      expect(resolveWorkEntryActions(null, 'random', null).primary).toBeNull();
    });

    test('« Recommencer » passe toujours par une confirmation explicite', () => {
      const banner = read('src/components/RevealBanner.tsx');
      expect(banner).toContain('onPress={confirmRestart}');
      expect(banner).toContain('Alert.alert(');
      expect(banner).toContain("onPress: onRestartWork");
      // onRestartWork n'est jamais branché directement sur un bouton.
      expect(banner).not.toContain('onPress={onRestartWork}');
    });

    test('START sur une œuvre déjà commencée reprend au lieu de réinitialiser', () => {
      const start = hookSection(read('src/state/useReadingSession.ts'), 'handleStartWork');
      expect(start).toContain('restart: options.restart === true');
      expect(start).toContain('err.isWorkAlreadyStarted');
      expect(start).toContain('return resumeWork(workId)');
    });
  });

  describe('contrat API de START_WORK', () => {
    beforeEach(() => mockFetch.mockReset());

    const okPage = {
      ok: true,
      status: 200,
      text: async () =>
        JSON.stringify({
          impression_id: 'imp-seq-1',
          page_id: 'p-1',
          page_sequence_number: 1,
          text: 'Primera página',
          language_tag: 'es',
          served_at: '2026-09-26T10:00:00Z',
          token_count: 10,
        }),
    };

    test('un START ordinaire n’envoie jamais de redémarrage', async () => {
      mockFetch.mockResolvedValueOnce(okPage);
      await startWork('w-niebla', 'ed-niebla', {}, 'user-1');
      const [url, init] = mockFetch.mock.calls[0];
      expect(url).toContain('/api/v1/works/w-niebla/start');
      expect(JSON.parse(init.body)).toEqual({ user_id: 'user-1', edition_id: 'ed-niebla', restart: false });
    });

    test('un redémarrage confirmé est explicite', async () => {
      mockFetch.mockResolvedValueOnce(okPage);
      await startWork('w-niebla', 'ed-niebla', { restart: true }, 'user-1');
      expect(JSON.parse(mockFetch.mock.calls[0][1].body).restart).toBe(true);
    });

    test('le 409 work_already_started est reconnu comme tel', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 409,
        text: async () =>
          JSON.stringify({ error: 'work_already_started', current_page_sequence_number: 47, message: 'déjà commencée' }),
      });
      const error = await startWork('w-niebla', 'ed-niebla', {}, 'user-1').catch((e) => e);
      expect(error).toBeInstanceOf(ApiError);
      expect(error.isWorkAlreadyStarted).toBe(true);
      expect(error.payload.current_page_sequence_number).toBe(47);
    });
  });

  describe('hasard et reprise d’erreur', () => {
    test('« page au hasard » depuis la lecture suivie quitte ce contexte', () => {
      const random = hookSection(read('src/state/useReadingSession.ts'), 'handleRandomPage');
      expect(random.indexOf("dispatch({ type: 'EXIT_SEQUENTIAL_MODE' })")).toBeGreaterThan(-1);
      expect(random.indexOf("dispatch({ type: 'EXIT_SEQUENTIAL_MODE' })")).toBeLessThan(
        random.indexOf('await loadRandomPage')
      );
    });

    test('un échec de start/resume se récupère en revenant au signet de découverte', () => {
      const enter = hookSection(read('src/state/useReadingSession.ts'), 'enterWork');
      expect(enter).toContain('retryRef.current = restoreDiscoveryBookmark');
    });
  });
});
