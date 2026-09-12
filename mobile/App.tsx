import React, { useEffect, useState } from 'react';
import { ActivityIndicator, StyleSheet, useColorScheme, View } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { ReadingScreen } from './src/screens/ReadingScreen';
import { AlphaGateScreen } from './src/screens/AlphaGateScreen';
import { getStoredUserId } from './src/lib/storage/secureStore';
import { setCurrentUserId } from './src/lib/auth/userContext';

export default function App() {
  const colorScheme = useColorScheme();
  const [isInitializing, setIsInitializing] = useState(true);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    async function checkExistingAlphaUser() {
      try {
        const storedId = await getStoredUserId();
        if (storedId) {
          setCurrentUserId(storedId);
          setIsAuthenticated(true);
        }
      } finally {
        setIsInitializing(false);
      }
    }
    checkExistingAlphaUser();
  }, []);

  const handleAlphaSuccess = (userId: string) => {
    setCurrentUserId(userId);
    setIsAuthenticated(true);
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
      {isAuthenticated ? (
        <ReadingScreen />
      ) : (
        <AlphaGateScreen onSuccess={handleAlphaSuccess} />
      )}
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
