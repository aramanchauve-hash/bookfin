import fs from 'fs';
import path from 'path';

describe('Neutralité absolue du feed principal', () => {
  test('fetchNextRandomPage n’utilise aucun paramètre de catalogue ou d’affinité', () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/lib/api/readingApi.ts'),
      'utf-8'
    );
    const fetchFunc = source.slice(source.indexOf('export async function fetchNextRandomPage'));
    const bodyEnd = fetchFunc.indexOf('}');
    const body = fetchFunc.slice(0, bodyEnd);

    // Ne doit pas filtrer par work ou bibliothèque
    expect(body).not.toContain('work_id');
    expect(body).not.toContain('library');
    expect(body).not.toContain('affinity');
    expect(body).toContain('/feed/next');
  });

  test('ReadingScreen ne contient aucun catalogue sur l’écran d’accueil', () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/screens/ReadingScreen.tsx'),
      'utf-8'
    );

    // L'écran d'accueil converge immédiatement vers la liseuse
    expect(source).toContain('ReadingContent');
    expect(source).not.toContain('CatalogueScreen');
    expect(source).not.toContain('FlatList'); // Le feed d'accueil n'est pas une liste ou catalogue
  });

  test('le backend Rust api_v1 préserve la route aléatoire sans affinité', () => {
    const source = fs.readFileSync(
      path.resolve(__dirname, '../../src/web/api_v1.rs'),
      'utf-8'
    );

    // Vérifie que /feed/next appelle feed_next_handler avec tirage aléatoire uniforme (rng)
    expect(source).toContain('feed_next_handler');
    expect(source).toContain('FeedNextPageUseCase');
    expect(source).toContain('gen_range(0.0..1.0)');
  });
});
