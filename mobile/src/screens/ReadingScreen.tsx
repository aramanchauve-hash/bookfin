import React from 'react';
import {
  ActivityIndicator,
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

/**
 * Écran principal unique de Bookfin (ReadingScreen) :
 * - Convergence immédiate vers la lecture dès l'ouverture.
 * - Aucune page intermédiaire, aucun catalogue, aucun profil.
 * - Anonymat absolu avant réaction.
 */
export const ReadingScreen: React.FC = () => {
  const insets = useSafeAreaInsets();
  const colorScheme = useColorScheme();
  const theme = colorScheme === 'dark' ? palette.dark : palette.light;

  const {
    state,
    tracker,
    handleReaction,
    handleContinueBook,
    handleRandomPage,
    handleRetry,
  } = useReadingSession();

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
      {/* 1. Écran de chargement sobre initial */}
      {isInitialLoading && (
        <View style={styles.centerBox}>
          <ActivityIndicator size="small" color={theme.textMuted} />
          <Text style={[styles.loadingText, { color: theme.textMuted }]}>
            Ouverture du livre...
          </Text>
        </View>
      )}

      {/* 2. Bannière d'erreur réseau discrète si anomalie (vraies erreurs réseau/serveur uniquement) */}
      {state.status === 'error' && state.errorMessage && (
        <NetworkBanner message={state.errorMessage} onRetry={handleRetry} />
      )}

      {/* 2bis. Indication discrète non bloquante après un 422 de validation de lecture :
          la lecture et la réaction restent immédiatement disponibles, pas de bouton Réessayer. */}
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

      {/* 3. Contenu de lecture enveloppé dans le conteneur gestuel et la transition de page */}
      {state.currentPage && (
        <SwipeableReadingContainer
          enabled={state.status === 'revealed'}
          onSwipeLeft={handleContinueBook}
          style={styles.transitionContainer}
        >
          <PageFlipTransition
            triggerKey={state.currentPage.impression_id || state.currentPage.id || state.currentPage.page_id}
            type={state.transitionType}
            style={styles.transitionContainer}
          >
            <ReadingContent
              page={state.currentPage}
              onScroll={tracker.onScroll}
            />
          </PageFlipTransition>
        </SwipeableReadingContainer>
      )}

      {/* 4. Révélation sobre de l'auteur et du titre post-réaction */}
      {(state.status === 'revealed' || state.status === 'end_of_edition') &&
        state.metadata && (
          <View style={styles.bannerContainer}>
            <RevealBanner metadata={state.metadata} />
          </View>
        )}

      {/* 5. Barre de réaction (Préférence : Like / Dislike) */}
      {(state.status === 'reading' || state.status === 'reacting') && (
        <ReactionToolbar
          onReact={handleReaction}
          isReacting={state.status === 'reacting'}
          selectedReaction={state.userReaction}
        />
      )}

      {/* 6. Barre de navigation post-révélation (Navigation : Suite / Hasard) */}
      {(state.status === 'revealed' ||
        state.status === 'navigating' ||
        state.status === 'end_of_edition') && (
        <NavigationToolbar
          onContinue={handleContinueBook}
          onRandom={handleRandomPage}
          isNavigating={state.status === 'navigating'}
          navigatingAction={state.lastNavigationAction}
          isEndOfEdition={state.status === 'end_of_edition'}
          endOfEditionMessage={state.errorMessage}
        />
      )}
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
