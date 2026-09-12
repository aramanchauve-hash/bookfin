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
  enabled: boolean;
  onSwipeLeft: () => void;
  style?: ViewStyle;
}

export const SWIPE_HORIZONTAL_THRESHOLD = -50;
export const SWIPE_VELOCITY_THRESHOLD = -0.3;
export const HORIZONTAL_TO_VERTICAL_RATIO = 2.5;
export const SWIPE_ANIMATION_DURATION_MS = 160;

/**
 * Conteneur gestuel horizontal activé STRICTEMENT en état 'revealed' :
 * - Un swipe gauche (page suivante) déclenche la même action que "Continuer".
 * - Ne capture JAMAIS les gestes de scroll vertical (ratio horizontal/vertical >= 2.5).
 * - Animation sobre et discrète de 160 ms (translation horizontale sans 3D ni son).
 * - Neutralisation instantanée si Reduce Motion est actif sur l'appareil.
 */
export const SwipeableReadingContainer: React.FC<SwipeableReadingContainerProps> = ({
  children,
  enabled,
  onSwipeLeft,
  style,
}) => {
  const [reduceMotion, setReduceMotion] = useState(false);
  const translateX = useRef(new Animated.Value(0)).current;
  const opacity = useRef(new Animated.Value(1)).current;
  const enabledRef = useRef(enabled);
  const onSwipeLeftRef = useRef(onSwipeLeft);
  const reduceMotionRef = useRef(reduceMotion);

  enabledRef.current = enabled;
  onSwipeLeftRef.current = onSwipeLeft;
  reduceMotionRef.current = reduceMotion;

  useEffect(() => {
    AccessibilityInfo.isReduceMotionEnabled().then(setReduceMotion);
    const sub = AccessibilityInfo.addEventListener('reduceMotionChanged', setReduceMotion);
    return () => {
      sub.remove();
    };
  }, []);

  const panResponder = useRef<PanResponderInstance>(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, gestureState) => {
        if (!enabledRef.current) return false;
        const { dx, dy } = gestureState;
        // Uniquement swipe vers la gauche significatif et strictement horizontal
        return (
          dx < -20 &&
          Math.abs(dx) > Math.abs(dy) * HORIZONTAL_TO_VERTICAL_RATIO
        );
      },
      onPanResponderMove: (_, gestureState) => {
        if (!enabledRef.current) return;
        if (gestureState.dx < 0) {
          // Résistance physique subtile pendant le glissement
          translateX.setValue(Math.max(gestureState.dx * 0.4, -60));
        }
      },
      onPanResponderRelease: (_, gestureState) => {
        if (!enabledRef.current) return;
        const isEligibleSwipe =
          gestureState.dx <= SWIPE_HORIZONTAL_THRESHOLD ||
          (gestureState.dx < -20 && gestureState.vx <= SWIPE_VELOCITY_THRESHOLD);

        if (isEligibleSwipe) {
          if (reduceMotionRef.current) {
            translateX.setValue(0);
            onSwipeLeftRef.current();
          } else {
            Animated.parallel([
              Animated.timing(translateX, {
                toValue: -80,
                duration: SWIPE_ANIMATION_DURATION_MS,
                useNativeDriver: true,
              }),
              Animated.timing(opacity, {
                toValue: 0.3,
                duration: SWIPE_ANIMATION_DURATION_MS,
                useNativeDriver: true,
              }),
            ]).start(() => {
              translateX.setValue(0);
              opacity.setValue(1);
              onSwipeLeftRef.current();
            });
          }
        } else {
          // Retour doux à la position d'origine si le geste n'est pas complété
          Animated.spring(translateX, {
            toValue: 0,
            useNativeDriver: true,
            bounciness: 0,
          }).start();
        }
      },
      onPanResponderTerminate: () => {
        translateX.setValue(0);
        opacity.setValue(1);
      },
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
