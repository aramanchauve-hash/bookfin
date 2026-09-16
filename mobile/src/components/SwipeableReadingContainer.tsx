import React, { useEffect, useRef, useState } from 'react';
import {
  AccessibilityInfo,
  Animated,
  PanResponder,
  PanResponderInstance,
  StyleSheet,
  ViewStyle,
} from 'react-native';

interface SwipeableReadingContainerProps {
  children: React.ReactNode;
  enabledForward: boolean;
  enabledBack?: boolean;
  onSwipeLeft: () => void;
  onSwipeRight?: () => void;
  style?: ViewStyle;
}

// Seuil unique de reconnaissance : pas de suivi continu du doigt, pas de
// second seuil basé sur la vélocité. Un geste est reconnu ou ne l'est pas.
export const SWIPE_DX_THRESHOLD = -50;
export const HORIZONTAL_TO_VERTICAL_RATIO = 2.5;
export const SWIPE_ANIMATION_DURATION_MS = 160;
// Micro-animation autonome : quelques pixels, jamais un déplacement hors écran.
export const SWIPE_TRANSLATE_DISTANCE = 18;
export const SWIPE_OPACITY_TARGET = 0.96;

/**
 * Un geste est reconnu comme "page suivante" uniquement s'il est franchement
 * horizontal vers la gauche : dx sous le seuil ET dominance horizontale forte.
 * Un scroll vertical (dy dominant) ne peut jamais satisfaire ce test.
 */
export function isRecognizedLeftSwipe(dx: number, dy: number): boolean {
  return dx < SWIPE_DX_THRESHOLD && Math.abs(dx) > Math.abs(dy) * HORIZONTAL_TO_VERTICAL_RATIO;
}

export function isRecognizedRightSwipe(dx: number, dy: number): boolean {
  return dx > Math.abs(SWIPE_DX_THRESHOLD) && Math.abs(dx) > Math.abs(dy) * HORIZONTAL_TO_VERTICAL_RATIO;
}

export interface SwipeControllerDeps {
  getEnabled?: () => boolean;
  getForwardEnabled?: () => boolean;
  getBackEnabled?: () => boolean;
  getReduceMotion: () => boolean;
  translateX: Animated.Value;
  opacity: Animated.Value;
  onRecognized: () => void;
  onBackRecognized?: () => void;
}

/**
 * Machine à état minimale pilotant le geste, indépendante de PanResponder et
 * de la couche JSX : testable directement, sans rendu React.
 *
 * Invariant central (corrige le bug iOS) : translateX/opacity ne sont
 * JAMAIS modifiés pendant le déplacement du doigt. Ils ne bougent que lors
 * de la micro-animation autonome post-relâchement, et sont remis à l'état
 * neutre AVANT même d'appeler la fonction métier de navigation. Le contenu
 * ne peut donc jamais rester translaté : soit il est au repos (0 / 1), soit
 * il est en train de revenir au repos.
 */
export function createSwipeController(deps: SwipeControllerDeps) {
  let hasTriggered = false;

  function resetTransform() {
    deps.translateX.setValue(0);
    deps.opacity.setValue(1);
  }

  function forwardEnabled() {
    return deps.getForwardEnabled?.() ?? deps.getEnabled?.() ?? false;
  }

  function backEnabled() {
    return deps.getBackEnabled?.() ?? false;
  }

  function playRecognizedSwipe(direction: 'forward' | 'back') {
    if (hasTriggered) return;
    hasTriggered = true;

    if (deps.getReduceMotion()) {
      // Reduce Motion : aucune animation visuelle, mais le contrat de reset
      // et de déclenchement métier reste strictement identique.
      resetTransform();
      if (direction === 'forward') deps.onRecognized();
      else deps.onBackRecognized?.();
      return;
    }

    Animated.parallel([
      Animated.timing(deps.translateX, {
        toValue: direction === 'forward' ? -SWIPE_TRANSLATE_DISTANCE : SWIPE_TRANSLATE_DISTANCE,
        duration: SWIPE_ANIMATION_DURATION_MS,
        useNativeDriver: true,
      }),
      Animated.timing(deps.opacity, {
        toValue: SWIPE_OPACITY_TARGET,
        duration: SWIPE_ANIMATION_DURATION_MS,
        useNativeDriver: true,
      }),
    ]).start(() => {
      // Reset avant la navigation : la page suivante apparaît toujours
      // avec une transform identité, jamais avec un reliquat de translation.
      resetTransform();
      if (direction === 'forward') deps.onRecognized();
      else deps.onBackRecognized?.();
    });
  }

  return {
    shouldClaim(dx: number, dy: number): boolean {
      return !hasTriggered && (
        (forwardEnabled() && isRecognizedLeftSwipe(dx, dy)) ||
        (backEnabled() && isRecognizedRightSwipe(dx, dy))
      );
    },
    handleRelease(dx: number, dy: number) {
      if (hasTriggered) return;
      if (forwardEnabled() && isRecognizedLeftSwipe(dx, dy)) {
        playRecognizedSwipe('forward');
      } else if (backEnabled() && isRecognizedRightSwipe(dx, dy)) {
        playRecognizedSwipe('back');
      } else {
        resetTransform();
      }
    },
    handleTerminate() {
      resetTransform();
    },
    /**
     * Appelé à chaque changement de la prop `enabled`. Filet de sécurité
     * indépendant de l'animation : dès que l'écran quitte 'revealed' (donc
     * dès le NAVIGATE_START synchrone), la transform est forcée à l'identité,
     * et le verrou anti-double-déclenchement est relâché seulement quand une
     * nouvelle page redevient 'revealed'.
     */
    notifyEnabledChange(enabled: boolean) {
      if (enabled) {
        hasTriggered = false;
      }
      resetTransform();
    },
    hasTriggered(): boolean {
      return hasTriggered;
    },
  };
}

