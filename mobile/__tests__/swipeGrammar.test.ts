import fs from 'fs';
import path from 'path';
import { ReadingTrackerCore } from '../src/lib/metrics/useReadingTracker';

describe('nouvelle grammaire swipe = like + continuation', () => {
  test('une page scrollable ne devient swipable qu’après le bas', () => {
    const tracker = new ReadingTrackerCore();
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 400 },
        contentSize: { width: 320, height: 2000 },
        layoutMeasurement: { width: 320, height: 600 },
      },
    } as any);
    expect(tracker.getSnapshot(100).bottom_reached).toBe(false);
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 1400 },
        contentSize: { width: 320, height: 2000 },
        layoutMeasurement: { width: 320, height: 600 },
      },
    } as any);
    expect(tracker.getSnapshot(100).bottom_reached).toBe(true);
  });

  test('une page sans overflow est immédiatement éligible au swipe', () => {
    const tracker = new ReadingTrackerCore();
    tracker.onContentSizeChange(320, 500, 600);
    expect(tracker.getSnapshot(100).bottom_reached).toBe(true);
  });

  test('le swipe passe par like confirmé puis continuation, tandis que les taps restent séparés', () => {
    const source = fs.readFileSync(path.resolve(__dirname, '../src/state/useReadingSession.ts'), 'utf-8');
    const swipeAction = source.slice(source.indexOf('const handleSwipeLikeContinue'));
    expect(swipeAction).toContain("reaction: 'like'");
    expect(swipeAction).toContain("navigation_action: 'continue_book'");
    expect(swipeAction.indexOf('await submitReaction')).toBeLessThan(swipeAction.indexOf('await continueReading'));
    expect(source).toContain('forwardCachedPageRef');
    expect(source).toContain('previousPageRef');
  });

  test('les contrôles pouce conservent des libellés explicites', () => {
    const source = fs.readFileSync(path.resolve(__dirname, '../src/components/ReactionToolbar.tsx'), 'utf-8');
    expect(source).toContain('accessibilityLabel="J\'aime cette page"');
    expect(source).toContain('accessibilityLabel="Je n\'aime pas cette page"');
  });
});
