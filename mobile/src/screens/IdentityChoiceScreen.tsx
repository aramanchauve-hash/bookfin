import React from 'react';
import { Pressable, StyleSheet, Text, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';

interface IdentityChoiceScreenProps {
  onContinueAnonymous: () => void;
  isCreating?: boolean;
}

/** Account actions deliberately remain absent until an authentication provider
 * and anonymous-to-account merge endpoint are chosen. They are visible rather
 * than disguised: immediate reading is always available without an account. */
export const IdentityChoiceScreen: React.FC<IdentityChoiceScreenProps> = ({ onContinueAnonymous, isCreating = false }) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={styles.content}>
        <Text style={[styles.title, { color: theme.textPrimary }]}>Comment voulez-vous utiliser Bookfin ?</Text>
        <Pressable accessibilityRole="button" accessibilityLabel="Continuer sans compte" disabled={isCreating} onPress={onContinueAnonymous} style={({ pressed }) => [styles.primary, { backgroundColor: theme.accent, opacity: isCreating ? 0.55 : pressed ? 0.84 : 1 }]}>
          <Text style={styles.primaryText}>{isCreating ? 'Ouverture…' : 'Continuer sans compte'}</Text>
        </Pressable>
        <Text style={[styles.futureAction, { color: theme.textMuted }]}>Créer un compte</Text>
        <Text style={[styles.futureAction, { color: theme.textMuted }]}>Se connecter</Text>
        <Text style={[styles.note, { color: theme.textSecondary }]}>La lecture, l’historique et vos langues restent disponibles sur cet appareil.</Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', paddingHorizontal: layout.horizontalPadding },
  content: { width: '100%', maxWidth: layout.maxReadingWidth, alignSelf: 'center', alignItems: 'center' },
  title: { fontFamily: typography.fontFamilySerif, fontSize: 27, lineHeight: 35, textAlign: 'center', marginBottom: 32 },
  primary: { width: '100%', minHeight: layout.minTouchTarget, borderRadius: layout.borderRadius, justifyContent: 'center', alignItems: 'center' },
  primaryText: { color: '#FFFFFF', fontFamily: typography.fontFamilySans, fontSize: 16, fontWeight: '600' },
  futureAction: { marginTop: 18, fontFamily: typography.fontFamilySans, fontSize: 16 },
  note: { marginTop: 34, textAlign: 'center', fontFamily: typography.fontFamilySans, fontSize: 13, lineHeight: 19 },
});
