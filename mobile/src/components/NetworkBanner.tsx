import React from 'react';
import { Pressable, StyleSheet, Text, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';

interface NetworkBannerProps {
  message: string;
  onRetry: () => void;
}

export const NetworkBanner: React.FC<NetworkBannerProps> = ({ message, onRetry }) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;

  return (
    <View style={[styles.banner, { backgroundColor: theme.surface, borderColor: theme.danger }]}>
      <Text style={[styles.text, { color: theme.danger, fontFamily: typography.fontFamilySans }]}>
        {message}
      </Text>
      <Pressable
        accessibilityRole="button"
        accessibilityLabel="Réessayer laction"
        onPress={onRetry}
        style={[styles.retryButton, { backgroundColor: theme.buttonBackground }]}
      >
        <Text style={[styles.retryText, { color: theme.textPrimary }]}>Réessayer</Text>
      </Pressable>
    </View>
  );
};

const styles = StyleSheet.create({
  banner: {
    marginHorizontal: layout.horizontalPadding,
    marginVertical: 10,
    paddingVertical: 10,
    paddingHorizontal: 16,
    borderRadius: layout.borderRadius,
    borderWidth: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 12,
  },
  text: {
    flex: 1,
    fontSize: 13,
  },
  retryButton: {
    paddingVertical: 6,
    paddingHorizontal: 12,
    borderRadius: 6,
    minHeight: 36,
    justifyContent: 'center',
  },
  retryText: {
    fontSize: 13,
    fontWeight: '600',
  },
});

