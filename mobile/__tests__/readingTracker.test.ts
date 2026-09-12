import { ReadingTrackerCore } from '../src/lib/metrics/useReadingTracker';

describe('ReadingTrackerCore - Mesure locale des signaux de lecture', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  test('Page sans scroll requis : défilement complet immédiat et content_overflows = false', () => {
    const tracker = new ReadingTrackerCore();

    // Simulation d'un événement scroll où tout le texte tient dans le viewport
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 0 },
        contentSize: { width: 360, height: 400 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);

    const snapshot = tracker.getSnapshot(150);
    expect(snapshot.content_overflows).toBe(false);
    expect(snapshot.scroll_depth_percent).toBe(100);
    expect(snapshot.bottom_reached).toBe(true);
  });

  test('Page avec scroll requis : calcul fidèle de la profondeur et du bas atteint', () => {
    const tracker = new ReadingTrackerCore();

    // Contenu long : 1200px sur un écran de 600px -> maxScrollable = 600px
    // 1. Défilement à 300px (50%)
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 300 },
        contentSize: { width: 360, height: 1200 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);

    let snapshot = tracker.getSnapshot(200);
    expect(snapshot.content_overflows).toBe(true);
    expect(snapshot.scroll_depth_percent).toBe(50);
    expect(snapshot.bottom_reached).toBe(false);

    // 2. Défilement jusqu'au bas (590px)
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 590 },
        contentSize: { width: 360, height: 1200 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);

    snapshot = tracker.getSnapshot(200);
    expect(snapshot.scroll_depth_percent).toBe(98);
    expect(snapshot.bottom_reached).toBe(true);

    // 3. Retour en arrière significatif (relecture)
    tracker.onScroll({
      nativeEvent: {
        contentOffset: { x: 0, y: 200 },
        contentSize: { width: 360, height: 1200 },
        layoutMeasurement: { width: 360, height: 600 },
      },
    } as any);

    snapshot = tracker.getSnapshot(200);
    expect(snapshot.scroll_back).toBe(true);
    // La profondeur maximale reste 98%
    expect(snapshot.scroll_depth_percent).toBe(98);
  });

  test('Temps actif : pause en arrière-plan et reprise au premier plan', () => {
    const tracker = new ReadingTrackerCore();

    // Avance de 5 secondes au premier plan
    jest.advanceTimersByTime(5000);
    expect(tracker.getActiveReadingTimeMs()).toBe(5000);

    // L'application passe en arrière-plan (mise en veille ou appel entrant)
    tracker.onAppStateChange('background');

    // 10 secondes s'écoulent en arrière-plan
    jest.advanceTimersByTime(10000);

    // Le temps actif n'a PAS augmenté : il est toujours à 5000 ms
    expect(tracker.getActiveReadingTimeMs()).toBe(5000);

    // Retour au premier plan (foreground)
    tracker.onAppStateChange('active');

    // 3 secondes supplémentaires au premier plan
    jest.advanceTimersByTime(3000);

    // Total actif = 5000 + 3000 = 8000 ms (les 10s de veille ont été ignorées)
    expect(tracker.getActiveReadingTimeMs()).toBe(8000);

    const snapshot = tracker.getSnapshot(200);
    expect(snapshot.dwell_time_ms).toBe(8000);
    // Vitesse de lecture calculée : 200 mots en 8s = 1500 WPM, plafonnée à 1200
    expect(snapshot.reading_speed_wpm).toBe(1200);
  });
});

