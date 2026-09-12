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
import { ReactionType } from '../types/api';

interface ReactionToolbarProps {
  onReact: (reaction: ReactionType) => void;
  isReacting: boolean;
  selectedReaction: ReactionType | null;
  disabled?: boolean;
}

/**
 * Barre d'actions de préférence (J'aime / Je n'aime pas) :
 * - Totalement orthogonale à la navigation.
 * - Protection anti-double tap avec état désactivé.
 * - Indicateur de transmission serveur discret.
 */
export const ReactionToolbar: React.FC<ReactionToolbarProps> = ({
  onReact,
  isReacting,
  selectedReaction,
  disabled = false,
}) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;

  const isDisabled = disabled || isReacting;

  return (
    <View style={[styles.container, { borderTopColor: theme.border }]}>
      <View style={styles.buttonGroup}>
        {/* Bouton Je n'aime pas */}
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Je n'aime pas cet extrait"
          disabled={isDisabled}
          onPress={() => onReact('dislike')}
          style={({ pressed }) => [
            styles.button,
            {
              backgroundColor:
                selectedReaction === 'dislike'
                  ? theme.buttonActive
                  : pressed
                  ? theme.buttonActive
                  : theme.buttonBackground,
              borderColor: theme.border,
              opacity: isDisabled && selectedReaction !== 'dislike' ? 0.6 : 1,
            },
          ]}
        >
          {isReacting && selectedReaction === 'dislike' ? (
            <ActivityIndicator size="small" color={theme.textPrimary} />
          ) : (
            <Text
              style={[
                styles.buttonText,
                {
                  color: theme.textSecondary,
                  fontFamily: typography.fontFamilySans,
                },
              ]}
            >
              Je n’aime pas
            </Text>
          )}
        </Pressable>

        {/* Bouton J'aime */}
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="J'aime cet extrait"
          disabled={isDisabled}
          onPress={() => onReact('like')}
          style={({ pressed }) => [
            styles.button,
            {
              backgroundColor:
                selectedReaction === 'like'
                  ? theme.buttonActive
                  : pressed
                  ? theme.buttonActive
                  : theme.buttonBackground,
              borderColor: selectedReaction === 'like' ? theme.accent : theme.border,
              opacity: isDisabled && selectedReaction !== 'like' ? 0.6 : 1,
            },
          ]}
        >
          {isReacting && selectedReaction === 'like' ? (
            <ActivityIndicator size="small" color={theme.accent} />
          ) : (
            <Text
              style={[
                styles.buttonText,
                {
                  color: theme.textPrimary,
                  fontFamily: typography.fontFamilySans,
                  fontWeight: '600',
                },
              ]}
            >
              J’aime
            </Text>
          )}
        </Pressable>
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
  buttonGroup: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 16,
    width: '100%',
    maxWidth: layout.maxReadingWidth,
  },
  button: {
    flex: 1,
    minHeight: layout.minTouchTarget,
    borderRadius: layout.borderRadius,
    borderWidth: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 16,
  },
  buttonText: {
    fontSize: typography.button.fontSize,
  },
});