/**
 * Conteneur gestuel horizontal activé STRICTEMENT en état 'revealed' :
 * - Un swipe gauche reconnu (dx < -50, forte dominance horizontale) déclenche
 *   la même action que "Continuer".
 * - Ne suit JAMAIS le doigt : aucune transform pendant onPanResponderMove.
 * - Ne capture JAMAIS un scroll vertical (dominance horizontale stricte).
 * - Micro-animation sobre (160 ms, quelques pixels, pas de 3D) au relâchement,
 *   suivie d'un reset garanti à l'identité avant la navigation.
 */
export const SwipeableReadingContainer: React.FC<SwipeableReadingContainerProps> = ({
  children,
  enabledForward,
  enabledBack = false,
  onSwipeLeft,
  onSwipeRight,
  style,
}) => {
  const [reduceMotion, setReduceMotion] = useState(false);
  const translateX = useRef(new Animated.Value(0)).current;
  const opacity = useRef(new Animated.Value(1)).current;
  const enabledForwardRef = useRef(enabledForward);
  const enabledBackRef = useRef(enabledBack);
  const onSwipeLeftRef = useRef(onSwipeLeft);
  const onSwipeRightRef = useRef(onSwipeRight);
  const reduceMotionRef = useRef(reduceMotion);

  enabledForwardRef.current = enabledForward;
  enabledBackRef.current = enabledBack;
  onSwipeLeftRef.current = onSwipeLeft;
  onSwipeRightRef.current = onSwipeRight;
  reduceMotionRef.current = reduceMotion;

  useEffect(() => {
    AccessibilityInfo.isReduceMotionEnabled().then(setReduceMotion);
    const sub = AccessibilityInfo.addEventListener('reduceMotionChanged', setReduceMotion);
    return () => {
      sub.remove();
    };
  }, []);

  const controller = useRef(
    createSwipeController({
      getForwardEnabled: () => enabledForwardRef.current,
      getBackEnabled: () => enabledBackRef.current,
      getReduceMotion: () => reduceMotionRef.current,
      translateX,
      opacity,
      onRecognized: () => onSwipeLeftRef.current(),
      onBackRecognized: () => onSwipeRightRef.current?.(),
    })
  ).current;

  useEffect(() => {
    controller.notifyEnabledChange(enabledForward || enabledBack);
  }, [enabledForward, enabledBack, controller]);

  const panResponder = useRef<PanResponderInstance>(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, gestureState) =>
        controller.shouldClaim(gestureState.dx, gestureState.dy),
      onPanResponderRelease: (_, gestureState) => {
        controller.handleRelease(gestureState.dx, gestureState.dy);
      },
      onPanResponderTerminate: () => {
        controller.handleTerminate();
      },
      onPanResponderTerminationRequest: () => true,
    })
  ).current;

  return (
    <Animated.View
      {...panResponder.panHandlers}
      style={[
        styles.container,
        style,
        {
          transform: [{ translateX }],
          opacity,
        },
      ]}
    >
      {children}
    </Animated.View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    width: '100%',
  },
});
