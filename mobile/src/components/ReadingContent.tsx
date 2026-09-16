import React, { useLayoutEffect, useRef } from 'react';
import { NativeScrollEvent, NativeSyntheticEvent, LayoutChangeEvent, ScrollView, StyleSheet, Text, TextStyle, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import { BookfinContentBlock, BookfinTextSpan, FeedPageDto } from '../types/api';
import { ReadingWebView } from './ReadingWebView';
import type { ReaderFontMode } from '../lib/reader/bookfinFont';

export type ParagraphLayout = 'spaced' | 'indented' | 'balanced';

interface ReadingContentProps {
  page: FeedPageDto;
  onScroll: (event: NativeSyntheticEvent<NativeScrollEvent>) => void;
  onContentSizeChange?: (w: number, h: number) => void;
  onContentSizeMeasured?: (w: number, h: number, viewportHeight: number) => void;
  initialScrollOffset?: number;
  paragraphLayout?: ParagraphLayout;
  /** Preview-only diagnostic. Technical labels are never shown to readers. */
  showBlockTypes?: boolean;
  renderer?: 'legacy' | 'webview';
  onWebViewScrollState?: (state: { scrollTop: number; scrollHeight: number; viewportHeight: number; atBottom: boolean }) => void;
  onWebViewSwipeLeft?: (atBottom: boolean) => void;
  onWebViewSwipeRight?: () => void;
  webViewJustify?: boolean;
  webViewSoftHyphens?: boolean;
  webViewFrenchMin?: 3 | 4;
  webViewFontMode?: ReaderFontMode;
  onWebViewFontStatus?: (status: { family: string; requestedMode: ReaderFontMode; romanLoaded: boolean; italicLoaded: boolean; status: 'loaded' | 'fallback' | 'not_installed' | 'italic_missing' }) => void;
}

export interface ScrollToTopTarget { scrollTo: (options: { x: number; y: number; animated: boolean }) => void; }

/** Kept as a small exported seam so the page-reset invariant is tested against the exact action used by ReadingContent. */
export function resetReadingScroll(target: ScrollToTopTarget | null): void {
  target?.scrollTo({ x: 0, y: 0, animated: false });
}

/** React Native justification is useful for EN/FR/ES prose. Verse opts out at block level. */
export function usesJustifiedReadingLayout(languageTag: string): boolean {
  return !['zh', 'ja'].includes(languageTag.toLowerCase());
}

function isSpeakerCue(block: BookfinContentBlock): boolean {
  if (block.type !== 'paragraph' || !block.spans || block.spans.length !== 1) return false;
  const text = block.spans[0].text.trim();
  return text.length >= 2 && text.length <= 48 && /^[A-ZÁÉÍÓÚÜÑ][A-ZÁÉÍÓÚÜÑ .'-]+$/.test(text);
}

function spanStyle(span: BookfinTextSpan): TextStyle {
  return { ...(span.italic ? { fontStyle: 'italic' } : {}), ...(span.bold ? { fontWeight: '700' } : {}) };
}

/** Inline V2 emphasis is deliberately kept as nested native Text, never flattened. */
export const BookfinSpans: React.FC<{ spans?: BookfinTextSpan[] }> = ({ spans = [] }) => (
  <>
    {spans.map((span, index) => (
      <Text key={`${index}-${span.text.slice(0, 16)}`} style={spanStyle(span)}>{span.text}</Text>
    ))}
  </>
);

interface BookfinBlockProps {
  block: BookfinContentBlock;
  justify: boolean;
  paragraphLayout: ParagraphLayout;
  showBlockType: boolean;
  color: string;
  mutedColor: string;
}

/** V2 renderer: blocks remain blocks and inline spans remain nested React Native Text spans. */
export const BookfinBlock: React.FC<BookfinBlockProps> = ({ block, justify, paragraphLayout, showBlockType, color, mutedColor }) => {
  const diagnostic = showBlockType ? <Text style={[styles.blockDiagnostic, { color: mutedColor }]}>{block.type}</Text> : null;

  if (block.type === 'scene_break') {
    return <View style={styles.sceneBreak}>{diagnostic}<Text accessibilityLabel="Séparation de scène" style={[styles.sceneBreakMark, { color }]}>* * *</Text></View>;
  }

  if (block.type === 'heading') {
    return <View style={styles.headingBlock}>{diagnostic}<Text style={[styles.headingText, { color }]}><BookfinSpans spans={block.spans} /></Text></View>;
  }

  if (block.type === 'blockquote') {
    return <View style={styles.blockQuote}>{diagnostic}<Text style={[styles.blockQuoteText, { color, textAlign: justify ? 'justify' : 'left' }]}><BookfinSpans spans={block.spans} /></Text></View>;
  }

  if (block.type === 'verse') {
    return <View style={styles.verseBlock}>{diagnostic}<Text style={[styles.verseText, { color }]}><BookfinSpans spans={block.spans} /></Text></View>;
  }

  const speakerCue = isSpeakerCue(block);
  return (
    <View style={speakerCue ? styles.speakerCueBlock : paragraphLayout === 'indented' ? styles.indentedParagraphBlock : styles.paragraphBlock}>
      {diagnostic}
      <Text style={[styles.bodyText, speakerCue ? styles.speakerCueText : null, { color, textAlign: justify ? 'justify' : 'left' }]}>
        {!speakerCue && paragraphLayout === 'indented' ? '\u2003' : null}
        <BookfinSpans spans={block.spans} />
      </Text>
    </View>
  );
};

/** Literary reading surface. V1 text is a rollout fallback; V2 is never concatenated into `text`. */
export const ReadingContent: React.FC<ReadingContentProps> = ({ page, onScroll, onContentSizeChange, onContentSizeMeasured, initialScrollOffset = 0, paragraphLayout = 'balanced', showBlockTypes = false, renderer, onWebViewScrollState, onWebViewSwipeLeft, onWebViewSwipeRight, webViewJustify, webViewSoftHyphens, webViewFrenchMin, webViewFontMode, onWebViewFontStatus }) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const scrollViewRef = useRef<ScrollView>(null);
  const viewportHeightRef = useRef(0);
  const contentSizeRef = useRef<{ width: number; height: number } | null>(null);
  const justify = usesJustifiedReadingLayout(page.language_tag);

  useLayoutEffect(() => {
    scrollViewRef.current?.scrollTo({ x: 0, y: initialScrollOffset, animated: false });
  }, [page.page_id, initialScrollOffset]);

  const onLayout = (event: LayoutChangeEvent) => {
    viewportHeightRef.current = event.nativeEvent.layout.height;
    if (contentSizeRef.current) onContentSizeMeasured?.(contentSizeRef.current.width, contentSizeRef.current.height, viewportHeightRef.current);
  };
  const handleContentSizeChange = (width: number, height: number) => {
    contentSizeRef.current = { width, height };
    onContentSizeChange?.(width, height);
    onContentSizeMeasured?.(width, height, viewportHeightRef.current);
  };

  const preferredRenderer = renderer ?? (page.blocks?.length ? 'webview' : 'legacy');
  if (preferredRenderer === 'webview' && page.blocks?.length) {
    return <ReadingWebView page={page} options={{ paragraphStyle: paragraphLayout === 'indented' ? 'book' : paragraphLayout === 'balanced' ? 'balanced' : 'screen', justify: webViewJustify ?? justify, softHyphens: webViewSoftHyphens, frenchMin: webViewFrenchMin, fontMode: webViewFontMode, showBlockTypes }} initialScrollOffset={initialScrollOffset} onScrollState={onWebViewScrollState} onSwipeLeft={onWebViewSwipeLeft} onSwipeRight={onWebViewSwipeRight} onFontStatus={onWebViewFontStatus} />;
  }

  return (
    <ScrollView key={page.page_id} ref={scrollViewRef} style={[styles.scrollView, { backgroundColor: theme.background }]} contentContainerStyle={styles.scrollContent} onScroll={onScroll} scrollEventThrottle={32} showsVerticalScrollIndicator={false} onContentSizeChange={handleContentSizeChange} onLayout={onLayout} accessibilityRole="text" accessibilityLabel="Extrait littéraire">
      <View style={styles.textWrapper}>
        <Text style={[styles.pageIndexText, { color: theme.textMuted }]}>{`· ${page.source_page_number || page.page_sequence_number || page.page_number} ·`}</Text>
        {page.blocks?.length ? page.blocks.map((block, index) => (
          <BookfinBlock key={`${page.page_id}-${index}`} block={block} justify={justify} paragraphLayout={paragraphLayout} showBlockType={showBlockTypes} color={theme.textPrimary} mutedColor={theme.textMuted} />
        )) : (
          <Text style={[styles.bodyText, { color: theme.textPrimary, textAlign: justify ? 'justify' : 'left' }]} selectable>{page.text || page.content}</Text>
        )}
        <View style={styles.bottomSpacer} />
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  scrollView: { flex: 1, width: '100%' },
  scrollContent: { flexGrow: 1, alignItems: 'center', paddingHorizontal: layout.horizontalPadding, paddingTop: 16, paddingBottom: 24 },
  textWrapper: { width: '100%', maxWidth: layout.maxReadingWidth },
  pageIndexText: { alignSelf: 'center', marginBottom: 22, fontSize: 13, letterSpacing: 1.5, fontFamily: typography.fontFamilySans, textTransform: 'uppercase' },
  bodyText: { fontSize: typography.reading.fontSize, lineHeight: typography.reading.lineHeight, letterSpacing: typography.reading.letterSpacing, fontFamily: typography.fontFamilySerif },
  paragraphBlock: { marginBottom: 12 },
  indentedParagraphBlock: { marginBottom: 4 },
  speakerCueBlock: { marginTop: 7, marginBottom: 3 },
  speakerCueText: { fontSize: 14, lineHeight: 20, letterSpacing: 0.9, fontFamily: typography.fontFamilySans, fontWeight: '700' },
  headingBlock: { alignItems: 'center', marginTop: 10, marginBottom: 20 },
  headingText: { fontSize: 20, lineHeight: 28, letterSpacing: 0.45, textAlign: 'center', fontFamily: typography.fontFamilySerif, fontWeight: '600' },
  sceneBreak: { alignItems: 'center', marginVertical: 20 },
  sceneBreakMark: { fontSize: 15, letterSpacing: 3, fontFamily: typography.fontFamilySerif },
  blockQuote: { marginVertical: 7, marginLeft: 12, paddingLeft: 14, borderLeftWidth: 2, borderLeftColor: '#B8B0A6' },
  blockQuoteText: { fontSize: typography.reading.fontSize, lineHeight: typography.reading.lineHeight, fontFamily: typography.fontFamilySerif, fontStyle: 'italic' },
  verseBlock: { alignSelf: 'flex-start', marginVertical: 9 },
  verseText: { fontSize: typography.reading.fontSize, lineHeight: typography.reading.lineHeight, fontFamily: typography.fontFamilySerif, textAlign: 'left' },
  blockDiagnostic: { alignSelf: 'flex-start', marginBottom: 3, fontSize: 10, letterSpacing: 0.7, fontFamily: typography.fontFamilySans },
  bottomSpacer: { height: 32 },
});
