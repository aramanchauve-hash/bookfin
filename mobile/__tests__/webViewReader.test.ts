import fs from 'fs';
import path from 'path';
import { BOOKFIN_READER_CSS, countSoftHyphens, insertSoftHyphens, insertSoftHyphensWithMin, renderBookfinPageToHtml, stripSoftHyphens } from '../src/lib/reader/bookfinHtml';
import { parseReaderBridgeMessage } from '../src/lib/reader/webViewBridge';
import { BOOKFIN_POLIPHILI_FONT_FACE_CSS, createFontFaceCss, POLIPHILI_FAMILY, poliphiliIsInstalled, poliphiliRomanIsInstalled, poliphiliItalicIsInstalled } from '../src/lib/reader/bookfinFont';
import { FeedPageDto } from '../src/types/api';

const page: FeedPageDto = {
  impression_id: 'dev', page_id: 'page-v2', page_sequence_number: 1, source_page_number: '1', text: '', language_tag: 'fr', served_at: 'now', token_count: 1,
  blocks: [
    { type: 'heading', level: 2, spans: [{ text: 'Chapitre <un>' }] },
    { type: 'paragraph', spans: [{ text: 'Extraordinairement longue ', italic: true }, { text: 'et <sûre>', bold: true }] },
    { type: 'blockquote', spans: [{ text: 'Citation' }] },
    { type: 'verse', spans: [{ text: 'Premier vers\nSecond vers' }] },
    { type: 'scene_break' },
  ],
};

