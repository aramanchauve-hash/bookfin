import * as SecureStore from 'expo-secure-store';

const KEY_USER_ID = 'bookfin_alpha_user_id';
const KEY_CONSENT = 'bookfin_alpha_consent_given';

/**
 * Gestion du stockage sécurisé de l'identité anonyme du lecteur alpha.
 * Le lecteur ne voit jamais cet identifiant ; il est généré lors de la validation
 * de l'invitation et persisté de façon transparente sur l'appareil.
 */

export async function getStoredUserId(): Promise<string | null> {
  try {
    return await SecureStore.getItemAsync(KEY_USER_ID);
  } catch {
    return null;
  }
}

export async function setStoredUserId(userId: string): Promise<void> {
  await SecureStore.setItemAsync(KEY_USER_ID, userId);
}

export async function getStoredConsentGiven(): Promise<boolean> {
  try {
    const val = await SecureStore.getItemAsync(KEY_CONSENT);
    return val === 'true';
  } catch {
    return false;
  }
}

export async function setStoredConsentGiven(given: boolean): Promise<void> {
  await SecureStore.setItemAsync(KEY_CONSENT, given ? 'true' : 'false');
}

export async function clearStoredAlphaIdentity(): Promise<void> {
  try {
    await SecureStore.deleteItemAsync(KEY_USER_ID);
    await SecureStore.deleteItemAsync(KEY_CONSENT);
  } catch {
    // Ignore error on clear
  }
}

