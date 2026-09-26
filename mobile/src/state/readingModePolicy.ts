import { PageRevealDto } from '../types/api';
import { ReadingState } from './readingReducer';

/**
 * Règles figées de séparation entre la découverte (feed aléatoire) et la lecture
 * suivie d'une œuvre. Fonctions pures : le hook et l'UI s'y conforment, les tests
 * les verrouillent.
 */

export type SwipeForwardIntent = 'like_and_continue' | 'next_page_only';

/**
 * Swipe gauche en bas de page :
 * - découverte : LIKE + CONTINUE_READING (grammaire historique) ;
 * - lecture suivie : page suivante uniquement. Tourner la page d'un roman n'est
 *   jamais une réaction positive : aucun LIKE implicite.
 */
export function swipeForwardIntent(readingMode: ReadingState['readingMode']): SwipeForwardIntent {
  return readingMode === 'sequential' ? 'next_page_only' : 'like_and_continue';
}

export interface WorkEntryActions {
  /** Action principale proposée sous la révélation, ou aucune. */
  primary: 'start' | 'resume' | null;
  /** « Recommencer depuis le début » : secondaire, toujours après confirmation. */
  canRestart: boolean;
  /** Page de reprise connue du serveur (œuvre déjà commencée). */
  resumePage: number | null;
}

const NO_ACTIONS: WorkEntryActions = { primary: null, canRestart: false, resumePage: null };

/**
 * Actions de lecture suivie proposées après une révélation :
 * - œuvre jamais commencée : « Commencer depuis le début » (START_WORK → page 1) ;
 * - œuvre déjà commencée : « Reprendre » (dernière progression), et « Recommencer »
 *   en secondaire avec confirmation — jamais de réinitialisation silencieuse ;
 * - déjà dans la lecture suivie de cette œuvre : aucune action.
 */
export function resolveWorkEntryActions(
  metadata: PageRevealDto | null,
  readingMode: ReadingState['readingMode'],
  activeWork: ReadingState['activeWork']
): WorkEntryActions {
  if (!metadata?.work_id || !metadata.edition_id) return NO_ACTIONS;
  if (readingMode === 'sequential' && activeWork?.workId === metadata.work_id) return NO_ACTIONS;

  const resumePage = metadata.work_progress_page_sequence_number ?? null;
  if (resumePage !== null) {
    return { primary: 'resume', canRestart: true, resumePage };
  }
  return { primary: 'start', canRestart: false, resumePage: null };
}
