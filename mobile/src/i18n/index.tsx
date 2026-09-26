import React, { createContext, useContext, useEffect, useState } from 'react';
import { NativeModules, Platform } from 'react-native';
import { SupportedAppLanguage, Translations } from './types';
import { fr } from './fr';
import { en } from './en';
import { es } from './es';
import { ru } from './ru';
import { zh } from './zh';
import { ja } from './ja';
import { getStoredAppLanguage, setStoredAppLanguage } from '../lib/storage/secureStore';

const TRANSLATIONS: Record<SupportedAppLanguage, Translations> = {
  fr,
  en,
  es,
  ru,
  zh,
  ja,
};

export const SUPPORTED_LANGUAGES_LIST: SupportedAppLanguage[] = ['fr', 'en', 'es', 'ru', 'zh', 'ja'];

/**
 * Détecte la langue du système d'exploitation de l'appareil.
 * Si la langue fait partie des 6 langues prises en charge (fr, en, es, ru, zh, ja),
 * elle est retournée. Sinon, fallback transparent en anglais ('en').
 */
export function detectSystemLanguage(): SupportedAppLanguage {
  try {
    let systemLocale = '';
    if (typeof Intl !== 'undefined' && Intl.DateTimeFormat) {
      systemLocale = Intl.DateTimeFormat().resolvedOptions().locale || '';
    }
    if (!systemLocale) {
      if (Platform.OS === 'ios') {
        systemLocale =
          NativeModules.SettingsManager?.settings?.AppleLocale ||
          NativeModules.SettingsManager?.settings?.AppleLanguages?.[0] ||
          '';
      } else if (Platform.OS === 'android') {
        systemLocale = NativeModules.I18nManager?.localeIdentifier || '';
      }
    }

    if (systemLocale) {
      const code = systemLocale.split(/[-_]/)[0].toLowerCase() as SupportedAppLanguage;
      if (SUPPORTED_LANGUAGES_LIST.includes(code)) {
        return code;
      }
    }
  } catch {
    // Fallback silencieux vers l'anglais
  }
  return 'en';
}

export function formatString(template: string, vars: Record<string, string | number>): string {
  let result = template;
  for (const [key, value] of Object.entries(vars)) {
    result = result.replace(new RegExp(`\\{${key}\\}`, 'g'), String(value));
  }
  return result;
}

interface I18nContextType {
  language: SupportedAppLanguage;
  setLanguage: (lang: SupportedAppLanguage) => Promise<void>;
  t: Translations;
  format: (template: string, vars: Record<string, string | number>) => string;
}

const defaultLanguage: SupportedAppLanguage = 'en';
const I18nContext = createContext<I18nContextType>({
  language: defaultLanguage,
  setLanguage: async () => {},
  t: TRANSLATIONS[defaultLanguage],
  format: formatString,
});

export const I18nProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [language, setLanguageState] = useState<SupportedAppLanguage>(defaultLanguage);

  useEffect(() => {
    async function initLanguage() {
      const stored = await getStoredAppLanguage();
      if (stored && SUPPORTED_LANGUAGES_LIST.includes(stored as SupportedAppLanguage)) {
        setLanguageState(stored as SupportedAppLanguage);
      } else {
        const detected = detectSystemLanguage();
        setLanguageState(detected);
      }
    }
    initLanguage();
  }, []);

  const changeLanguage = async (newLang: SupportedAppLanguage) => {
    setLanguageState(newLang);
    await setStoredAppLanguage(newLang);
  };

  const currentTranslations = TRANSLATIONS[language] || TRANSLATIONS.en;

  return (
    <I18nContext.Provider
      value={{
        language,
        setLanguage: changeLanguage,
        t: currentTranslations,
        format: formatString,
      }}
    >
      {children}
    </I18nContext.Provider>
  );
};

export function useI18n(): I18nContextType {
  return useContext(I18nContext);
}

export { SUPPORTED_APP_LANGUAGES } from './types';
export type { SupportedAppLanguage, Translations, AppLanguageOption } from './types';
