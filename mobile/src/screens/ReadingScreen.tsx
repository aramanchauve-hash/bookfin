import React, { useState } from 'react';
import {
  ActivityIndicator,
  Modal,
  Pressable,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { palette, typography } from '../lib/theme/typography';
import { useReadingSession } from '../state/useReadingSession';
import { PageFlipTransition } from '../components/PageFlipTransition';
import { SwipeableReadingContainer } from '../components/SwipeableReadingContainer';
import { ReadingContent } from '../components/ReadingContent';
import { ReactionToolbar } from '../components/ReactionToolbar';
import { RevealBanner } from '../components/RevealBanner';
import { NavigationToolbar } from '../components/NavigationToolbar';
import { NetworkBanner } from '../components/NetworkBanner';
import { ProfileScreen } from './ProfileScreen';
import { LibraryScreen } from './LibraryScreen';
import { SettingsScreen } from './SettingsScreen';
import { ReadingLanguage } from '../lib/onboarding/languages';
import { useI18n } from '../i18n';
import { resolveWorkEntryActions } from '../state/readingModePolicy';

/**
 * Écran principal unique de Bookfin (ReadingScreen) :
 * - Convergence immédiate vers la lecture dès l'ouverture.
 * - Aucune page intermédiaire, aucun catalogue, aucun profil intrusive.
 * - Anonymat absolu avant réaction.
 * - Accès discret au Profil / Bibliothèque / Réglages.
 * - Support de la lecture suivie et retour à la découverte aléatoire.
 */
export interface ReadingScreenProps {
  onOpenLanguageSettings?: () => void;
  readingLanguages?: ReadingLanguage[];
  onUpdateReadingLanguages?: (languages: ReadingLanguage[]) => Promise<void>;
}

export const ReadingScreen: React.FC<ReadingScreenProps> = ({
  onOpenLanguageSettings,
  readingLanguages = [],
  onUpdateReadingLanguages,
}) => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;
  const { t } = useI18n();

  const [modalView, setModalView] = useState<'none' | 'profile' | 'library' | 'settings'>('none');

  const {
    state,
    tracker,
    handleReaction,
    handleContinueBook,
    handleSwipeLikeContinue,
    handleSwipeBack,
    canSwipeBack,
    canReuseForward,
    handleRandomPage,
    handleRetry,
    handleStartWork,
    handleResumeWork,
    handleReturnToDiscovery,
    handleSequentialPageTurn,
  } = useReadingSession();

  const isSequential = state.readingMode === 'sequential';
  const workActions = resolveWorkEntryActions(state.metadata, state.readingMode, state.activeWork);

  const isInitialLoading = state.status === 'loading' && !state.currentPage;

  return (
    <View
      style={[
        styles.container,
        {
          backgroundColor: theme.background,
          paddingTop: insets.top,
          paddingBottom: insets.bottom,
        },
      ]}
    >
      {/* 1. Barre supérieure discrète (Mode séquentiel info / Profil) */}
      <View style={styles.topBar}>
        {state.readingMode === 'sequential' ? (
          <Pressable
            accessibilityRole="button"
            accessibilityLabel={t.reading.back_to_discovery}
            onPress={handleReturnToDiscovery}
            hitSlop={10}
            style={styles.backToDiscoveryButton}
          >
            <Text style={[styles.backToDiscoveryText, { color: theme.accent }]}>
              {t.reading.back_to_discovery}
            </Text>
          </Pressable>
        ) : (
          <View style={styles.topBarSpacer} />
        )}

        <Pressable
          accessibilityRole="button"
          accessibilityLabel={t.reading.profile_button}
          onPress={() => {
            if (onOpenLanguageSettings && !onUpdateReadingLanguages) {
              onOpenLanguageSettings();
            } else {
              setModalView('profile');
            }
          }}
          hitSlop={10}
          style={styles.profileButton}
        >
          <Text style={[styles.profileText, { color: theme.textMuted }]}>
            {t.reading.profile_button}
          </Text>
        </Pressable>
      </View>

      {/* 2. Écran de chargement sobre initial */}
      {isInitialLoading && (
        <View style={styles.centerBox}>
          <ActivityIndicator size="small" color={theme.textMuted} />
          <Text style={[styles.loadingText, { color: theme.textMuted }]}>
            {t.common.loading}
          </Text>
        </View>
      )}

      {/* 3. Bannière d'erreur réseau discrète si anomalie */}
      {state.status === 'error' && state.errorMessage && (
        <NetworkBanner message={state.errorMessage} onRetry={handleRetry} />
      )}

      {/* 3bis. Indication discrète non bloquante après validation de lecture */}
      {state.status === 'reading' && state.validationHint && (
        <View style={styles.hintContainer}>
          <Text
            style={[
              styles.hintText,
              { color: theme.textMuted, fontFamily: typography.fontFamilySans },
            ]}
          >
            {state.validationHint}
          </Text>
        </View>
      )}

      {/* 4. Contenu de lecture enveloppé dans le conteneur gestuel et la transition de page */}
      {state.currentPage && (
        <SwipeableReadingContainer
          enabledForward={
            (tracker.bottomReached || canReuseForward) &&
            (state.status === 'reading' || (state.isBufferedPage && state.status === 'revealed'))
          }
          enabledBack={
            canSwipeBack &&
            state.status !== 'reacting' &&
            state.status !== 'navigating' &&
            state.status !== 'loading'
          }
          onSwipeLeft={handleSwipeLikeContinue}
          onSwipeRight={handleSwipeBack}
          style={styles.transitionContainer}
        >
          <PageFlipTransition
            triggerKey={state.currentPage.impression_id || state.currentPage.id || state.currentPage.page_id}
            type={state.transitionType}
          >
            <ReadingContent
              page={state.currentPage}
              onScroll={tracker.onScroll}
              onContentSizeMeasured={tracker.onContentSizeChange}
              initialScrollOffset={state.restoreScrollOffset}
              onWebViewScrollState={tracker.onWebViewScrollState}
              onWebViewSwipeLeft={(atBottom) => {
                if (atBottom || canReuseForward) handleSwipeLikeContinue();
              }}
              onWebViewSwipeRight={handleSwipeBack}
            />
          </PageFlipTransition>
        </SwipeableReadingContainer>
      )}

      {/* 5. Révélation sobre de l'auteur et du titre post-réaction avec option de commencer */}
      {(state.status === 'revealed' || state.status === 'end_of_edition') &&
        state.metadata && (
          <View style={styles.bannerContainer}>
            <RevealBanner
              metadata={state.metadata}
              workActions={workActions}
              onStartWork={() =>
                handleStartWork(
                  state.metadata!.work_id!,
                  state.metadata!.edition_id!,
                  state.metadata?.title,
                  state.metadata?.author
                )
              }
              onResumeWork={() =>
                handleResumeWork(
                  state.metadata!.work_id!,
                  state.metadata!.edition_id!,
                  state.metadata?.title,
                  state.metadata?.author
                )
              }
              onRestartWork={() =>
                handleStartWork(
                  state.metadata!.work_id!,
                  state.metadata!.edition_id!,
                  state.metadata?.title,
                  state.metadata?.author,
                  { restart: true }
                )
              }
            />
          </View>
        )}

      {/* 6. Barre de réaction (Préférence : Like / Dislike) — découverte uniquement.
          En lecture suivie, tourner la page n'appelle aucune réaction. */}
      {!isSequential && (state.status === 'reading' || state.status === 'reacting') && (
        <ReactionToolbar
          onReact={handleReaction}
          isReacting={state.status === 'reacting'}
          selectedReaction={state.userReaction}
        />
      )}

      {/* 7. Barre de navigation : post-révélation en découverte, permanente en lecture suivie
          (page suivante sans réaction ; le hasard quitte la lecture suivie). */}
      {(isSequential
        ? state.status === 'reading' ||
          state.status === 'revealed' ||
          state.status === 'navigating' ||
          state.status === 'end_of_edition'
        : !state.isBufferedPage &&
          (state.status === 'revealed' ||
            state.status === 'navigating' ||
            state.status === 'end_of_edition')) && (
        <NavigationToolbar
          onContinue={isSequential ? handleSequentialPageTurn : handleContinueBook}
          onRandom={handleRandomPage}
          isNavigating={state.status === 'navigating'}
          navigatingAction={state.lastNavigationAction}
          isEndOfEdition={state.status === 'end_of_edition'}
          endOfEditionMessage={state.errorMessage}
        />
      )}

      {/* Modals pour Profil, Bibliothèque et Réglages */}
      <Modal
        visible={modalView !== 'none'}
        animationType="slide"
        presentationStyle="pageSheet"
        onRequestClose={() => setModalView('none')}
      >
        {modalView === 'profile' && (
          <ProfileScreen
            onClose={() => setModalView('none')}
            onOpenLibrary={() => setModalView('library')}
            onOpenSettings={() => setModalView('settings')}
          />
        )}
        {modalView === 'library' && (
          <LibraryScreen
            onBack={() => setModalView('profile')}
            onSelectWork={(item) => {
              handleResumeWork(
                item.work_id,
                item.edition_id,
                item.title,
                item.author
              );
              setModalView('none');
            }}
          />
        )}
        {modalView === 'settings' && (
          <SettingsScreen
            onBack={() => setModalView('profile')}
            readingLanguages={readingLanguages}
            onUpdateReadingLanguages={onUpdateReadingLanguages || (async () => {})}
          />
        )}
      </Modal>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  centerBox: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    gap: 12,
  },
  topBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 20,
    paddingTop: 6,
    minHeight: 28,
  },
  topBarSpacer: {
    flex: 1,
  },
  backToDiscoveryButton: {
    paddingVertical: 4,
    paddingRight: 12,
  },
  backToDiscoveryText: {
    fontSize: 13,
    fontFamily: typography.fontFamilySans,
    fontWeight: '500',
  },
  profileButton: {
    paddingVertical: 4,
    paddingLeft: 12,
  },
  profileText: {
    fontSize: 13,
    fontFamily: typography.fontFamilySans,
  },
  loadingText: {
    fontSize: 14,
    fontFamily: typography.fontFamilySans,
    letterSpacing: 0.5,
  },
  transitionContainer: {
    flex: 1,
  },
  bannerContainer: {
    paddingHorizontal: 20,
    alignItems: 'center',
    width: '100%',
  },
  hintContainer: {
    paddingHorizontal: 20,
    paddingVertical: 6,
    alignItems: 'center',
    width: '100%',
  },
  hintText: {
    fontSize: 12,
    textAlign: 'center',
  },
});
