import React, { useEffect, useState } from 'react';
import { ActivityIndicator, StyleSheet, useColorScheme, View } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ReadingScreen } from './src/screens/ReadingScreen';
import { LanguageSelectionScreen } from './src/screens/LanguageSelectionScreen';
import { IdentityChoiceScreen } from './src/screens/IdentityChoiceScreen';
import { getStoredLanguages, getStoredUserId, setStoredLanguages, setStoredUserId } from './src/lib/storage/secureStore';
import { setCurrentUserId } from './src/lib/auth/userContext';
import { ReadingLanguage } from './src/lib/onboarding/languages';
import { createAnonymousIdentity, syncReadingLanguages } from './src/lib/api/identityApi';

// Expo Go uses a development bundle, so the exact `expo start -c` procedure
// below opens the local corpus lab. The conditional require is eliminated from
// production bundles together with the local literary fixtures.
const CorpusPreviewScreen: React.ComponentType | null = __DEV__
  ? require('./src/screens/CorpusPreviewScreen').CorpusPreviewScreen
  : null;
const showCorpusPreview = __DEV__ && process.env.EXPO_PUBLIC_BOOKFIN_FIXTURE_PREVIEW !== '0';

export default function App() {
  const colorScheme = useColorScheme();
  const [isInitializing, setIsInitializing] = useState(true);
  const [userId, setUserId] = useState<string | null>(null);
  const [languages, setLanguages] = useState<ReadingLanguage[] | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isEditingLanguages, setIsEditingLanguages] = useState(false);

  useEffect(() => {
    async function restoreReader() {
      try {
        const [storedId, storedLanguages] = await Promise.all([getStoredUserId(), getStoredLanguages()]);
        if (storedId) setCurrentUserId(storedId);
        setUserId(storedId);
        setLanguages(storedLanguages as ReadingLanguage[] | null);
      } finally {
        setIsInitializing(false);
      }
    }
    restoreReader();
  }, []);

  const saveLanguages = async (chosen: ReadingLanguage[]) => {
    setIsSaving(true);
    try {
      // Persist first: an interrupted network request must never cause the app
      // to ask the reader again. Sync follows as soon as an identity exists.
      await setStoredLanguages(chosen);
      setLanguages(chosen);
      if (userId) await syncReadingLanguages(userId, chosen);
    } finally {
      setIsSaving(false);
    }
  };

  const createAnonymousReader = async () => {
    if (!languages) return;
    setIsSaving(true);
    try {
      const identity = await createAnonymousIdentity();
      await setStoredUserId(identity.user_id);
      await syncReadingLanguages(identity.user_id, languages);
      setCurrentUserId(identity.user_id);
      setUserId(identity.user_id);
    } finally {
      setIsSaving(false);
    }
  };

  if (isInitializing) {
    return (
      <View style={styles.splashContainer}>
        <ActivityIndicator size="small" color="#736F66" />
      </View>
    );
  }

  return (
    <SafeAreaProvider>
      <StatusBar style={colorScheme === 'dark' ? 'light' : 'dark'} />
      {showCorpusPreview && CorpusPreviewScreen ? <CorpusPreviewScreen /> : !languages || isEditingLanguages ? <LanguageSelectionScreen initialLanguages={languages || []} onConfirm={async (chosen) => {
        await saveLanguages(chosen);
        setIsEditingLanguages(false);
      }} isSaving={isSaving} /> :
        !userId ? <IdentityChoiceScreen onContinueAnonymous={createAnonymousReader} isCreating={isSaving} /> :
          <ReadingScreen onOpenLanguageSettings={() => setIsEditingLanguages(true)} />}
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  splashContainer: {
    flex: 1,
    backgroundColor: '#F8F6F0',
    alignItems: 'center',
    justifyContent: 'center',
  },
});
