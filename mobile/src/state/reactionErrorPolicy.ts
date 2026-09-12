import { ApiError } from '../lib/api/client';
import { ReadingAction } from './readingReducer';

export const READING_VALIDATION_HINT =
  'Continuez simplement votre lecture quelques instants.';

/**
 * Traduit une erreur de soumission de réaction en action de state machine.
 *
 * Un 422 de validation de lecture (temps/scroll/délai serveur insuffisant) N'EST PAS
 * une erreur réseau fatale : c'est un signal produit attendu ("pas encore assez lu").
 * Il doit renvoyer l'utilisateur dans un état lisible ('reading'), pas dans l'état
 * d'erreur bloquant réservé aux vraies pannes réseau/serveur.
 */
export function resolveReactionErrorAction(err: unknown): ReadingAction {
  if (err instanceof ApiError && err.isReadingValidationError) {
    return { type: 'REACT_VALIDATION_RETRY', message: READING_VALIDATION_HINT };
  }

  const message =
    err instanceof Error ? err.message : 'Échec de l’envoi de la réaction.';
  return { type: 'REACT_ERROR', message };
}
