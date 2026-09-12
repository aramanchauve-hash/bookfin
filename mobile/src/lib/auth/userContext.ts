/**
 * Isolation de l'identité utilisateur de développement.
 * Évite la dispersion d'identifiants bruts dans les composants UI
 * et permet un remplacement futur aisé par un vrai gestionnaire de sessions.
 */

export const DEFAULT_DEV_USER_ID = '11111111-1111-1111-1111-111111111111';

let memoryUserId: string | null = null;

export function setCurrentUserId(userId: string | null): void {
  memoryUserId = userId;
}

export function getCurrentUserId(): string {
  if (memoryUserId) {
    return memoryUserId;
  }
  return process.env.EXPO_PUBLIC_DEV_USER_ID || DEFAULT_DEV_USER_ID;
}

export function generateClientRequestId(): string {
  // Générateur UUIDv4 client simple et robuste
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

