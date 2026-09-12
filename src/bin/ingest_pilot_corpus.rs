use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgConnectOptions, PgPool};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

const NAMESPACE_UUID: &str = "7b1981a4-6842-4dc8-a83a-867df3c965e9";
const TARGET_PAGE_CHARS: usize = 1600;
const LOCAL_DEV_DATABASE_URL: &str = "postgres://bookfin:bookfin@localhost:5432/bookfin";
const DATABASE_URL_REQUIRED_MESSAGE: &str =
    "DATABASE_URL must be set; use --allow-local-fallback only for explicit local development";
const REMOTE_RESET_REFUSED_MESSAGE: &str =
    "--reset is allowed only for a local PostgreSQL database (localhost, 127.0.0.1, or ::1)";

#[derive(Debug, Deserialize)]
struct PilotManifest {
    corpus_name: String,
    version: i32,
    stories: Vec<ManifestStory>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ManifestStory {
    work_id: String,
    edition_id: String,
    slug: String,
    title: String,
    author: String,
    original_language_tag: String,
    edition_title: String,
    language_tag: String,
    publication_year: Option<i32>,
    source_name: String,
    source_url: String,
    rights_status: String,
    license: String,
    file_path: String,
}

fn paginate_text(text: &str, target_chars: usize) -> Vec<String> {
    let mut pages = Vec::new();
    let text = text.trim();
    if text.is_empty() {
        return pages;
    }

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let total_chars = chars.len();
    let mut cursor = 0;

    while cursor < total_chars {
        // If remaining chars is within 1.25x of target, take all remaining to avoid a tiny trailing page
        if total_chars - cursor <= (target_chars * 5) / 4 {
            let start_byte = chars[cursor].0;
            let page_slice = &text[start_byte..];
            let trimmed = page_slice.trim();
            if !trimmed.is_empty() {
                pages.push(trimmed.to_string());
            }
            break;
        }

        let target_idx = cursor + target_chars;
        if target_idx >= total_chars {
            let start_byte = chars[cursor].0;
            let trimmed = text[start_byte..].trim();
            if !trimmed.is_empty() {
                pages.push(trimmed.to_string());
            }
            break;
        }

        // Search for nearest word boundary (whitespace) starting from target_idx
        let mut cut_idx = None;

        // Try searching forward first (up to +200)
        let max_fwd = (target_idx + 200).min(total_chars);
        for i in target_idx..max_fwd {
            if chars[i].1.is_whitespace() {
                cut_idx = Some(i);
                break;
            }
        }

        // If not found forward, search backward (down to target_idx - 200)
        if cut_idx.is_none() {
            let min_back = target_idx.saturating_sub(200).max(cursor + 100);
            for i in (min_back..target_idx).rev() {
                if chars[i].1.is_whitespace() {
                    cut_idx = Some(i);
                    break;
                }
            }
        }

        let split_idx = cut_idx.unwrap_or(target_idx);
        let start_byte = chars[cursor].0;
        let end_byte = chars[split_idx].0;
        let page_slice = &text[start_byte..end_byte];
        let trimmed = page_slice.trim();
        if !trimmed.is_empty() {
            pages.push(trimmed.to_string());
        }

        // Advance cursor past any whitespace after the split
        cursor = split_idx;
        while cursor < total_chars && chars[cursor].1.is_whitespace() {
            cursor += 1;
        }
    }

    pages
}

fn compute_content_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    // Normalize spaces for hash
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn resolve_database_url(
    database_url: Option<String>,
    allow_local_fallback: bool,
) -> Result<String, &'static str> {
    match database_url.filter(|url| !url.trim().is_empty()) {
        Some(url) => Ok(url),
        None if allow_local_fallback => Ok(LOCAL_DEV_DATABASE_URL.to_owned()),
        None => Err(DATABASE_URL_REQUIRED_MESSAGE),
    }
}

fn is_local_database_url(database_url: &str) -> bool {
    let Ok(options) = PgConnectOptions::from_str(database_url) else {
        return false;
    };

    matches!(
        options.get_host(),
        host if host.eq_ignore_ascii_case("localhost")
            || host == "127.0.0.1"
            || host == "::1"
            || host == "[::1]"
    )
}

