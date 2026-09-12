import React from 'react';
import {
  ActivityIndicator,
  Pressable,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import { NavigationAction } from '../types/api';

interface NavigationToolbarProps {
  onContinue: () => void;
  onRandom: () => void;
  isNavigating: boolean;
  navigatingAction: NavigationAction | null;
  isEndOfEdition?: boolean;
  endOfEditionMessage?: string | null;
}

/**
 * Barre de choix de navigation post-révélation :
 * - "Lire la page suivante" (CONTINUE_BOOK)
 * - "Une autre page au hasard" (RANDOM_PAGE)
 * - Gestion propre de EndOfEdition lorsque l'édition est terminée.
 */
export const NavigationToolbar: React.FC<NavigationToolbarProps> = ({
  onContinue,
  onRandom,
  isNavigating,
  navigatingAction,
  isEndOfEdition = false,
  endOfEditionMessage,
}) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;

  return (
    <View style={[styles.container, { borderTopColor: theme.border }]}>
      <View style={styles.contentWrapper}>
        {isEndOfEdition ? (
          // Fin de l'édition atteinte : invitation claire au tirage aléatoire
          <View style={styles.endOfEditionBlock}>
            <Text style={[styles.endOfEditionText, { color: theme.textSecondary }]}>
              {endOfEditionMessage || 'Fin de cette édition'}
            </Text>
            <Pressable
              accessibilityRole="button"
              accessibilityLabel="Explorer une autre page au hasard"
              disabled={isNavigating}
              onPress={onRandom}
              style={({ pressed }) => [
                styles.primaryButton,
                {
                  backgroundColor: theme.accent,
                  opacity: isNavigating ? 0.7 : pressed ? 0.85 : 1,
                },
              ]}
            >
              {isNavigating ? (
                <ActivityIndicator size="small" color="#FFFFFF" />
              ) : (
                <Text style={[styles.primaryButtonText, { color: '#FFFFFF' }]}>
                  Une autre page au hasard
                </Text>
              )}
            </Pressable>
          </View>
        ) : (
          // Deux options de navigation standard
          <View style={styles.buttonRow}>
            <Pressable
              accessibilityRole="button"
              accessibilityLabel="Lire la page suivante de ce livre"
              disabled={isNavigating}
              onPress={onContinue}
              style={({ pressed }) => [
                styles.primaryButton,
                {
                  backgroundColor: theme.accent,
                  opacity: isNavigating && navigatingAction === 'continue_book' ? 0.7 : pressed ? 0.85 : 1,
                },
              ]}
            >
              {isNavigating && navigatingAction === 'continue_book' ? (
                <ActivityIndicator size="small" color="#FFFFFF" />
              ) : (
                <Text style={[styles.primaryButtonText, { color: '#FFFFFF' }]}>
                  Lire la page suivante →
                </Text>
              )}
            </Pressable>

            <Pressable
              accessibilityRole="button"
              accessibilityLabel="Tirer une autre page au hasard"
              disabled={isNavigating}
              onPress={onRandom}
              style={({ pressed }) => [
                styles.secondaryButton,
                {
                  backgroundColor: theme.buttonBackground,
                  borderColor: theme.border,
                  opacity: isNavigating && navigatingAction === 'random_page' ? 0.7 : pressed ? 0.85 : 1,
                },
              ]}
            >
              {isNavigating && navigatingAction === 'random_page' ? (
                <ActivityIndicator size="small" color={theme.textPrimary} />
              ) : (
                <Text style={[styles.secondaryButtonText, { color: theme.textPrimary }]}>
                  Une autre page au hasard
                </Text>
              )}
            </Pressable>
          </View>
        )}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    width: '100%',
    paddingHorizontal: layout.horizontalPadding,
    paddingVertical: 14,
    borderTopWidth: 1,
    alignItems: 'center',
  },
  contentWrapper: {
    width: '100%',
    maxWidth: layout.maxReadingWidth,
  },
  buttonRow: {
    flexDirection: 'column',
    gap: 10,
    width: '100%',
  },
  primaryButton: {
    minHeight: layout.minTouchTarget,
    borderRadius: layout.borderRadius,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 16,
  },
  primaryButtonText: {
    fontSize: typography.button.fontSize,
    fontWeight: '600',
    fontFamily: typography.fontFamilySans,
  },
  secondaryButton: {
    minHeight: layout.minTouchTarget,
    borderRadius: layout.borderRadius,
    borderWidth: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 16,
  },
  secondaryButtonText: {
    fontSize: typography.button.fontSize,
    fontFamily: typography.fontFamilySans,
  },
  endOfEditionBlock: {
    alignItems: 'center',
    gap: 12,
    width: '100%',
  },
  endOfEditionText: {
    fontSize: 15,
    fontStyle: 'italic',
    textAlign: 'center',
  },
});

