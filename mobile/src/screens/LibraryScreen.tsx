import React, { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  FlatList,
  Pressable,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { layout, palette, typography } from '../lib/theme/typography';
import { UserLibraryItemDto } from '../types/api';
import { fetchUserLibrary } from '../lib/api/readingApi';
import { useI18n } from '../i18n';

export interface LibraryScreenProps {
  onBack: () => void;
  onSelectWork: (item: UserLibraryItemDto) => void;
  initialItems?: UserLibraryItemDto[];
}

export const LibraryScreen: React.FC<LibraryScreenProps> = ({
  onBack,
  onSelectWork,
  initialItems,
}) => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const { t, format } = useI18n();

  const [items, setItems] = useState<UserLibraryItemDto[]>(initialItems || []);
  const [isLoading, setIsLoading] = useState(!initialItems);
  const [error, setError] = useState<string | null>(null);

  const loadLibrary = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await fetchUserLibrary();
      setItems(data);
    } catch {
      setError(t.common.error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (!initialItems) {
      loadLibrary();
    }
  }, []);

  const renderItem = ({ item }: { item: UserLibraryItemDto }) => {
    const progressText =
      item.total_pages > 0
        ? format(t.reading.page_progress, {
            current: item.current_page_sequence_number || 1,
            total: item.total_pages,
          })
        : format(t.reading.page_number_single, {
            current: item.current_page_sequence_number || 1,
          });

    return (
      <View
        style={[
          styles.itemCard,
          {
            backgroundColor: theme.surface,
            borderColor: theme.border,
          },
        ]}
      >
        <View style={styles.itemInfo}>
          <Text
            style={[
              styles.itemTitle,
              { color: theme.textPrimary, fontFamily: typography.fontFamilySerif },
            ]}
            numberOfLines={2}
          >
            {item.title}
          </Text>
          <Text
            style={[
              styles.itemAuthor,
              { color: theme.accent, fontFamily: typography.fontFamilySans },
            ]}
            numberOfLines={1}
          >
            {item.author}
          </Text>
          <Text
            style={[
              styles.itemProgress,
              { color: theme.textMuted, fontFamily: typography.fontFamilySans },
            ]}
          >
            {progressText}
          </Text>
        </View>

        <Pressable
          accessibilityRole="button"
          accessibilityLabel={`${t.library.resume} ${item.title}`}
          onPress={() => onSelectWork(item)}
          style={({ pressed }) => [
            styles.resumeButton,
            {
              backgroundColor: theme.accent,
              opacity: pressed ? 0.85 : 1,
            },
          ]}
        >
          <Text style={styles.resumeButtonText}>{t.library.resume}</Text>
        </Pressable>
      </View>
    );
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
          {t.library.title}
        </Text>
        <View style={styles.headerSpacer} />
      </View>

      {/* Body */}
      {isLoading ? (
        <View style={styles.centerBox}>
          <ActivityIndicator size="small" color={theme.textMuted} />
        </View>
      ) : error ? (
        <View style={styles.centerBox}>
          <Text
            style={[
              styles.errorText,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {error}
          </Text>
          <Pressable
            accessibilityRole="button"
            onPress={loadLibrary}
            style={[styles.retryButton, { borderColor: theme.border }]}
          >
            <Text
              style={[
                styles.retryText,
                { color: theme.accent, fontFamily: typography.fontFamilySans },
              ]}
            >
              {t.common.retry}
            </Text>
          </Pressable>
        </View>
      ) : items.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Text
            style={[
              styles.emptyTitle,
              { color: theme.textPrimary, fontFamily: typography.fontFamilySerif },
            ]}
          >
            {t.library.empty_title}
          </Text>
          <Text
            style={[
              styles.emptyDesc,
              { color: theme.textSecondary, fontFamily: typography.fontFamilySans },
            ]}
          >
            {t.library.empty_desc}
          </Text>
        </View>
      ) : (
        <FlatList
          data={items}
          keyExtractor={(item) => item.work_id}
          renderItem={renderItem}
          contentContainerStyle={styles.listContent}
          showsVerticalScrollIndicator={false}
        />
      )}
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
  centerBox: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: layout.horizontalPadding,
    gap: 16,
  },
  errorText: {
    fontSize: 15,
    textAlign: 'center',
  },
  retryButton: {
    borderWidth: 1,
    borderRadius: layout.borderRadius,
    paddingHorizontal: 20,
    paddingVertical: 10,
  },
  retryText: {
    fontSize: 15,
    fontWeight: '500',
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 32,
    maxWidth: layout.maxReadingWidth,
    alignSelf: 'center',
  },
  emptyTitle: {
    fontSize: 22,
    lineHeight: 28,
    textAlign: 'center',
    marginBottom: 10,
  },
  emptyDesc: {
    fontSize: 15,
    lineHeight: 22,
    textAlign: 'center',
    fontFamily: typography.fontFamilySans,
  },
  listContent: {
    paddingHorizontal: layout.horizontalPadding,
    paddingTop: 16,
    paddingBottom: 32,
    maxWidth: layout.maxReadingWidth,
    width: '100%',
    alignSelf: 'center',
    gap: 12,
  },
  itemCard: {
    borderWidth: 1,
    borderRadius: layout.borderRadius,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  itemInfo: {
    flex: 1,
    marginRight: 16,
  },
  itemTitle: {
    fontSize: 18,
    lineHeight: 24,
    marginBottom: 4,
  },
  itemAuthor: {
    fontSize: 14,
    marginBottom: 6,
    fontWeight: '500',
  },
  itemProgress: {
    fontSize: 13,
  },
  resumeButton: {
    minHeight: layout.minTouchTarget,
    paddingHorizontal: 16,
    borderRadius: layout.borderRadius,
    justifyContent: 'center',
    alignItems: 'center',
  },
  resumeButtonText: {
    color: '#FFFFFF',
    fontFamily: typography.fontFamilySans,
    fontSize: 14,
    fontWeight: '600',
  },
});
