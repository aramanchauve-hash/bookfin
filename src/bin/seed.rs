use sqlx::PgPool;
use uuid::Uuid;

const FIXED_TEST_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

struct SeedBook {
    title: &'static str,
    author: &'static str,
    original_language_tag: &'static str,
    publication_year: Option<i32>,
    extracts: &'static [&'static str],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());

    println!("==================================================");
    println!("  Bookfin - Peuplement de développement (Seed)");
    println!("==================================================");
    println!("Connexion à : {}", database_url);

    let pool = PgPool::connect(&database_url).await?;

    // 1. Utilisateurs fixes de développement
    let user_ids = [
        Uuid::parse_str(FIXED_TEST_USER_ID)?,
        Uuid::parse_str("11111111-1111-1111-1111-111111111111")?,
    ];

    for uid in user_ids {
        sqlx::query(
            r#"
            INSERT INTO users (id, created_at)
            VALUES ($1, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(uid)
        .execute(&pool)
        .await?;

        // 2. Préférences linguistiques ordonnées
        let prefs = [("fr", 1), ("en", 2), ("es", 3), ("de", 4), ("ja", 5)];
        for (lang, priority) in prefs {
            sqlx::query(
                r#"
                INSERT INTO user_language_preferences (user_id, language_tag, priority, created_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (user_id, language_tag) DO NOTHING
                "#,
            )
            .bind(uid)
            .bind(lang)
            .bind(priority)
            .execute(&pool)
            .await?;
        }
    }
    println!("✓ Utilisateurs fixes configurés avec préférences linguistiques.");

    // 3. Corpus de livres et d'extraits multilingues (25 extraits au total)
    let books: &[SeedBook] = &[
        // Anglais (majoritaire)
        SeedBook {
            title: "The Whispering Pines",
            author: "Eleanor Vance",
            original_language_tag: "en",
            publication_year: Some(1924),
            extracts: &[
                "The fog drifted between the tall pines like ancient breath, curling around the stone stairs that led nowhere.",
                "A single lantern flickered in the watchtower, trembling against the cold mountain wind.",
            ],
        },
        SeedBook {
            title: "Shadows over Greenwich",
            author: "Arthur Pendelton",
            original_language_tag: "en",
            publication_year: Some(1888),
            extracts: &[
                "Clocks in London struck eleven in discordant chorus, muffled by the heavy yellow river mist.",
                "He traced the watermark on the parchment, noticing the subtle seal of the Admiralty.",
            ],
        },
        SeedBook {
            title: "The Silicon Horizon",
            author: "Mira Thorne",
            original_language_tag: "en",
            publication_year: Some(2018),
            extracts: &[
                "In the quiet desert night, data centers humming with synthetic thought cast amber halos toward Orion.",
            ],
        },
        SeedBook {
            title: "Letters to the Sea",
            author: "Julian Croft",
            original_language_tag: "en",
            publication_year: Some(1952),
            extracts: &[
                "The tide retreated, abandoning fragments of green glass polished smoother than emeralds.",
                "Every sailor carries two maps: one printed on oilcloth, the other etched in sleepless memory.",
            ],
        },
        SeedBook {
            title: "Echoes of the North",
            author: "Bridget O'Connor",
            original_language_tag: "en",
            publication_year: Some(1976),
            extracts: &[
                "The peat fire crackled, warming hands that had spent decades hauling nets across freezing waters.",
            ],
        },
        SeedBook {
            title: "The Clockmaker of Prague",
            author: "Karel Vaneck",
            original_language_tag: "en",
            publication_year: Some(1935),
            extracts: &[
                "Behind the silver gears, tiny brass dancers waited for the strike of noon to begin their perpetual waltz.",
                "Escapements clicked like heartbeat cadences in the dimly lit workshop.",
            ],
        },
        SeedBook {
            title: "Dust and Constellations",
            author: "Sarah K. Adams",
            original_language_tag: "en",
            publication_year: Some(2004),
            extracts: &[
                "Telescopes pointed at the zenith revealed galaxies spinning like silver spirals in bottomless ink.",
                "We measure light that began traveling when mountains were still seabed.",
            ],
        },
        SeedBook {
            title: "Winter at Heron Hall",
            author: "Thomas Sterling",
            original_language_tag: "en",
            publication_year: Some(1912),
            extracts: &[
                "Snow sealed the tall windows in lace patterns of ice, silencing the grand corridors.",
                "Footsteps vanished under the falling snow within mere minutes.",
            ],
        },
        // Français (plusieurs extraits)
        SeedBook {
            title: "Les Chemins d'Automne",
            author: "Camille Delacroix",
            original_language_tag: "fr",
            publication_year: Some(1968),
            extracts: &[
                "Les feuilles rousses tourbillonnaient sur les pavés humides, emportant avec elles l'odeur sucrée du raisin fermenté.",
                "Au loin, le clocher du village égrenait les heures avec une lenteur rassurante.",
            ],
        },
        SeedBook {
            title: "L'Ombre du Marais",
            author: "Henri de Saint-Pol",
            original_language_tag: "fr",
            publication_year: Some(1895),
            extracts: &[
                "Une barque glissait sans bruit entre les roseaux denses, fendant l'eau noire comme un miroir liquide.",
                "Le héron immobile veillait sur l'aurore naissante, statue de plumes argentées.",
            ],
        },
        SeedBook {
            title: "Le Dernier Phare",
            author: "Élise Marceau",
            original_language_tag: "fr",
            publication_year: Some(1984),
            extracts: &[
                "La tempête frappait la falaise de granit, projetant des gerbes d'écume blanche jusqu'au sommet de la tour.",
            ],
        },
        // Espagnol
        SeedBook {
            title: "El Viento del Sur",
            author: "Mateo Silva",
            original_language_tag: "es",
            publication_year: Some(1973),
            extracts: &[
                "El viento soplaba caliente a través de los olivares, levantando polvo dorado entre las colinas.",
                "Las campanas sonaban a siesta, mientras los patios descansaban a la sombra del jazmín.",
            ],
        },
        // Allemand
        SeedBook {
            title: "Stimmen im Wald",
            author: "Lukas Weber",
            original_language_tag: "de",
            publication_year: Some(1981),
            extracts: &[
                "Im dichten Tannenwald brach die Abenddämmerung an, und der Moosboden dämpfte jeden Schritt.",
                "Ein kalter Bach murmelte zwischen bemoosten Granitblöcken dem Tal entgegen.",
            ],
        },
        // Écriture non latine (Japonais)
        SeedBook {
            title: "雪の庭 (Snow Garden)",
            author: "Kenji Sato",
            original_language_tag: "ja",
            publication_year: Some(1962),
            extracts: &[
                "雪は静かに庭の石灯籠の上に降り積もり、夜の静寂をいっそう深くしていた。",
            ],
        },
        // Écriture non latine (Grec)
        SeedBook {
            title: "Το Φως του Αιγαίου (The Light of the Aegean)",
            author: "Elena Pappas",
            original_language_tag: "el",
            publication_year: Some(1999),
            extracts: &[
                "Ο ήλιος έλουζε τα λευκά σπίτια του νησιού με ένα φως τόσο δυνατό που πονούσαν τα μάτια.",
            ],
        },
    ];

    let mut total_extracts = 0;
    let mut current_idx = 0;
    let total_count = 25.0;

    for book in books {
        let work_id = Uuid::new_v4();
        let edition_id = Uuid::new_v4();

        // 1. Inserer l'œuvre (work)
        sqlx::query(
            r#"
            INSERT INTO works (id, title, author, original_language_tag, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(work_id)
        .bind(book.title)
        .bind(book.author)
        .bind(book.original_language_tag)
        .execute(&pool)
        .await?;

        // 2. Inserer l'édition
        sqlx::query(
            r#"
            INSERT INTO editions (id, work_id, edition_title, translator, publication_year, source_name, language_tag, created_at)
            VALUES ($1, $2, $3, NULL, $4, 'Bookfin Curated Classical Library', $5, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(edition_id)
        .bind(work_id)
        .bind(book.title)
        .bind(book.publication_year)
        .bind(book.original_language_tag)
        .execute(&pool)
        .await?;

        // Compatibilite retroactif books
        sqlx::query(
            r#"
            INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(work_id)
        .bind(book.title)
        .bind(book.author)
        .bind(book.original_language_tag)
        .bind(book.publication_year)
        .execute(&pool)
        .await?;

        for (page_num, &extract_text) in (1..).zip(book.extracts.iter()) {
            let page_id = Uuid::new_v4();
            let token_count = extract_text.split_whitespace().count() as i32;
            let random_key = (current_idx as f64 + 0.5) / total_count;
            let content_hash = bookfin::domain::models::Page::compute_hash(extract_text);
            current_idx += 1;

            // 3. Inserer la page avec son hash SHA-256 stable
            sqlx::query(
                r#"
                INSERT INTO pages (id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key, is_active, version, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE, 1, NOW())
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(page_id)
            .bind(edition_id)
            .bind(page_num)
            .bind(extract_text)
            .bind(&content_hash)
            .bind(book.original_language_tag)
            .bind(token_count)
            .bind(random_key)
            .execute(&pool)
            .await?;

            // Compatibilite retroactif extracts
            sqlx::query(
                r#"
                INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(page_id)
            .bind(work_id)
            .bind(extract_text)
            .bind(book.original_language_tag)
            .bind(token_count)
            .bind(random_key)
            .execute(&pool)
            .await?;

            total_extracts += 1;
        }
    }

    println!(
        "✓ {} œuvres/éditions et {} pages littéraires insérées (avec hash SHA-256 stable).",
        books.len(),
        total_extracts
    );
    println!("✓ Clés random_key distribuées équitablement entre 0.0 et 1.0.");
    println!("==================================================");
    println!("  Seed terminé avec succès !");
    println!("==================================================");

    Ok(())
}
