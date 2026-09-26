import React from 'react';
import {
  Pressable,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { layout, palette, typography } from '../lib/theme/typography';
import { useI18n } from '../i18n';

export interface ProfileScreenProps {
  onClose: () => void;
  onOpenLibrary: () => void;
  onOpenSettings: () => void;
  libraryCount?: number;
}

export const ProfileScreen: React.FC<ProfileScreenProps> = ({
  onClose,
  onOpenLibrary,
  onOpenSettings,
  libraryCount,
}) => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const { t, format } = useI18n();

  const librarySubtitle =
    typeof libraryCount === 'number' && libraryCount > 0
      ? libraryCount === 1
        ? t.library.books_count_one
        : format(t.library.books_count_many, { count: libraryCount })
      : undefined;

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
        <Text
          style={[
            styles.headerTitle,
            { color: theme.textPrimary, fontFamily: typography.fontFamilySerif },
          ]}
        >
          {t.profile.title}
        </Text>
        <Pressable
          accessibilityRole="button"
          accessibilityLabel={t.common.close}
          onPress={onClose}
          hitSlop={12}
          style={styles.closeButton}
        >
          <Text style={[styles.closeText, { color: theme.accent }]}>
            {t.common.close}
          </Text>
        </Pressable>
      </View>

      <View style={styles.content}>
        {/* Menu list */}
        <View
          style={[
            styles.card,
            {
              backgroundColor: theme.surface,
              borderColor: theme.border,
            },
          ]}
        >
          {/* Bibliothèque */}
          <Pressable
            accessibilityRole="button"
            accessibilityLabel={t.profile.library}
            onPress={onOpenLibrary}
            style={({ pressed }) => [
              styles.row,
              { borderBottomColor: theme.border, opacity: pressed ? 0.7 : 1 },
            ]}
          >
            <View style={styles.rowLabelContainer}>
              <Text
                style={[
                  styles.rowLabel,
                  { color: theme.textPrimary, fontFamily: typography.fontFamilySans },
                ]}
              >
                {t.profile.library}
              </Text>
              {librarySubtitle && (
                <Text
                  style={[
                    styles.rowBadge,
                    { color: theme.textMuted, fontFamily: typography.fontFamilySans },
                  ]}
                >
                  {librarySubtitle}
                </Text>
              )}
            </View>
            <Text style={[styles.chevron, { color: theme.textMuted }]}>›</Text>
          </Pressable>

          {/* Réglages */}
          <Pressable
            accessibilityRole="button"
            accessibilityLabel={t.profile.settings}
            onPress={onOpenSettings}
            style={({ pressed }) => [
              styles.row,
              styles.lastRow,
              { opacity: pressed ? 0.7 : 1 },
            ]}
          >
            <Text
              style={[
                styles.rowLabel,
                { color: theme.textPrimary, fontFamily: typography.fontFamilySans },
              ]}
            >
              {t.profile.settings}
            </Text>
            <Text style={[styles.chevron, { color: theme.textMuted }]}>›</Text>
          </Pressable>
        </View>

        {/* Section Compte */}
        <View style={styles.sectionHeader}>
          <Text
            style={[
              styles.sectionTitle,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.profile.account_section}
          </Text>
        </View>

        <View
          style={[
            styles.card,
            styles.accountCard,
            {
              backgroundColor: theme.surface,
              borderColor: theme.border,
            },
          ]}
        >
          <Text
            style={[
              styles.accountStatus,
              { color: theme.textPrimary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.profile.anonymous_reader}
          </Text>
          <Text
            style={[
              styles.accountNote,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.profile.data_stored_locally}
          </Text>

          <View style={[styles.divider, { backgroundColor: theme.border }]} />

          <Text
            style={[
              styles.futureAction,
              { color: theme.textMuted, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.profile.create_account_soon}
          </Text>
          <Text
            style={[
              styles.futureAction,
              { color: theme.textMuted, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.profile.login_soon}
          </Text>
        </View>
      </View>
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
    fontSize: 24,
    fontWeight: '600',
  },
  closeButton: {
    minHeight: layout.minTouchTarget,
    justifyContent: 'center',
  },
  closeText: {
    fontSize: 16,
    fontFamily: typography.fontFamilySans,
    fontWeight: '500',
  },
  content: {
    flex: 1,
    paddingHorizontal: layout.horizontalPadding,
    maxWidth: layout.maxReadingWidth,
    width: '100%',
    alignSelf: 'center',
    paddingTop: 16,
  },
  card: {
    borderWidth: 1,
    borderRadius: layout.borderRadius,
    overflow: 'hidden',
    marginBottom: 24,
  },
  row: {
    minHeight: layout.minTouchTarget,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    borderBottomWidth: StyleSheet.hairlineWidth,
  },
  lastRow: {
    borderBottomWidth: 0,
  },
  rowLabelContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  rowLabel: {
    fontSize: 16,
  },
  rowBadge: {
    fontSize: 13,
  },
  chevron: {
    fontSize: 20,
    fontWeight: '400',
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
  accountCard: {
    padding: 16,
  },
  accountStatus: {
    fontSize: 16,
    fontWeight: '500',
  },
  accountNote: {
    fontSize: 13,
    marginTop: 4,
    lineHeight: 18,
  },
  divider: {
    height: StyleSheet.hairlineWidth,
    marginVertical: 14,
  },
  futureAction: {
    fontSize: 14,
    marginTop: 6,
  },
});
