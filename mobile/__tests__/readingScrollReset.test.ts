import { resetReadingScroll, usesJustifiedReadingLayout } from '../src/components/ReadingContent';

describe('ReadingContent scroll reset', () => {
  test('page B resets the reused reading surface to y=0 without animation', () => {
    const scrollTo = jest.fn();
    resetReadingScroll({ scrollTo });
    expect(scrollTo).toHaveBeenCalledWith({ x: 0, y: 0, animated: false });
  });

  test('an unavailable ref is safe while a new page is mounting', () => {
    expect(() => resetReadingScroll(null)).not.toThrow();
  });

  test('justification is reserved for languages where word spacing is appropriate', () => {
    expect(usesJustifiedReadingLayout('fr')).toBe(true);
    expect(usesJustifiedReadingLayout('ru')).toBe(true);
    expect(usesJustifiedReadingLayout('zh')).toBe(false);
    expect(usesJustifiedReadingLayout('ja')).toBe(false);
  });
});
