import fs from 'fs';
import path from 'path';
import { usesJustifiedReadingLayout } from '../src/components/ReadingContent';

describe('Corpus V2 iPhone preview preparation', () => {
  const root = path.resolve(__dirname, '../..');

  test('the curated fixture remains the real multilingual source', () => {
    const fixture = JSON.parse(fs.readFileSync(path.join(root, 'corpus/curated_v1/mobile_preview_fixture.json'), 'utf8'));
    const workIds = fixture.samples.map((sample: { work_id: string }) => sample.work_id);

    expect(fixture.samples).toHaveLength(8);
    expect(workIds).toEqual(expect.arrayContaining([
      'en-austen-pride-prejudice',
      'fr-diderot-jacques-le-fataliste',
      'fr-maupassant-bel-ami',
      'es-unamuno-niebla',
      'es-valle-inclan-luces-de-bohemia',
    ]));
  });

  test('the V2 renderer preserves blocks and nested spans rather than flattening them', () => {
    const source = fs.readFileSync(path.join(root, 'mobile/src/components/ReadingContent.tsx'), 'utf8');

    expect(source).toContain('page.blocks.map');
    expect(source).toContain('<BookfinSpans spans={block.spans} />');
    expect(source).toContain("block.type === 'scene_break'");
    expect(source).toContain("block.type === 'verse'");
    expect(source).toContain('span.italic');
    expect(source).toContain('span.bold');
    expect(fs.readFileSync(path.join(root, 'mobile/src/lib/dev/curatedPreviewFixture.ts'), 'utf8')).toContain('page-en-hazlitt-table-talk-verse');
  });

  test('EN, FR and ES prose use the iOS justification experiment', () => {
    expect(usesJustifiedReadingLayout('en')).toBe(true);
    expect(usesJustifiedReadingLayout('fr')).toBe(true);
    expect(usesJustifiedReadingLayout('es')).toBe(true);
  });

  test('the preview has no production API calls and keeps all REVIEW works visible', () => {
    const preview = fs.readFileSync(path.join(root, 'mobile/src/screens/CorpusPreviewScreen.tsx'), 'utf8');
    const samples = fs.readFileSync(path.join(root, 'mobile/src/lib/dev/curatedPreviewFixture.ts'), 'utf8');

    expect(preview).not.toMatch(/readingApi|fetch\(/);
    expect(samples).toContain('review-fr-barbey-les-diaboliques');
    expect(samples).toContain('review-es-unamuno-niebla');
    expect(samples).toContain('review-es-valle-inclan-luces-de-bohemia');
  });

  test('Metro imports only local mobile fixtures, never the repository corpus', () => {
    const samples = fs.readFileSync(path.join(root, 'mobile/src/lib/dev/curatedPreviewFixture.ts'), 'utf8');

    expect(samples).not.toMatch(/from\s+['"][^'"]*\.\.\/\.\.\/\.\.\/\.\.\/corpus/);
    expect(samples).toContain("../../fixtures/curated_v1/mobile_preview_fixture.json");
    expect(samples).toContain("../../fixtures/curated_v1/niebla_pages.json");
  });

  test('generated page fixtures contain one unmodified real V2 page each', () => {
    const cases: Array<[string, string, number]> = [
      ['turn_of_the_screw_pages.json', 'corpus/curated_v1/pages/en/en-james-turn-of-the-screw_pages.json', 5],
      ['hazlitt_table_talk_pages.json', 'corpus/curated_v1/pages/en/en-hazlitt-table-talk_pages.json', 4],
      ['a_rebours_pages.json', 'corpus/curated_v1/pages/fr/fr-huysmans-a-rebours_pages.json', 1],
      ['les_diaboliques_pages.json', 'corpus/curated_v1/pages/fr/fr-barbey-les-diaboliques_pages.json', 347],
      ['niebla_pages.json', 'corpus/curated_v1/pages/es/es-unamuno-niebla_pages.json', 1],
      ['luces_de_bohemia_pages.json', 'corpus/curated_v1/pages/es/es-valle-inclan-luces-de-bohemia_pages.json', 2],
    ];

    for (const [fixtureName, sourceName, pageNumber] of cases) {
      const fixture = JSON.parse(fs.readFileSync(path.join(root, 'mobile/src/fixtures/curated_v1', fixtureName), 'utf8'));
      const source = JSON.parse(fs.readFileSync(path.join(root, sourceName), 'utf8'));
      const sourcePage = source.pages.find((page: { page_sequence_number: number }) => page.page_sequence_number === pageNumber);

      expect(fixture.pages_count).toBe(1);
      expect(fixture.source_page_sequence_number).toBe(pageNumber);
      expect(fixture.pages[0]).toEqual(sourcePage);
    }
  });
});
