import { useEffect, useRef, useCallback, useState } from 'react';
import { AppState, AppStateStatus, NativeScrollEvent, NativeSyntheticEvent } from 'react-native';

export interface ReadingMetricsSnapshot {
  dwell_time_ms: number;
  scroll_depth_percent: number;
  reading_speed_wpm: number;
  content_overflows: boolean;
  bottom_reached: boolean;
  scroll_back: boolean;
  scroll_offset_y: number;
}

/**
 * Moteur pur de calcul et de suivi des signaux de lecture.
 * Entièrement testable sans dépendance au moteur de rendu React.
 */
export class ReadingTrackerCore {
  private accumulatedMs: number = 0;
  private activeSince: number | null = null;
  private isPaused: boolean = false;

  private maxScrollPercent: number = 0;
  private prevScrollY: number = 0;
  private bottomReached: boolean = false;
  private scrollBack: boolean = false;
  private contentOverflows: boolean = false;

  constructor() {
    this.reset();
  }

  public reset(): void {
    this.accumulatedMs = 0;
    this.activeSince = Date.now();
    this.isPaused = false;
    this.maxScrollPercent = 0;
    this.prevScrollY = 0;
    this.bottomReached = false;
    this.scrollBack = false;
    this.contentOverflows = false;
  }

  public onAppStateChange(nextState: AppStateStatus | string): void {
    const now = Date.now();
    if (nextState === 'active') {
      if (this.isPaused) {
        this.activeSince = now;
        this.isPaused = false;
      }
    } else {
      if (!this.isPaused && this.activeSince !== null) {
        this.accumulatedMs += Math.max(0, now - this.activeSince);
        this.activeSince = null;
        this.isPaused = true;
      }
    }
  }

  public getActiveReadingTimeMs(): number {
    let total = this.accumulatedMs;
    if (!this.isPaused && this.activeSince !== null) {
      total += Math.max(0, Date.now() - this.activeSince);
    }
    // Plafond d'assainissement de 30 minutes
    return Math.min(total, 1_800_000);
  }

  public onScroll(event: NativeSyntheticEvent<NativeScrollEvent>): void {
    const { contentOffset, contentSize, layoutMeasurement } = event.nativeEvent;
    const currentY = contentOffset.y;
    const totalHeight = contentSize.height;
    const visibleHeight = layoutMeasurement.height;
    const maxScrollable = totalHeight - visibleHeight;

    if (maxScrollable > 20) {
      this.contentOverflows = true;
      const progress = Math.min(100, Math.max(0, (currentY / maxScrollable) * 100));
      if (progress > this.maxScrollPercent) {
        this.maxScrollPercent = Math.round(progress);
      }

      if (currentY >= maxScrollable - 25) {
        this.bottomReached = true;
      }

      if (currentY < this.prevScrollY - 40) {
        this.scrollBack = true;
      }
    } else {
      this.contentOverflows = false;
      this.maxScrollPercent = 100;
      this.bottomReached = true;
    }

    this.prevScrollY = currentY;
  }

  public onWebViewScroll(state: { scrollTop: number; scrollHeight: number; viewportHeight: number }): void {
    this.onScroll({ nativeEvent: {
      contentOffset: { x: 0, y: state.scrollTop },
      contentSize: { width: 0, height: state.scrollHeight },
      layoutMeasurement: { width: 0, height: state.viewportHeight },
    } } as NativeSyntheticEvent<NativeScrollEvent>);
  }

  public onContentSizeChange(
    _contentWidth: number,
    contentHeight: number,
    containerHeight: number
  ): void {
    if (contentHeight > containerHeight + 20) {
      this.contentOverflows = true;
    } else {
      this.contentOverflows = false;
      this.maxScrollPercent = 100;
      this.bottomReached = true;
    }
  }

  public getSnapshot(tokenCount: number): ReadingMetricsSnapshot {
    const activeMs = this.getActiveReadingTimeMs();
    const minutes = activeMs / 60000;
    const readingSpeedWpm =
      minutes > 0.01 && tokenCount > 0 ? Math.round(tokenCount / minutes) : 0;

    return {
      dwell_time_ms: activeMs,
      scroll_depth_percent: this.maxScrollPercent,
      reading_speed_wpm: Math.min(readingSpeedWpm, 1200),
      content_overflows: this.contentOverflows,
      bottom_reached: this.bottomReached,
      scroll_back: this.scrollBack,
      scroll_offset_y: this.prevScrollY,
    };
  }
}

/**
 * Convertit le scroll_depth du tracker (pourcentage 0-100) vers la fraction 0.0-1.0
 * attendue par le backend (cf. ReadingValidationConfig::min_scroll_depth = 0.75).
 */
export function toServerScrollDepth(scrollDepthPercent: number): number {
  return scrollDepthPercent / 100;
}

/**
 * Hook React intégrant le ReadingTrackerCore avec le cycle de vie du composant.
 */
export function useReadingTracker(pageId?: string) {
  const trackerRef = useRef<ReadingTrackerCore>(new ReadingTrackerCore());
  const [bottomReached, setBottomReached] = useState(false);

  useEffect(() => {
    trackerRef.current.reset();
    setBottomReached(false);
  }, [pageId]);

  useEffect(() => {
    const handleAppState = (state: AppStateStatus) => {
      trackerRef.current.onAppStateChange(state);
    };

    const sub = AppState.addEventListener('change', handleAppState);
    return () => {
      sub.remove();
    };
  }, []);

  const onScroll = useCallback((e: NativeSyntheticEvent<NativeScrollEvent>) => {
    trackerRef.current.onScroll(e);
    setBottomReached(trackerRef.current.getSnapshot(0).bottom_reached);
  }, []);

  const onContentSizeChange = useCallback(
    (w: number, h: number, containerH: number) => {
      trackerRef.current.onContentSizeChange(w, h, containerH);
      setBottomReached(trackerRef.current.getSnapshot(0).bottom_reached);
    },
    []
  );

  const onWebViewScrollState = useCallback((state: { scrollTop: number; scrollHeight: number; viewportHeight: number }) => {
    trackerRef.current.onWebViewScroll(state);
    setBottomReached(trackerRef.current.getSnapshot(0).bottom_reached);
  }, []);

  const getSnapshot = useCallback((tokenCount: number) => {
    return trackerRef.current.getSnapshot(tokenCount);
  }, []);

  const getActiveReadingTimeMs = useCallback(() => {
    return trackerRef.current.getActiveReadingTimeMs();
  }, []);

  const reset = useCallback(() => {
    trackerRef.current.reset();
  }, []);

  return {
    onScroll,
    onContentSizeChange,
    onWebViewScrollState,
    getSnapshot,
    getActiveReadingTimeMs,
    reset,
    bottomReached,
  };
}
