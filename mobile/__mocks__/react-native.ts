export const Platform = {
  OS: 'android',
  select: (objs: any) => objs.android || objs.default,
};

type AppStateListener = (state: string) => void;
const appStateListeners: AppStateListener[] = [];

export const AppState = {
  currentState: 'active',
  addEventListener: (_type: string, listener: AppStateListener) => {
    appStateListeners.push(listener);
    return {
      remove: () => {
        const idx = appStateListeners.indexOf(listener);
        if (idx !== -1) appStateListeners.splice(idx, 1);
      },
    };
  },
  // Méthode de simulation pour les tests
  _simulateStateChange: (nextState: string) => {
    AppState.currentState = nextState;
    appStateListeners.forEach((fn) => fn(nextState));
  },
  _resetListeners: () => {
    appStateListeners.length = 0;
    AppState.currentState = 'active';
  },
};

export const AccessibilityInfo = {
  isReduceMotionEnabled: jest.fn().mockResolvedValue(false),
  addEventListener: jest.fn().mockReturnValue({ remove: jest.fn() }),
};

export const StyleSheet = {
  create: (styles: any) => styles,
};

export const View = 'View';
export const Text = 'Text';
export const ScrollView = 'ScrollView';
export const Pressable = 'Pressable';
export const ActivityIndicator = 'ActivityIndicator';
export const Animated = {
  Value: class {
    value: number;
    constructor(v: number) {
      this.value = v;
    }
    setValue(v: number) {
      this.value = v;
    }
    interpolate() {
      return this;
    }
  },
  timing: () => ({
    start: (cb?: any) => {
      if (cb) cb();
    },
  }),
  View: 'Animated.View',
};
export const useColorScheme = () => 'light';

