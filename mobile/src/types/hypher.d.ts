declare module 'hypher' {
  class Hypher { constructor(patterns: unknown); hyphenate(word: string): string[]; }
  export = Hypher;
}
declare module 'hyphenation.fr' { const patterns: unknown; export = patterns; }
declare module 'hyphenation.en-us' { const patterns: unknown; export = patterns; }
declare module 'hyphenation.es' { const patterns: unknown; export = patterns; }
