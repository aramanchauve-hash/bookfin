export type ReaderBridgeMessage =
  | { type: 'READY'; pageId: string }
  | { type: 'SCROLL_STATE'; scrollTop: number; scrollHeight: number; viewportHeight: number; atBottom: boolean }
  | { type: 'SWIPE_LEFT'; atBottom: boolean }
  | { type: 'SWIPE_RIGHT' }
  | { type: 'FONT_STATUS'; family: string; requestedMode: 'poliphili' | 'system'; romanLoaded: boolean; italicLoaded: boolean; status: 'loaded' | 'fallback' | 'not_installed' | 'italic_missing' };

export function parseReaderBridgeMessage(data: string): ReaderBridgeMessage | null {
  try {
    const message = JSON.parse(data) as Record<string, unknown>;
    if (message.type === 'READY' && typeof message.pageId === 'string') return { type: 'READY', pageId: message.pageId };
    if (message.type === 'SWIPE_LEFT' && typeof message.atBottom === 'boolean') return { type: 'SWIPE_LEFT', atBottom: message.atBottom };
    if (message.type === 'SWIPE_RIGHT') return { type: 'SWIPE_RIGHT' };
    if (message.type === 'FONT_STATUS' && typeof message.family === 'string' && (message.requestedMode === 'poliphili' || message.requestedMode === 'system') && typeof message.romanLoaded === 'boolean' && typeof message.italicLoaded === 'boolean' && (message.status === 'loaded' || message.status === 'fallback' || message.status === 'not_installed' || message.status === 'italic_missing')) return message as Extract<ReaderBridgeMessage, { type: 'FONT_STATUS' }>;
    if (message.type === 'SCROLL_STATE' && ['scrollTop', 'scrollHeight', 'viewportHeight'].every((key) => typeof message[key] === 'number') && typeof message.atBottom === 'boolean') return message as ReaderBridgeMessage;
  } catch { /* Ignore untrusted or malformed WebView input. */ }
  return null;
}
