import { ApiError, getApiBaseUrl, requestJson } from '../src/lib/api/client';
import {
  continueReading,
  fetchNextRandomPage,
  revealPageMetadata,
  submitReaction,
} from '../src/lib/api/readingApi';
import { Platform } from 'react-native';

// Mock global fetch
const mockFetch = jest.fn();
(global as any).fetch = mockFetch;

describe('API Client & Résilience Réseau', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  test('Résolution de l\'URL : Android Emulator utilise 127.0.0.1:3000 via adb reverse', () => {
    (Platform as any).OS = 'android';
    delete process.env.EXPO_PUBLIC_API_BASE_URL;

    expect(getApiBaseUrl()).toBe('http://127.0.0.1:3000');
  });

  test('fetchNextRandomPage : extrait reçu anonyme sans fuite de métadonnées', async () => {
    const mockPage = {
      id: 'imp-1',
      page_id: 'page-1',
      page_number: 1,
      source_page_number: '1',
      content: 'Au commencement était le verbe.',
      language_tag: 'fr',
      served_at: '2026-09-12T10:00:00Z',
      token_count: 50,
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify(mockPage),
    });

    const page = await fetchNextRandomPage('user-1');
    expect(page.content).toBe('Au commencement était le verbe.');
    expect((page as any).title).toBeUndefined();
    expect((page as any).author).toBeUndefined();
  });

  test('submitReaction : envoi idempotent avec le même event_id en cas de retry', async () => {
    const eventId = 'event-id-stable-123';
    const payload = {
      user_id: 'user-1',
      page_id: 'page-1',
      reaction: 'like' as const,
      reaction_type: 'like' as const,
      reading_time_ms: 15000,
      dwell_time_ms: 15000,
      scroll_depth: 85,
      scroll_depth_percent: 85,
      reading_speed_wpm: 220,
      event_id: eventId,
      impression_id: 'imp-1',
    };

    const mockResponse = {
      status: 'recorded',
      reaction_id: 'reac-1',
      event_id: eventId,
      page_id: 'page-1',
      user_id: 'user-1',
      dwell_time_ms: 15000,
      scroll_depth_percent: 85,
      reading_speed_wpm: 220,
      is_qualified: true,
    };

    // 1. Premier envoi
    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 201,
      text: async () => JSON.stringify(mockResponse),
    });

    const res1 = await submitReaction(payload);
    expect(res1.status).toBe('recorded');
    expect(res1.event_id).toBe(eventId);

    // 2. Deuxième envoi (retry avec le même event_id)
    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      text: async () => JSON.stringify(mockResponse),
    });

    const res2 = await submitReaction(payload);
    expect(res2.event_id).toBe(eventId);

    // Vérification que les deux appels fetch ont envoyé exactement le même payload JSON
    const body1 = JSON.parse(mockFetch.mock.calls[0][1].body);
    const body2 = JSON.parse(mockFetch.mock.calls[1][1].body);
    expect(body1.event_id).toBe(body2.event_id);
  });

  test('continueReading : détection et typage de EndOfEdition (HTTP 404 structuré)', async () => {
    const endOfEditionError = {
      error: 'end_of_edition',
      edition_id: 'edition-1',
      last_page_number: 10,
      message: 'Fin du livre atteinte',
    };

    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 404,
      text: async () => JSON.stringify(endOfEditionError),
    });

    try {
      await continueReading('page-1', 'imp-1', 'user-1');
      throw new Error('Aurait dû lever une ApiError');
    } catch (err: any) {
      expect(err).toBeInstanceOf(ApiError);
      expect(err.status).toBe(404);
      expect(err.isEndOfEdition).toBe(true);
      expect(err.payload.last_page_number).toBe(10);
    }
  });

  test('revealPageMetadata : 403 Forbidden intercepté si aucune réaction enregistrée', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 403,
      text: async () => JSON.stringify({ message: 'Reaction required before reveal' }),
    });

    try {
      await revealPageMetadata('page-unreacted', 'user-1');
      throw new Error('Aurait dû lever une ApiError 403');
    } catch (err: any) {
      expect(err).toBeInstanceOf(ApiError);
      expect(err.status).toBe(403);
    }
  });
});
