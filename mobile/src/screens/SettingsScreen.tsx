import React, { useState } from 'react';
import {
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { layout, palette, typography } from '../lib/theme/typography';
import { useI18n } from '../i18n';
import { SUPPORTED_APP_LANGUAGES, SupportedAppLanguage } from '../i18n/types';
import {
  canConfirmLanguages,
  ReadingLanguage,
  SUPPORTED_READING_LANGUAGES,
} from '../lib/onboarding/languages';

export interface SettingsScreenProps {
  onBack: () => void;
  readingLanguages: ReadingLanguage[];
  onUpdateReadingLanguages: (languages: ReadingLanguage[]) => Promise<void>;
}

export const SettingsScreen: React.FC<SettingsScreenProps> = ({
  onBack,
  readingLanguages,
  onUpdateReadingLanguages,
}) => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const { t, language: currentAppLanguage, setLanguage } = useI18n();

  const [selectedReadingLangs, setSelectedReadingLangs] = useState<ReadingLanguage[]>(readingLanguages);
  const [isSavingReading, setIsSavingReading] = useState(false);

  const handleSelectAppLanguage = async (code: SupportedAppLanguage) => {
    if (code !== currentAppLanguage) {
      await setLanguage(code);
    }
  };

  const handleToggleReadingLanguage = async (code: ReadingLanguage) => {
    const next = selectedReadingLangs.includes(code)
      ? selectedReadingLangs.filter((item) => item !== code)
      : [...selectedReadingLangs, code];

    if (!canConfirmLanguages(next)) {
      return; // Do not allow deselecting all
    }

    setSelectedReadingLangs(next);
    setIsSavingReading(true);
    try {
      await onUpdateReadingLanguages(next);
    } finally {
      setIsSavingReading(false);
    }
  };

  return (
    <View
      style={[
        styles.container,
        {
          backgroundColor: theme.background,
          paddingTop: insets.top + 8,
          paddingBottom: insets.bottom + 16,
        },
      ]}
    >
      {/* Header */}
      <View style={styles.header}>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={t.common.back}
          onPress={onBack}
          hitSlop={12}
          style={styles.backButton}
        >
          <Text style={[styles.backText, { color: theme.accent }]}>
            ‹ {t.common.back}
          </Text>
        </Pressable>
        <Text
          style={[
            styles.headerTitle,
            { color: theme.textPrimary, fontFamily: typography.fontFamilySerif },
          ]}
        >
          {t.settings.title}
        </Text>
        <View style={styles.headerSpacer} />
      </View>

      <ScrollView
        contentContainerStyle={styles.scrollContent}
        showsVerticalScrollIndicator={false}
      >
        {/* Section A: Langue de l'application */}
        <View style={styles.sectionHeader}>
          <Text
            style={[
              styles.sectionTitle,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.settings.app_language_section}
          </Text>
        </View>

        <View
          style={[
            styles.card,
            {
              backgroundColor: theme.surface,
              borderColor: theme.border,
            },
          ]}
        >
          {SUPPORTED_APP_LANGUAGES.map((item, index) => {
            const isSelected = currentAppLanguage === item.code;
            const isLast = index === SUPPORTED_APP_LANGUAGES.length - 1;
            return (
              <Pressable
                key={item.code}
                accessibilityRole="radio"
                accessibilityState={{ selected: isSelected }}
                accessibilityLabel={item.nativeLabel}
                onPress={() => handleSelectAppLanguage(item.code)}
                style={({ pressed }) => [
                  styles.row,
                  !isLast && { borderBottomColor: theme.border, borderBottomWidth: StyleSheet.hairlineWidth },
                  { opacity: pressed ? 0.7 : 1 },
                ]}
              >
                <View style={styles.langLabelContainer}>
                  <Text
                    style={[
                      styles.langLabel,
                      { color: theme.textPrimary, fontFamily: typography.fontFamilySans },
                    ]}
                  >
                    {item.nativeLabel}
                  </Text>
                  {item.nativeLabel !== item.label && (
                    <Text
                      style={[
                        styles.langSubLabel,
                        { color: theme.textMuted, fontFamily: typography.fontFamilySans },
                      ]}
                    >
                      ({item.label})
                    </Text>
                  )}
                </View>
                {isSelected && (
                  <Text style={[styles.checkmark, { color: theme.accent }]}>✓</Text>
                )}
              </Pressable>
            );
          })}
        </View>

        {/* Section B: Langues de lecture (Corpus) */}
        <View style={styles.sectionHeader}>
          <Text
            style={[
              styles.sectionTitle,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.settings.reading_languages_section}
          </Text>
          <Text
            style={[
              styles.sectionDesc,
              { color: theme.textMuted, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.settings.reading_languages_desc}
          </Text>
        </View>

        <View
          style={[
            styles.card,
            {
              backgroundColor: theme.surface,
              borderColor: theme.border,
            },
          ]}
        >
          {SUPPORTED_READING_LANGUAGES.map((item, index) => {
            const isSelected = selectedReadingLangs.includes(item.code);
            const isLast = index === SUPPORTED_READING_LANGUAGES.length - 1;
            return (
              <Pressable
                key={item.code}
                accessibilityRole="checkbox"
                accessibilityState={{ checked: isSelected }}
                accessibilityLabel={item.label}
                disabled={isSavingReading}
                onPress={() => handleToggleReadingLanguage(item.code)}
                style={({ pressed }) => [
                  styles.row,
                  !isLast && { borderBottomColor: theme.border, borderBottomWidth: StyleSheet.hairlineWidth },
                  { opacity: pressed ? 0.7 : 1 },
                ]}
              >
                <Text
                  style={[
                    styles.langLabel,
                    { color: theme.textPrimary, fontFamily: typography.fontFamilySans },
                  ]}
                >
                  {item.label}
                </Text>
                <Text style={[styles.checkmark, { color: isSelected ? theme.accent : 'transparent' }]}>
                  {isSelected ? '✓' : ''}
                </Text>
              </Pressable>
            );
          })}
        </View>
      </ScrollView>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: layout.horizontalPadding,
    paddingVertical: 12,
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '600',
  },
  backButton: {
    minHeight: layout.minTouchTarget,
    justifyContent: 'center',
    minWidth: 60,
  },
  backText: {
    fontSize: 16,
    fontFamily: typography.fontFamilySans,
    fontWeight: '500',
  },
  headerSpacer: {
    minWidth: 60,
  },
  scrollContent: {
    paddingHorizontal: layout.horizontalPadding,
    paddingTop: 16,
    paddingBottom: 40,
    maxWidth: layout.maxReadingWidth,
    width: '100%',
    alignSelf: 'center',
  },
  sectionHeader: {
    marginBottom: 8,
    paddingHorizontal: 4,
  },
  sectionTitle: {
    fontSize: 13,
    textTransform: 'uppercase',
    letterSpacing: 0.8,
  },
  sectionDesc: {
    fontSize: 13,
    marginTop: 4,
    lineHeight: 18,
  },
  card: {
    borderWidth: 1,
    borderRadius: layout.borderRadius,
    overflow: 'hidden',
    marginBottom: 28,
  },
  row: {
    minHeight: layout.minTouchTarget,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
  },
  langLabelContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  langLabel: {
    fontSize: 16,
  },
  langSubLabel: {
    fontSize: 14,
  },
  checkmark: {
    fontSize: 17,
    fontWeight: '700',
  },
});