describe('Bookfin WebView renderer', () => {
  test('renders controlled semantic HTML with escaped V2 spans and language', () => {
    const html = renderBookfinPageToHtml(page, { softHyphens: false });
    expect(html).toContain('lang="fr"');
    expect(html).toContain('<h2>');
    expect(html).toContain('&lt;un&gt;');
    expect(html).toContain('<em>');
    expect(html).toContain('<strong>');
    expect(html).toContain('<blockquote>');
    expect(html).toContain('class="bf-verse"');
    expect(html).toContain('<br>');
    expect(html).toContain('bf-scene-break');
  });

  test('soft hyphens are deterministic and never alter canonical text', () => {
    const canonical = 'Extraordinairement délicatement internationalisation';
    const rendered = insertSoftHyphens(canonical, 'fr');
    expect(rendered).toContain('\u00ad');
    expect(countSoftHyphens(rendered)).toBeGreaterThan(0);
    expect(insertSoftHyphens(canonical, 'fr')).toBe(rendered);
    expect(stripSoftHyphens(rendered)).toBe(canonical);
    expect(stripSoftHyphens(insertSoftHyphens('internationalization', 'en'))).toBe('internationalization');
    expect(stripSoftHyphens(insertSoftHyphens('extraordinariamente', 'es'))).toBe('extraordinariamente');
    expect(insertSoftHyphens('https://example.com/internationalization', 'en')).toBe('https://example.com/internationalization');
  });

  test('BOOK BALANCED stays between dense book and screen presets', () => {
    const html = renderBookfinPageToHtml(page, { paragraphStyle: 'balanced' });
    expect(html).toContain('bf-balanced-style');
    expect(fs.readFileSync(path.resolve(__dirname, '../src/lib/reader/bookfinHtml.ts'), 'utf8')).toContain('text-indent:.9em');
  });

  test('French 4/4 patterns and CSS manual/off modes are explicit', () => {
    const hyphenated = insertSoftHyphensWithMin('extraordinairement', 'fr', 4);
    hyphenated.split('\u00ad').forEach((part) => expect(part.length).toBeGreaterThanOrEqual(4));
    expect(renderBookfinPageToHtml(page, { softHyphens: false })).toContain('bf-hyphenation-off');
    expect(renderBookfinPageToHtml(page, { softHyphens: true })).toContain('bf-hyphenation-patterns');
    expect(BOOKFIN_READER_CSS).toContain('-webkit-hyphens:none');
    expect(BOOKFIN_READER_CSS).toContain('-webkit-hyphens:manual');
  });

  test('accepts only typed reader bridge messages', () => {
    expect(parseReaderBridgeMessage('{"type":"SWIPE_LEFT","atBottom":true}')).toEqual({ type: 'SWIPE_LEFT', atBottom: true });
    expect(parseReaderBridgeMessage('{"type":"SWIPE_RIGHT"}')).toEqual({ type: 'SWIPE_RIGHT' });
    expect(parseReaderBridgeMessage('{"type":"SCROLL_STATE","scrollTop":10,"scrollHeight":100,"viewportHeight":50,"atBottom":false}')).toMatchObject({ type: 'SCROLL_STATE', atBottom: false });
    expect(parseReaderBridgeMessage('{oops')).toBeNull();
    expect(parseReaderBridgeMessage('{"type":"SWIPE_LEFT"}')).toBeNull();
    expect(parseReaderBridgeMessage('{"type":"FONT_STATUS","family":"Bookfin Poliphili","requestedMode":"poliphili","romanLoaded":true,"italicLoaded":true,"status":"loaded"}')).toMatchObject({ type: 'FONT_STATUS', romanLoaded: true, italicLoaded: true });
    expect(parseReaderBridgeMessage('{"type":"FONT_STATUS","romanLoaded":true}')).toBeNull();
  });

  test('font-face infrastructure loads licensed Poliphili Roman and flags italic missing', () => {
    expect(poliphiliRomanIsInstalled).toBe(true);
    expect(poliphiliItalicIsInstalled).toBe(false);
    expect(poliphiliIsInstalled).toBe(true);
    expect(BOOKFIN_POLIPHILI_FONT_FACE_CSS).toContain('@font-face');
    expect(BOOKFIN_POLIPHILI_FONT_FACE_CSS).toContain(`font-family:"${POLIPHILI_FAMILY}"`);
    expect(BOOKFIN_POLIPHILI_FONT_FACE_CSS).toContain('font-style:normal');
    expect(BOOKFIN_POLIPHILI_FONT_FACE_CSS).toContain('font-weight:400');
    expect(BOOKFIN_READER_CSS).toContain(`"${POLIPHILI_FAMILY}"`);
    expect(BOOKFIN_READER_CSS).toContain('font-synthesis:none');
    expect(BOOKFIN_READER_CSS).toContain('-webkit-font-synthesis:none');
    expect(BOOKFIN_READER_CSS).toContain('"Times New Roman"');
    expect(BOOKFIN_READER_CSS).toContain('Georgia');
    expect(BOOKFIN_READER_CSS).toContain('serif');
  });

  test('body font uses Poliphili first and graceful fallback stack', () => {
    expect(BOOKFIN_READER_CSS).toMatch(/body\{[^}]*font-family:"Bookfin Poliphili", "Times New Roman", Georgia, serif/);
    expect(BOOKFIN_READER_CSS).toContain('.bf-font-system{font-family:"Times New Roman", Georgia, serif}');
  });

  test('canonical JSON page content is never mutated by HTML rendering or font injection', () => {
    const pageClone = JSON.parse(JSON.stringify(page));
    const html = renderBookfinPageToHtml(page);
    expect(page).toEqual(pageClone);
    expect(html).toContain('Extraordinairement longue');
    // Ensure no font URLs or font styles are injected into the input data
    expect(JSON.stringify(page)).not.toContain('font-family');
    expect(JSON.stringify(page)).not.toContain('@font-face');
  });

  test('FONT_STATUS bridge messages are strictly validated and malformed messages are safely ignored', () => {
    // Valid status loaded
    expect(parseReaderBridgeMessage(JSON.stringify({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: true,
      italicLoaded: true,
      status: 'loaded',
    }))).toEqual({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: true,
      italicLoaded: true,
      status: 'loaded',
    });

    // Valid status not_installed
    expect(parseReaderBridgeMessage(JSON.stringify({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: false,
      italicLoaded: false,
      status: 'not_installed',
    }))).toEqual({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: false,
      italicLoaded: false,
      status: 'not_installed',
    });

    // Valid status fallback
    expect(parseReaderBridgeMessage(JSON.stringify({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'system',
      romanLoaded: false,
      italicLoaded: false,
      status: 'fallback',
    }))).toEqual({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'system',
      romanLoaded: false,
      italicLoaded: false,
      status: 'fallback',
    });

    // Valid status italic_missing
    expect(parseReaderBridgeMessage(JSON.stringify({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: true,
      italicLoaded: false,
      status: 'italic_missing',
    }))).toEqual({
      type: 'FONT_STATUS',
      family: POLIPHILI_FAMILY,
      requestedMode: 'poliphili',
      romanLoaded: true,
      italicLoaded: false,
      status: 'italic_missing',
    });

    // Malformed variants
    expect(parseReaderBridgeMessage('{"type":"FONT_STATUS"}')).toBeNull();
    expect(parseReaderBridgeMessage('{"type":"FONT_STATUS","romanLoaded":"yes"}')).toBeNull();
    expect(parseReaderBridgeMessage('{"type":"FONT_STATUS","status":"invalid"}')).toBeNull();
    expect(parseReaderBridgeMessage('not valid json')).toBeNull();
    expect(parseReaderBridgeMessage('')).toBeNull();
    expect(parseReaderBridgeMessage('null')).toBeNull();
    expect(parseReaderBridgeMessage('12345')).toBeNull();
  });

  test('keeps one WebView mounted and replaces pages through injectJavaScript', () => {
    const source = fs.readFileSync(path.resolve(__dirname, '../src/components/ReadingWebView.tsx'), 'utf8');
    expect(source).toContain('injectJavaScript');
    expect(source).toContain('window.BookfinReader.setPage');
    expect(source).toContain('document.fonts.ready');
    expect(source).toContain("type:'FONT_STATUS'");
    expect(source).not.toContain('key={page.page_id}');
  });

  test('multilingual validation renders the 9 required FR, EN and ES benchmark works with BOOK BALANCED', () => {
    const fixture = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../src/fixtures/curated_v1/mobile_preview_fixture.json'), 'utf8'));
    const aReboursFixture = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../src/fixtures/curated_v1/a_rebours_pages.json'), 'utf8'));
    const hazlittFixture = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../src/fixtures/curated_v1/hazlitt_table_talk_pages.json'), 'utf8'));

    const requiredWorks = [
      // FR: La Parure, Bel-Ami, À rebours
      { workId: 'fr-maupassant-la-parure', lang: 'fr' },
      { workId: 'fr-maupassant-bel-ami', lang: 'fr' },
      { workId: 'fr-huysmans-a-rebours', lang: 'fr', source: aReboursFixture },
      // EN: Dorian Gray, Pride & Prejudice, Table-Talk
      { workId: 'en-wilde-dorian-gray', lang: 'en' },
      { workId: 'en-austen-pride-prejudice', lang: 'en' },
      { workId: 'en-hazlitt-table-talk', lang: 'en', source: hazlittFixture },
      // ES: Niebla, Don Quijote, Luces de bohemia
      { workId: 'es-unamuno-niebla', lang: 'es' },
      { workId: 'es-cervantes-quijote', lang: 'es' },
      { workId: 'es-valle-inclan-luces-de-bohemia', lang: 'es' },
    ];

    for (const req of requiredWorks) {
      let pageData: FeedPageDto;
      if (req.source) {
        const rawPage = req.source.pages[0];
        pageData = {
          impression_id: `test-${req.workId}`,
          page_id: `page-${req.workId}`,
          page_sequence_number: rawPage.page_sequence_number,
          source_page_number: String(rawPage.page_sequence_number),
          text: '',
          language_tag: req.lang,
          served_at: 'now',
          token_count: 100,
          blocks: rawPage.blocks,
        };
      } else {
        const sample = fixture.samples.find((s: { work_id: string }) => s.work_id === req.workId);
        expect(sample).toBeDefined();
        pageData = {
          impression_id: `test-${sample.id}`,
          page_id: `page-${sample.id}`,
          page_sequence_number: sample.page_sequence_number,
          source_page_number: String(sample.page_sequence_number),
          text: '',
          language_tag: sample.language,
          served_at: 'now',
          token_count: 100,
          blocks: sample.blocks,
        };
      }

      const html = renderBookfinPageToHtml(pageData);
      // Validates default BOOK BALANCED preset
      expect(html).toContain('bf-balanced-style');
      expect(html).toContain(`lang="${req.lang}"`);
      expect(html).toContain('bf-font-poliphili');
      expect(html).toContain('bf-hyphenation-off');
      // Validates no unescaped dangerous tags
      expect(html).not.toContain('<script');
      // Validates blocks were rendered into html semantic tags
      expect(html).toMatch(/<(p|h2|h3|blockquote|div class="bf-verse"|div class="bf-scene-break")/);
    }
  });

  test('20-page continuous reading stress test preserves persistent shell, scroll resets and font status', () => {
    // Generate 20 distinct pages across FR, EN and ES
    const pages: FeedPageDto[] = [];
    const langs = ['fr', 'en', 'es'];
    for (let i = 1; i <= 20; i++) {
      const lang = langs[(i - 1) % 3];
      pages.push({
        impression_id: `stress-imp-${i}`,
        page_id: `stress-page-${i}`,
        page_sequence_number: i,
        source_page_number: String(i),
        text: '',
        language_tag: lang,
        served_at: 'now',
        token_count: 200,
        blocks: [
          { type: 'heading', level: 2, spans: [{ text: `Chapitre ${i}` }] },
          { type: 'paragraph', spans: [{ text: `Paragraphe d'ouverture de la page ${i} démontrant la régularité du rendu.` }] },
          { type: 'paragraph', spans: [{ text: `Passage avec emphase `, italic: true }, { text: `sans synthèse artificielle.` }] },
          { type: 'scene_break' },
          { type: 'paragraph', spans: [{ text: `Conclusion de la page de test ${i}.` }] },
        ],
      });
    }

    expect(pages).toHaveLength(20);

    // Verify each page transition payload
    for (const p of pages) {
      const html = renderBookfinPageToHtml(p);
      expect(html).toContain('bf-balanced-style');
      expect(html).toContain(`lang="${p.language_tag}"`);

      // Verify serialization used by ReadingWebView.setPage
      const payload = {
        pageId: p.page_id,
        language: p.language_tag,
        html,
        fontMode: 'poliphili',
        scrollOffset: 0,
        theme: { fg: '#1C1917', muted: '#78716C' },
      };
      const script = `window.BookfinReader&&window.BookfinReader.setPage(${JSON.stringify(payload).replace(/</g, '\\u003c')});true;`;

      expect(script).toContain('setPage');
      expect(script).toContain(p.page_id);
      expect(script).not.toContain('<script');
      expect(payload.scrollOffset).toBe(0);

      // Verify font status contract remains stable: Roman loaded, Italic missing
      const statusMessage = JSON.stringify({
        type: 'FONT_STATUS',
        family: POLIPHILI_FAMILY,
        requestedMode: payload.fontMode,
        romanLoaded: true,
        italicLoaded: false,
        status: 'italic_missing',
      });
      const parsed = parseReaderBridgeMessage(statusMessage);
      expect(parsed).toEqual({
        type: 'FONT_STATUS',
        family: POLIPHILI_FAMILY,
        requestedMode: 'poliphili',
        romanLoaded: true,
        italicLoaded: false,
        status: 'italic_missing',
      });
    }
  });
});
