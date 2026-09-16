import React, { useMemo, useState } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, useColorScheme, View } from 'react-native';
import { ReadingContent, ParagraphLayout } from '../components/ReadingContent';
import { curatedPreviewSamples } from '../lib/dev/curatedPreviewFixture';
import { layout, palette, typography } from '../lib/theme/typography';
import { FeedPageDto } from '../types/api';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { countSoftHyphens, renderBookfinPageToHtml } from '../lib/reader/bookfinHtml';
import { POLIPHILI_LICENSE_STATUS } from '../lib/reader/bookfinFont';
import type { ReaderFontMode } from '../lib/reader/bookfinFont';

/** Development-only iPhone typography lab. It has no API client, impression,
 * reaction, or production navigation side effects. */
export const CorpusPreviewScreen: React.FC = () => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const [sampleIndex, setSampleIndex] = useState(0);
  const [paragraphLayout, setParagraphLayout] = useState<ParagraphLayout>('balanced');
  const [showBlockTypes, setShowBlockTypes] = useState(false);
  const [renderer, setRenderer] = useState<'legacy' | 'webview'>('webview');
  const [justify, setJustify] = useState(true);
  const [softHyphens, setSoftHyphens] = useState(false);
  const [frenchMin, setFrenchMin] = useState<3 | 4>(4);
  const [fontMode, setFontMode] = useState<ReaderFontMode>('poliphili');
  const [fontStatus, setFontStatus] = useState<'loaded' | 'fallback' | 'not_installed' | 'italic_missing'>('not_installed');
  const [fontDetail, setFontDetail] = useState<{ roman: boolean; italic: boolean } | null>(null);
  const sample = curatedPreviewSamples[sampleIndex];
  const page = useMemo<FeedPageDto>(() => ({
    impression_id: `dev-${sample.id}`,
    page_id: `dev-${sample.id}`,
    page_sequence_number: sample.pageSequenceNumber,
    source_page_number: String(sample.pageSequenceNumber),
    text: '',
    language_tag: sample.language,
    served_at: 'development-preview',
    token_count: 0,
    blocks: sample.blocks,
  }), [sample]);

  const select = (index: number) => setSampleIndex((index + curatedPreviewSamples.length) % curatedPreviewSamples.length);
  const softHyphenCount = useMemo(
    () => softHyphens ? countSoftHyphens(renderBookfinPageToHtml(page, { softHyphens: true, frenchMin })) : 0,
    [page, softHyphens, frenchMin]
  );

  const fontStatusLabel = useMemo(() => {
    if (fontStatus === 'loaded') return 'POLIPHILI ROMAN & ITALIC LOADED';
    if (fontStatus === 'italic_missing') return 'POLIPHILI ROMAN LOADED · POLIPHILI ITALIC NOT AVAILABLE';
    if (fontStatus === 'fallback') return 'POLIPHILI FALLBACK';
    return POLIPHILI_LICENSE_STATUS;
  }, [fontStatus]);

  return (
    <View style={[styles.container, { backgroundColor: theme.background, paddingTop: insets.top, paddingBottom: insets.bottom }]}>
      <View style={[styles.header, { borderBottomColor: theme.border }]}>
        <Text style={[styles.kicker, { color: theme.textMuted }]}>CORPUS V2 · DEV ONLY</Text>
        <Text style={[styles.title, { color: theme.textPrimary }]} numberOfLines={1}>{sample.title}</Text>
        <Text style={[styles.meta, { color: theme.textSecondary }]}>{sample.author} · {sample.language.toUpperCase()} · p. {sample.pageSequenceNumber}</Text>
        {sample.reviewRequired && <Text style={[styles.review, { color: theme.accent }]}>REVIEW — validation humaine iPhone requise</Text>}
      </View>

      <Text style={[styles.hyphenCount, { color: theme.textMuted }]}>Soft hyphens: {softHyphenCount}</Text>
      <Text style={[styles.fontStatus, { color: fontStatus === 'loaded' ? theme.accent : fontStatus === 'italic_missing' ? '#C27803' : theme.textMuted }]}>
        Font: {fontStatusLabel}
        {fontDetail ? ` [Roman: ${fontDetail.roman ? '✓' : '✗'} · Italic: ${fontDetail.italic ? '✓' : '✗'}]` : ''}
      </Text>

      <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.choices} style={styles.choiceScroller}>
        {curatedPreviewSamples.map((item, index) => (
          <Pressable key={item.id} onPress={() => select(index)} accessibilityRole="button" accessibilityLabel={`Afficher ${item.title}`} style={[styles.choice, { borderColor: index === sampleIndex ? theme.accent : theme.border, backgroundColor: index === sampleIndex ? theme.buttonBackground : 'transparent' }]}>
            <Text numberOfLines={1} style={[styles.choiceText, { color: theme.textPrimary }]}>{item.language.toUpperCase()} · {item.title}</Text>
          </Pressable>
        ))}
      </ScrollView>

      <ReadingContent page={page} onScroll={() => undefined} paragraphLayout={paragraphLayout} showBlockTypes={showBlockTypes} renderer={renderer} webViewJustify={justify} webViewSoftHyphens={softHyphens} webViewFrenchMin={frenchMin} webViewFontMode={fontMode} onWebViewFontStatus={(status) => { setFontStatus(status.status); setFontDetail({ roman: status.romanLoaded, italic: status.italicLoaded }); }} />

      <View style={[styles.controls, { borderTopColor: theme.border }]}>
        <View style={styles.navigationRow}>
          <PreviewButton label="‹ Précédent" onPress={() => select(sampleIndex - 1)} color={theme.textPrimary} border={theme.border} />
          <PreviewButton label="Suivant ›" onPress={() => select(sampleIndex + 1)} color={theme.textPrimary} border={theme.border} />
        </View>
        <View style={styles.optionsRow}>
          <PreviewButton label={paragraphLayout === 'spaced' ? '¶ Espacé' : '¶ Retrait'} onPress={() => setParagraphLayout(paragraphLayout === 'spaced' ? 'indented' : 'spaced')} color={theme.textSecondary} border={theme.border} />
          <PreviewButton label={showBlockTypes ? 'Types: oui' : 'Types: non'} onPress={() => setShowBlockTypes(!showBlockTypes)} color={theme.textSecondary} border={theme.border} />
          <PreviewButton label={softHyphens ? `Césure ${sample.language.toUpperCase()}: patterns` : `Césure ${sample.language.toUpperCase()}: désactivée`} onPress={() => setSoftHyphens(!softHyphens)} color={theme.textSecondary} border={theme.border} />
          <PreviewButton label="BOOK BALANCED" onPress={() => setParagraphLayout('balanced')} color={theme.textSecondary} border={theme.border} />
          {sample.language === 'fr' && <PreviewButton label={`FR ${frenchMin}/${frenchMin}`} onPress={() => setFrenchMin(frenchMin === 4 ? 3 : 4)} color={theme.textSecondary} border={theme.border} />}
          <PreviewButton label={renderer === 'webview' ? 'WebView V1' : 'RN legacy'} onPress={() => setRenderer(renderer === 'webview' ? 'legacy' : 'webview')} color={theme.textSecondary} border={theme.border} />
          <PreviewButton label={fontMode === 'poliphili' ? 'Font: Poliphili' : 'Font: fallback'} onPress={() => setFontMode(fontMode === 'poliphili' ? 'system' : 'poliphili')} color={theme.textSecondary} border={theme.border} />
          <PreviewButton label={justify ? 'Justifié' : 'Aligné'} onPress={() => setJustify(!justify)} color={theme.textSecondary} border={theme.border} />
        </View>
      </View>
    </View>
  );
};

