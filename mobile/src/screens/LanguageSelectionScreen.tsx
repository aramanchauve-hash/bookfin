import React, { useState } from 'react';
import { Pressable, StyleSheet, Text, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import {
  canConfirmLanguages,
  ReadingLanguage,
  SUPPORTED_READING_LANGUAGES,
} from '../lib/onboarding/languages';

interface LanguageSelectionScreenProps {
  initialLanguages?: readonly ReadingLanguage[];
  onConfirm: (languages: ReadingLanguage[]) => void;
  isSaving?: boolean;
}

export const LanguageSelectionScreen: React.FC<LanguageSelectionScreenProps> = ({
  initialLanguages = [],
  onConfirm,
  isSaving = false,
}) => {
  const [selected, setSelected] = useState<ReadingLanguage[]>([...initialLanguages]);
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const isValid = canConfirmLanguages(selected);

  const toggle = (language: ReadingLanguage) => {
    setSelected((current) =>
      current.includes(language) ? current.filter((item) => item !== language) : [...current, language]
    );
  };

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={styles.content}>
        <Text style={[styles.title, { color: theme.textPrimary }]}>Dans quelles langues souhaitez-vous lire ?</Text>
        <Text style={[styles.description, { color: theme.textSecondary }]}>Vous pouvez en choisir plusieurs.</Text>
        <View style={styles.choices}>
          {SUPPORTED_READING_LANGUAGES.map((language) => {
            const active = selected.includes(language.code);
            return (
              <Pressable
                key={language.code}
                accessibilityRole="checkbox"
                accessibilityState={{ checked: active }}
                accessibilityLabel={language.label}
                onPress={() => toggle(language.code)}
                style={({ pressed }) => [
                  styles.choice,
                  { borderColor: active ? theme.accent : theme.border, backgroundColor: active ? theme.buttonActive : theme.surface, opacity: pressed ? 0.82 : 1 },
                ]}
              >
                <Text style={[styles.choiceText, { color: theme.textPrimary }]}>{language.label}</Text>
                <Text style={[styles.check, { color: active ? theme.accent : theme.textMuted }]}>{active ? '\u2713' : ''}</Text>
              </Pressable>
            );
          })}
        </View>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel="Valider les langues de lecture"
          disabled={!isValid || isSaving}
          onPress={() => onConfirm(selected)}
          style={({ pressed }) => [styles.confirm, { backgroundColor: theme.accent, opacity: !isValid || isSaving ? 0.45 : pressed ? 0.84 : 1 }]}
        >
          <Text style={styles.confirmText}>{isSaving ? 'Enregistrement…' : 'Continuer'}</Text>
        </Pressable>
        {!isValid && <Text style={[styles.validation, { color: theme.textMuted }]}>Choisissez au moins une langue.</Text>}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', paddingHorizontal: layout.horizontalPadding },
  content: { width: '100%', maxWidth: layout.maxReadingWidth, alignSelf: 'center' },
  title: { fontFamily: typography.fontFamilySerif, fontSize: 27, lineHeight: 35, textAlign: 'center' },
  description: { marginTop: 12, fontFamily: typography.fontFamilySans, fontSize: 15, textAlign: 'center' },
  choices: { marginTop: 32, gap: 10 },
  choice: { minHeight: layout.minTouchTarget, borderWidth: 1, borderRadius: layout.borderRadius, paddingHorizontal: 16, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between' },
  choiceText: { fontFamily: typography.fontFamilySans, fontSize: 16 },
  check: { minWidth: 18, fontSize: 17, fontWeight: '700', textAlign: 'right' },
  confirm: { marginTop: 28, minHeight: layout.minTouchTarget, justifyContent: 'center', alignItems: 'center', borderRadius: layout.borderRadius },
  confirmText: { color: '#FFFFFF', fontFamily: typography.fontFamilySans, fontSize: 16, fontWeight: '600' },
  validation: { marginTop: 10, textAlign: 'center', fontFamily: typography.fontFamilySans, fontSize: 13 },
});
