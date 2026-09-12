import React, { useEffect, useRef, useState } from 'react';
import { AccessibilityInfo, Animated, StyleSheet, ViewStyle } from 'react-native';

interface PageFlipTransitionProps {
  children: React.ReactNode;
  triggerKey: string | number;
  type?: 'page_flip' | 'fade' | 'none';
  style?: ViewStyle;
}

/**
 * Animation fugace et discrète évoquant une page tournée (150–200 ms).
 * Respecte le paramètre système "Reduce Motion".
 */
export const PageFlipTransition: React.FC<PageFlipTransitionProps> = ({
  children,
  triggerKey,
  type = 'page_flip',
  style,
}) => {
  const [reduceMotion, setReduceMotion] = useState(false);
  const animValue = useRef(new Animated.Value(1)).current;

  useEffect(() => {
    AccessibilityInfo.isReduceMotionEnabled().then(setReduceMotion);
    const sub = AccessibilityInfo.addEventListener('reduceMotionChanged', setReduceMotion);
    return () => {
      sub.remove();
    };
  }, []);

  useEffect(() => {
    if (reduceMotion || type === 'none') {
      animValue.setValue(1);
      return;
    }

    // Animation ultra-courte (160 ms)
    animValue.setValue(0);
    Animated.timing(animValue, {
      toValue: 1,
      duration: type === 'page_flip' ? 180 : 120,
      useNativeDriver: true,
    }).start();
  }, [triggerKey, type, reduceMotion, animValue]);

  if (reduceMotion || type === 'none') {
    return <Animated.View style={[styles.container, style]}>{children}</Animated.View>;
  }

  // Effet de translation horizontale subtile (12px) et fondu d'apparition
  const translateX = animValue.interpolate({
    inputRange: [0, 1],
    outputRange: [type === 'page_flip' ? 16 : 0, 0],
  });

  const opacity = animValue.interpolate({
    inputRange: [0, 0.4, 1],
    outputRange: [0, 0.6, 1],
  });

  return (
    <Animated.View
      style={[
        styles.container,
        style,
        {
          opacity,
          transform: [{ translateX }],
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