const PreviewButton: React.FC<{ label: string; onPress: () => void; color: string; border: string }> = ({ label, onPress, color, border }) => (
  <Pressable accessibilityRole="button" onPress={onPress} style={[styles.button, { borderColor: border }]}>
    <Text style={[styles.buttonText, { color }]}>{label}</Text>
  </Pressable>
);

const styles = StyleSheet.create({
  container: { flex: 1 },
  header: { alignItems: 'center', borderBottomWidth: StyleSheet.hairlineWidth, paddingHorizontal: layout.horizontalPadding, paddingTop: 10, paddingBottom: 8 },
  kicker: { fontFamily: typography.fontFamilySans, fontSize: 10, letterSpacing: 1.2 },
  title: { marginTop: 3, fontFamily: typography.fontFamilySerif, fontSize: 18, lineHeight: 24 },
  meta: { marginTop: 1, fontFamily: typography.fontFamilySans, fontSize: 12 },
  review: { marginTop: 5, fontFamily: typography.fontFamilySans, fontSize: 11, fontWeight: '700', textAlign: 'center' },
  hyphenCount: { alignSelf: 'center', marginTop: 3, fontFamily: typography.fontFamilySans, fontSize: 11 },
  fontStatus: { alignSelf: 'center', marginTop: 2, fontFamily: typography.fontFamilySans, fontSize: 10, textAlign: 'center' },
  choiceScroller: { flexGrow: 0, maxHeight: 42 },
  choices: { gap: 6, paddingHorizontal: 12, paddingVertical: 7 },
  choice: { maxWidth: 190, borderWidth: StyleSheet.hairlineWidth, borderRadius: 14, paddingHorizontal: 10, paddingVertical: 5 },
  choiceText: { fontFamily: typography.fontFamilySans, fontSize: 11 },
  controls: { borderTopWidth: StyleSheet.hairlineWidth, paddingHorizontal: 16, paddingTop: 7, paddingBottom: 9, gap: 6 },
  navigationRow: { flexDirection: 'row', justifyContent: 'center', gap: 10 },
  optionsRow: { flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'center', gap: 8 },
  button: { minHeight: 36, justifyContent: 'center', borderWidth: StyleSheet.hairlineWidth, borderRadius: 18, paddingHorizontal: 14 },
  buttonText: { fontFamily: typography.fontFamilySans, fontSize: 13 },
});
