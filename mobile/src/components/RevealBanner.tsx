import React from 'react';
import { StyleSheet, Text, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import { PageRevealDto } from '../types/api';

interface RevealBannerProps {
  metadata: PageRevealDto;
}

/**
 * Révélation sobre post-réaction :
 * - Affiche avec retenue le titre de l'œuvre et le nom de l'auteur.
 * - Mentionne le traducteur et l'édition si disponibles dans le corpus.
 */
export const RevealBanner: React.FC<RevealBannerProps> = ({ metadata }) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;

  return (
    <View
      style={[
        styles.container,
        {
          backgroundColor: theme.surface,
          borderColor: theme.border,
        },
      ]}
      accessibilityRole="summary"
      accessibilityLabel={`Livre révélé : ${metadata.title} par ${metadata.author}`}
    >
      <Text
        style={[
          styles.title,
          {
            color: theme.textPrimary,
            fontFamily: typography.fontFamilySerif,
          },
        ]}
      >
        {metadata.title}
      </Text>

      <Text
        style={[
          styles.author,
          {
            color: theme.accent,
            fontFamily: typography.fontFamilySans,
          },
        ]}
      >
        {metadata.author}
      </Text>

      {/* Détails éditoriaux secondaires si renseignés */}
      {(metadata.translator || metadata.edition_title) && (
        <Text style={[styles.details, { color: theme.textMuted }]}>
          {[
            metadata.translator ? `Trad. ${metadata.translator}` : null,
            metadata.edition_title || null,
          ]
            .filter(Boolean)
            .join(' · ')}
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    width: '100%',
    maxWidth: layout.maxReadingWidth,
    paddingVertical: 14,
    paddingHorizontal: 20,
    borderRadius: layout.borderRadius,
    borderWidth: 1,
    alignItems: 'center',
    marginVertical: 12,
  },
  title: {
    fontSize: typography.title.fontSize,
    lineHeight: typography.title.lineHeight,
    textAlign: 'center',
    marginBottom: 4,
  },
  author: {
    fontSize: typography.author.fontSize,
    textAlign: 'center',
    fontWeight: '500',
    marginBottom: 2,
  },
  details: {
    fontSize: 12,
    textAlign: 'center',
    marginTop: 4,
  },
});

