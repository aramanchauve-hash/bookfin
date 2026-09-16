import fs from 'fs';
import path from 'path';
import { Animated } from 'react-native';
import {
  createSwipeController,
  isRecognizedLeftSwipe,
  isRecognizedRightSwipe,
} from '../src/components/SwipeableReadingContainer';

/**
 * Reproduit le bug iOS rapporté : après un swipe gauche reconnu, le contenu
 * restait translaté hors écran. Ces tests exercent directement la machine à
 * état du geste (createSwipeController), pas une réimplémentation locale,
 * pour garantir que la logique testée est celle réellement utilisée par le
 * PanResponder du composant.
 */
describe('Bookfin Mobile - Swipe : reset de transform après navigation', () => {
  // Le mock jest de react-native.Animated.Value expose .value publiquement,
  // contrairement au vrai type RN (qui n'expose pas cette propriété) : ce
  // helper contourne juste le typage, pas le runtime réellement testé.
  const currentValue = (v: Animated.Value): number => (v as unknown as { value: number }).value;

  function buildController(overrides: { enabled?: boolean; reduceMotion?: boolean } = {}) {
    const translateX = new Animated.Value(0);
    const opacity = new Animated.Value(1);
    const onRecognized = jest.fn();
    let enabled = overrides.enabled ?? true;
    const reduceMotion = overrides.reduceMotion ?? false;

    const controller = createSwipeController({
      getEnabled: () => enabled,
      getReduceMotion: () => reduceMotion,
      translateX,
      opacity,
      onRecognized,
    });

    return {
      controller,
      translateX,
      opacity,
      onRecognized,
      setEnabled: (v: boolean) => {
        enabled = v;
        controller.notifyEnabledChange(v);
      },
    };
  }

  test('swipe reconnu en revealed => déclenche la fonction métier (CONTINUE_READING)', () => {
    const { controller, onRecognized } = buildController({ enabled: true });
    controller.handleRelease(-60, 0);
    expect(onRecognized).toHaveBeenCalledTimes(1);
  });

  test('après un swipe réussi, translateX et opacity reviennent à leur état neutre', () => {
    const { controller, translateX, opacity } = buildController({ enabled: true });
    controller.handleRelease(-60, 0);
    expect(currentValue(translateX)).toBe(0);
    expect(currentValue(opacity)).toBe(1);
  });

  test('nouvelle page (enabled repasse à true) => transform identité et geste réarmé', () => {
    const { controller, translateX, opacity, onRecognized, setEnabled } = buildController({
      enabled: true,
    });
    controller.handleRelease(-60, 0);
    expect(onRecognized).toHaveBeenCalledTimes(1);

    // La page suivante arrive : le parent désactive puis réactive le swipe.
    setEnabled(false);
    setEnabled(true);

    expect(currentValue(translateX)).toBe(0);
    expect(currentValue(opacity)).toBe(1);
    expect(controller.hasTriggered()).toBe(false);

    // Un nouveau swipe sur la nouvelle page doit à nouveau fonctionner.
    controller.handleRelease(-60, 0);
    expect(onRecognized).toHaveBeenCalledTimes(2);
  });

  test('erreur réseau pendant la navigation : la transform reste à 0, indépendamment de l issue métier', () => {
    const translateX = new Animated.Value(0);
    const opacity = new Animated.Value(1);
    const onRecognized = jest.fn(() => {
      throw new Error('Erreur réseau simulée pendant CONTINUE_READING');
    });

    const controller = createSwipeController({
      getEnabled: () => true,
      getReduceMotion: () => false,
      translateX,
      opacity,
      onRecognized,
    });

    expect(() => controller.handleRelease(-60, 0)).toThrow();
    // Le reset a lieu avant l'appel métier : il n'est jamais compromis par
    // un échec de la requête de navigation.
    expect(currentValue(translateX)).toBe(0);
    expect(currentValue(opacity)).toBe(1);
  });

  test('swipe ignoré en état "reading" (enabled=false)', () => {
    const { controller, onRecognized } = buildController({ enabled: false });
    expect(controller.shouldClaim(-100, 0)).toBe(false);
    controller.handleRelease(-100, 0);
    expect(onRecognized).not.toHaveBeenCalled();
  });

  test('scroll vertical ne déclenche jamais la navigation', () => {
    const { controller, onRecognized } = buildController({ enabled: true });
    expect(controller.shouldClaim(-10, 60)).toBe(false);
    controller.handleRelease(-10, 60);
    expect(onRecognized).not.toHaveBeenCalled();
    expect(isRecognizedLeftSwipe(-10, 60)).toBe(false);
  });

  test('swipe droit appelle uniquement le retour lorsque le buffer précédent existe', () => {
    const translateX = new Animated.Value(0);
    const opacity = new Animated.Value(1);
    const onForward = jest.fn();
    const onBack = jest.fn();
    const controller = createSwipeController({
      getForwardEnabled: () => false,
      getBackEnabled: () => true,
      getReduceMotion: () => true,
      translateX,
      opacity,
      onRecognized: onForward,
      onBackRecognized: onBack,
    });
    expect(isRecognizedRightSwipe(60, 0)).toBe(true);
    controller.handleRelease(60, 0);
    expect(onBack).toHaveBeenCalledTimes(1);
    expect(onForward).not.toHaveBeenCalled();
  });

  test('plusieurs swipes rapides successifs => une seule navigation déclenchée', () => {
    const { controller, onRecognized } = buildController({ enabled: true });
    controller.handleRelease(-60, 0);
    controller.handleRelease(-70, 0);
    controller.handleRelease(-80, 0);
    expect(onRecognized).toHaveBeenCalledTimes(1);
  });

  test('Reduce Motion actif : navigation déclenchée sans animation, transform toujours propre', () => {
    const { controller, translateX, opacity, onRecognized } = buildController({
      enabled: true,
      reduceMotion: true,
    });
    controller.handleRelease(-60, 0);
    expect(onRecognized).toHaveBeenCalledTimes(1);
    expect(currentValue(translateX)).toBe(0);
    expect(currentValue(opacity)).toBe(1);
  });

  test('ReadingScreen ne contient plus qu un seul wrapping de PageFlipTransition/SwipeableReadingContainer, correctement fermé', () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/screens/ReadingScreen.tsx'),
      'utf-8'
    );
    const swipeOpen = source.match(/<SwipeableReadingContainer[\s>]/g) || [];
    const swipeClose = source.match(/<\/SwipeableReadingContainer>/g) || [];
    const flipOpen = source.match(/<PageFlipTransition[\s>]/g) || [];
    const flipClose = source.match(/<\/PageFlipTransition>/g) || [];

    expect(swipeOpen.length).toBe(1);
    expect(swipeClose.length).toBe(1);
    expect(flipOpen.length).toBe(1);
    expect(flipClose.length).toBe(1);
  });
});
