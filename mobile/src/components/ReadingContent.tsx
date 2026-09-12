import React, { useRef } from 'react';
import {
  NativeScrollEvent,
  NativeSyntheticEvent,
  ScrollView,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import { FeedPageDto } from '../types/api';

interface ReadingContentProps {
  page: FeedPageDto;
  onScroll: (event: NativeSyntheticEvent<NativeScrollEvent>) => void;
  onContentSizeChange?: (w: number, h: number) => void;
}

/**
 * Composant de lecture littéraire central :
 * - Rendu plein et fidèle du texte fourni par le backend sans tronquage.
 * - Typographie soignée, marges généreuses et largeur de confort.
 * - Scroll naturel uniquement si nécessaire.
 */
export const ReadingContent: React.FC<ReadingContentProps> = ({
  page,
  onScroll,
  onContentSizeChange,
}) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const scrollViewRef = useRef<ScrollView>(null);

  return (
    <ScrollView
      ref={scrollViewRef}
      style={[styles.scrollView, { backgroundColor: theme.background }]}
      contentContainerStyle={styles.scrollContent}
      onScroll={onScroll}
      scrollEventThrottle={32}
      showsVerticalScrollIndicator={false}
      onContentSizeChange={onContentSizeChange}
      accessibilityRole="text"
      accessibilityLabel="Extrait littéraire"
    >
      <View style={styles.textWrapper}>
        {/* En-tête sobre : numéro d'ordre littéraire */}
          <Text style={[styles.pageIndexText, { color: theme.textMuted }]}>
            {`· ${page.source_page_number || page.page_sequence_number || page.page_number} ·`}
          </Text>

        {/* Corps textuel littéraire */}
        <Text
          style={[
            styles.bodyText,
            {
              color: theme.textPrimary,
              fontFamily: typography.fontFamilySerif,
            },
          ]}
          selectable={true}
        >
          {page.text || page.content}
        </Text>

        {/* Espace respirant au bas du texte avant les boutons */}
        <View style={styles.bottomSpacer} />
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  scrollView: {
    flex: 1,
    width: '100%',
  },
  scrollContent: {
    flexGrow: 1,
    alignItems: 'center',
    paddingHorizontal: layout.horizontalPadding,
    paddingTop: 16,
    paddingBottom: 24,
  },
  textWrapper: {
    width: '100%',
    maxWidth: layout.maxReadingWidth,
  },
  headerIndicator: {
    alignItems: 'center',
    marginBottom: 20,
    paddingTop: 4,
  },
  pageIndexText: {
    fontSize: 13,
    letterSpacing: 1.5,
    fontFamily: typography.fontFamilySans,
    textTransform: 'uppercase',
  },
  bodyText: {
    fontSize: typography.reading.fontSize,
    lineHeight: typography.reading.lineHeight,
    letterSpacing: typography.reading.letterSpacing,
    textAlign: 'left',
  },
  bottomSpacer: {
    height: 32,
  },
});
