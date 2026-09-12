import { Platform } from 'react-native';

export const palette = {
  light: {
    background: '#FAF7F2', // Papier ivoire doux
    surface: '#FFFFFF',
    textPrimary: '#1E1B18', // Encre foncée chaude
    textSecondary: '#635D56',
    textMuted: '#968E85',
    accent: '#8B261D', // Sceau littéraire pourpre / bordeaux
    border: '#E8E2D9',
    buttonBackground: '#ECE6DC',
    buttonActive: '#DFD8CC',
    danger: '#B33927',
    success: '#2E6E45',
  },
  dark: {
    background: '#141311', // Noir charbon doux
    surface: '#1E1C19',
    textPrimary: '#EAE6DF',
    textSecondary: '#A8A095',
    textMuted: '#6B655D',
    accent: '#D95C4D',
    border: '#2E2B26',
    buttonBackground: '#262420',
    buttonActive: '#33302B',
    danger: '#D95C4D',
    success: '#4E9E67',
  },
};

export const typography = {
  fontFamilySerif: Platform.select({
    ios: 'Georgia',
    android: 'serif',
    default: 'serif',
  }),
  fontFamilySans: Platform.select({
    ios: 'System',
    android: 'sans-serif',
    default: 'sans-serif',
  }),
  reading: {
    fontSize: 18.5,
    lineHeight: 30,
    letterSpacing: 0.2,
  },
  metadata: {
    fontSize: 14,
    lineHeight: 20,
    letterSpacing: 0.5,
  },
  title: {
    fontSize: 21,
    lineHeight: 28,
    fontWeight: '600' as const,
  },
  author: {
    fontSize: 16,
    lineHeight: 22,
    fontStyle: 'italic' as const,
  },
  button: {
    fontSize: 15,
    fontWeight: '600' as const,
    letterSpacing: 0.3,
  },
};

export const layout = {
  horizontalPadding: 24,
  maxReadingWidth: 640,
  minTouchTarget: 48,
  borderRadius: 8,
};

