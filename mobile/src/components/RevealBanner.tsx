import React from 'react';
import { Alert, Pressable, StyleSheet, Text, useColorScheme, View } from 'react-native';
import { layout, palette, typography } from '../lib/theme/typography';
import { PageRevealDto } from '../types/api';
import { useI18n } from '../i18n';
import { WorkEntryActions } from '../state/readingModePolicy';

interface RevealBannerProps {
  metadata: PageRevealDto;
  /** Actions de lecture suivie résolues par `resolveWorkEntryActions`. */
  workActions?: WorkEntryActions;
  onStartWork?: () => void;
  onResumeWork?: () => void;
  /** Appelé uniquement après confirmation explicite du lecteur. */
  onRestartWork?: () => void;
  isStartingWork?: boolean;
}

/**
 * Révélation sobre post-réaction :
 * - Affiche avec retenue le titre de l'œuvre et le nom de l'auteur.
 * - Mentionne le traducteur et l'édition si disponibles dans le corpus.
 * - Propose sobrement la lecture suivie : « Commencer » si l'œuvre n'a jamais été
 *   commencée, sinon « Reprendre » ; « Recommencer » reste secondaire et confirmé.
 */
export const RevealBanner: React.FC<RevealBannerProps> = ({
  metadata,
  workActions,
  onStartWork,
  onResumeWork,
  onRestartWork,
  isStartingWork = false,
}) => {
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const { t, format } = useI18n();

  const primary = workActions?.primary ?? null;
  const resumePage = workActions?.resumePage ?? 1;
  const primaryLabel =
    primary === 'resume'
      ? format(t.reading.resume_work, { page: resumePage })
      : t.reading.start_from_beginning;
  const onPrimary = primary === 'resume' ? onResumeWork : onStartWork;

  const confirmRestart = () => {
    if (!onRestartWork) return;
    Alert.alert(
      t.reading.restart_confirm_title,
      format(t.reading.restart_confirm_message, { page: resumePage }),
      [
        { text: t.common.cancel, style: 'cancel' },
        { text: t.reading.restart_confirm_action, style: 'destructive', onPress: onRestartWork },
      ],
      { cancelable: true }
    );
  };

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
      accessibilityLabel={format(t.reading.revealed_accessibility, {
        title: metadata.title,
        author: metadata.author,
      })}
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

      {/* Lecture suivie : action principale discrète (Commencer ou Reprendre) */}
      {primary && onPrimary && (
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={primaryLabel}
          disabled={isStartingWork}
          onPress={onPrimary}
          style={({ pressed }) => [
            styles.startWorkButton,
            {
              borderColor: theme.border,
              backgroundColor: theme.buttonBackground,
              opacity: isStartingWork ? 0.6 : pressed ? 0.8 : 1,
            },
          ]}
        >
          <Text style={[styles.startWorkText, { color: theme.textPrimary }]}>
            {isStartingWork ? t.reading.start_from_beginning_starting : primaryLabel}
          </Text>
        </Pressable>
      )}

      {/* Recommencer : secondaire, jamais sans confirmation */}
      {workActions?.canRestart && onRestartWork && (
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={t.reading.restart_from_beginning}
          disabled={isStartingWork}
          onPress={confirmRestart}
          hitSlop={8}
          style={styles.restartButton}
        >
          <Text style={[styles.restartText, { color: theme.textMuted }]}>
            {t.reading.restart_from_beginning}
          </Text>
        </Pressable>
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
  startWorkButton: {
    marginTop: 12,
    borderWidth: 1,
    borderRadius: layout.borderRadius,
    paddingVertical: 8,
    paddingHorizontal: 16,
    alignItems: 'center',
    justifyContent: 'center',
  },
  startWorkText: {
    fontSize: 13,
    fontWeight: '500',
    fontFamily: typography.fontFamilySans,
    letterSpacing: 0.2,
  },
  restartButton: {
    marginTop: 8,
    alignItems: 'center',
  },
  restartText: {
    fontSize: 12,
    fontFamily: typography.fontFamilySans,
    textDecorationLine: 'underline',
  },
});