fn validate_reset_target(database_url: &str) -> Result<(), &'static str> {
    if is_local_database_url(database_url) {
        Ok(())
    } else {
        Err(REMOTE_RESET_REFUSED_MESSAGE)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let allow_local_fallback = args.iter().any(|arg| arg == "--allow-local-fallback");
    let reset_requested = args.iter().any(|arg| arg == "--reset");
    let database_url =
        resolve_database_url(std::env::var("DATABASE_URL").ok(), allow_local_fallback)?;

    if reset_requested {
        validate_reset_target(&database_url)?;
    }

    println!("===============================================================================");
    println!("  Bookfin - Ingestion du Corpus Pilote de Nouvelles du Domaine Public");
    println!("===============================================================================");
    println!("Connexion a PostgreSQL configuree.");

    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|_| "Unable to connect to configured PostgreSQL database")?;

    println!("Application des migrations SQLx...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("Migrations appliquees avec succes.");

    if reset_requested {
        println!("Nettoyage des impressions et reactions de test en base (--reset)...");
        sqlx::query("TRUNCATE reactions, page_impressions CASCADE")
            .execute(&pool)
            .await?;
        println!("Tables nettoyees.");
    }

    let manifest_content = fs::read_to_string("corpus/pilot_short_stories/manifest.json")?;
    let manifest: PilotManifest = serde_json::from_str(&manifest_content)?;

    println!(
        "Manifeste chargé : {} (v{}), {} œuvres déclarées.\n",
        manifest.corpus_name,
        manifest.version,
        manifest.stories.len()
    );

    let namespace = Uuid::parse_str(NAMESPACE_UUID)?;
    let mut rng = rand::thread_rng();

    let mut total_pages_inserted = 0usize;
    let mut story_stats = Vec::new();

    for (idx, story) in manifest.stories.iter().enumerate() {
        let work_id = Uuid::parse_str(&story.work_id)?;
        let edition_id = Uuid::parse_str(&story.edition_id)?;

        // 1. Insert or Update Work
        sqlx::query(
            r#"
            INSERT INTO works (id, title, author, original_language_tag, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (id) DO UPDATE 
                SET title = EXCLUDED.title,
                    author = EXCLUDED.author,
                    original_language_tag = EXCLUDED.original_language_tag
            "#,
        )
        .bind(work_id)
        .bind(&story.title)
        .bind(&story.author)
        .bind(&story.original_language_tag)
        .execute(&pool)
        .await?;

        // 2. Insert or Update Edition
        sqlx::query(
            r#"
            INSERT INTO editions (
                id, work_id, edition_title, translator, language_tag,
                publication_year, publisher, source_name, source_url,
                rights_status, license, is_active, created_at
            )
            VALUES ($1, $2, $3, NULL, $4, $5, NULL, $6, $7, $8, $9, true, NOW())
            ON CONFLICT (id) DO UPDATE
                SET edition_title = EXCLUDED.edition_title,
                    source_name = EXCLUDED.source_name,
                    source_url = EXCLUDED.source_url,
                    is_active = true
            "#,
        )
        .bind(edition_id)
        .bind(work_id)
        .bind(&story.edition_title)
        .bind(&story.language_tag)
        .bind(story.publication_year)
        .bind(&story.source_name)
        .bind(&story.source_url)
        .bind(&story.rights_status)
        .bind(&story.license)
        .execute(&pool)
        .await?;

        // 3. Read Raw Story Text
        let file_path = Path::new(&story.file_path);
        let raw_text = fs::read_to_string(file_path)?;

        // 4. Paginate deterministically
        let pages = paginate_text(&raw_text, TARGET_PAGE_CHARS);
        let pages_count = pages.len();

        let mut page_chars: Vec<usize> = Vec::new();

        for (seq_0, page_content) in pages.iter().enumerate() {
            let page_sequence_number = (seq_0 + 1) as i32;
            let source_page_num = format!("p. {}", page_sequence_number);
            let content_hash = compute_content_hash(page_content);
            let token_count = page_content.split_whitespace().count() as i32;
            let random_key: f64 = rng.gen_range(0.0..1.0);

            // Deterministic UUID for page idempotence
            let page_name = format!("{}:{}", story.edition_id, page_sequence_number);
            let page_id = Uuid::new_v5(&namespace, page_name.as_bytes());

            page_chars.push(page_content.chars().count());

            sqlx::query(
                r#"
                INSERT INTO pages (
                    id, edition_id, page_number, source_page_number, content,
                    content_hash, language_tag, token_count, random_key,
                    is_active, version, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, 1, NOW())
                ON CONFLICT (id) DO UPDATE
                    SET content = EXCLUDED.content,
                        content_hash = EXCLUDED.content_hash,
                        token_count = EXCLUDED.token_count,
                        source_page_number = EXCLUDED.source_page_number,
                        is_active = true
                "#,
            )
            .bind(page_id)
            .bind(edition_id)
            .bind(page_sequence_number)
            .bind(source_page_num)
            .bind(page_content)
            .bind(content_hash)
            .bind(&story.language_tag)
            .bind(token_count)
            .bind(random_key)
            .execute(&pool)
            .await?;

            total_pages_inserted += 1;
        }

        let min_c = page_chars.iter().min().copied().unwrap_or(0);
        let max_c = page_chars.iter().max().copied().unwrap_or(0);
        let avg_c = if pages_count > 0 {
            page_chars.iter().sum::<usize>() / pages_count
        } else {
            0
        };

        story_stats.push((
            idx + 1,
            story.language_tag.clone(),
            story.title.clone(),
            story.author.clone(),
            pages_count,
            min_c,
            max_c,
            avg_c,
        ));
    }

    println!("------------------------------------------------------------------------------------------------------------------");
    println!(
        "{:<3} | {:<4} | {:<28} | {:<22} | {:<5} | {:<18}",
        "#", "Lang", "Titre", "Auteur", "Pages", "Caractères (min-max/m)"
    );
    println!("------------------------------------------------------------------------------------------------------------------");
    for (i, lang, title, author, pages, min_c, max_c, avg_c) in &story_stats {
        let stats_str = format!("{}-{} (m:{})", min_c, max_c, avg_c);
        let short_title = if title.chars().count() > 28 {
            format!(
                "{}…",
                &title[..title.char_indices().nth(27).map(|(i, _)| i).unwrap_or(28)]
            )
        } else {
            title.clone()
        };
        let short_author = if author.chars().count() > 22 {
            format!(
                "{}…",
                &author[..author.char_indices().nth(21).map(|(i, _)| i).unwrap_or(22)]
            )
        } else {
            author.clone()
        };
        println!(
            "{:<3} | {:<4} | {:<28} | {:<22} | {:<5} | {:<18}",
            i, lang, short_title, short_author, pages, stats_str
        );
    }
    println!("------------------------------------------------------------------------------------------------------------------");
    println!("✓ Ingestion terminée avec succès !");
    println!("✓ Nombre total d'œuvres : {}", manifest.stories.len());
    println!(
        "✓ Nombre total de pages éligibles insérées : {}",
        total_pages_inserted
    );
    println!("===============================================================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        is_local_database_url, resolve_database_url, validate_reset_target,
        DATABASE_URL_REQUIRED_MESSAGE, REMOTE_RESET_REFUSED_MESSAGE,
    };

    #[test]
    fn database_url_is_required_without_explicit_dev_fallback() {
        assert_eq!(
            resolve_database_url(None, false),
            Err(DATABASE_URL_REQUIRED_MESSAGE)
        );
    }

    #[test]
    fn explicit_dev_fallback_is_local_only() {
        let database_url =
            resolve_database_url(None, true).expect("explicit fallback should resolve");
        assert!(is_local_database_url(&database_url));
    }

    #[test]
    fn local_database_urls_are_recognized() {
        for database_url in [
            "postgres://bookfin@localhost/bookfin",
            "postgres://bookfin@127.0.0.1/bookfin",
            "postgres://bookfin@[::1]/bookfin",
        ] {
            assert!(is_local_database_url(database_url), "{database_url}");
        }
    }

    #[test]
    fn railway_and_other_remote_database_urls_are_not_local() {
        for database_url in [
            "postgres://bookfin@railway.internal/bookfin",
            "postgres://bookfin@bookfin-db.up.railway.app/bookfin",
            "postgres://bookfin@db.example.com/bookfin",
        ] {
            assert!(!is_local_database_url(database_url), "{database_url}");
        }
    }

    #[test]
    fn reset_refuses_remote_databases_without_leaking_the_url() {
        let secret_database_url = "postgres://bookfin:never-print-this@railway.internal/bookfin";
        let error =
            validate_reset_target(secret_database_url).expect_err("remote reset must be refused");

        assert_eq!(error, REMOTE_RESET_REFUSED_MESSAGE);
        assert!(!error.contains("never-print-this"));
        assert!(!error.contains(secret_database_url));
    }
}
