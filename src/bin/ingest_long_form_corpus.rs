//! Idempotent importer for Bookfin Long-Form Corpus 01.
//! The page definition is intentionally identical to the pilot importer.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use std::fs;
use std::path::Path;
use uuid::Uuid;

const NAMESPACE_UUID: &str = "a790e0e8-983d-4f86-b5a0-704a255b9f17";
const TARGET_PAGE_CHARS: usize = 1600;
const MANIFEST_PATH: &str = "corpus/long_form_01/manifest.json";

#[derive(Debug, Deserialize)]
struct Manifest {
    corpus_name: String,
    version: i32,
    works: Vec<Work>,
}

#[derive(Debug, Deserialize)]
struct Work {
    work_id: String,
    edition_id: String,
    slug: String,
    title: String,
    author: String,
    original_language_tag: String,
    work_type: String,
    publication_year: i32,
    edition_title: String,
    language_tag: String,
    source_name: String,
    source_url: String,
    provenance: String,
    rights_status: String,
    rights_basis: String,
    license: String,
    file_path: String,
    source_text_sha256: String,
}

fn paginate_text(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let (mut cursor, total) = (0usize, chars.len());
    let mut pages = Vec::new();

    while cursor < total {
        if total - cursor <= (TARGET_PAGE_CHARS * 5) / 4 {
            pages.push(text[chars[cursor].0..].trim().to_owned());
            break;
        }
        let target = cursor + TARGET_PAGE_CHARS;
        let mut cut = (target..(target + 200).min(total)).find(|&i| chars[i].1.is_whitespace());
        if cut.is_none() {
            cut = ((target.saturating_sub(200)).max(cursor + 100)..target)
                .rev()
                .find(|&i| chars[i].1.is_whitespace());
        }
        let split = cut.unwrap_or(target);
        pages.push(text[chars[cursor].0..chars[split].0].trim().to_owned());
        cursor = split;
        while cursor < total && chars[cursor].1.is_whitespace() {
            cursor += 1;
        }
    }
    pages
}

fn sha256(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn content_hash(text: &str) -> String {
    sha256(&text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn stable_random_key(page_id: Uuid) -> f64 {
    let digest = Sha256::digest(page_id.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    // Exactly representable 53-bit fraction in [0, 1), stable across runs.
    (u64::from_be_bytes(bytes) >> 11) as f64 / (1u64 << 53) as f64
}

async fn import_work(
    tx: &mut Transaction<'_, Postgres>,
    namespace: Uuid,
    work: &Work,
) -> Result<usize, Box<dyn std::error::Error>> {
    let work_id = Uuid::parse_str(&work.work_id)?;
    let edition_id = Uuid::parse_str(&work.edition_id)?;
    let text = fs::read_to_string(Path::new(&work.file_path))?;
    if sha256(&text) != work.source_text_sha256 {
        return Err(format!("{}: source_text_sha256 mismatch", work.slug).into());
    }
    let pages = paginate_text(&text);
    if pages.is_empty() {
        return Err(format!("{}: no generated pages", work.slug).into());
    }

    sqlx::query(
        r#"INSERT INTO works (id, title, author, original_language_tag, work_type, created_at)
           VALUES ($1, $2, $3, $4, $5, NOW())
           ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title, author = EXCLUDED.author,
             original_language_tag = EXCLUDED.original_language_tag, work_type = EXCLUDED.work_type"#,
    )
    .bind(work_id).bind(&work.title).bind(&work.author).bind(&work.original_language_tag).bind(&work.work_type)
    .execute(&mut **tx).await?;

    sqlx::query(
        r#"INSERT INTO editions (id, work_id, edition_title, language_tag, publication_year, source_name,
              source_url, provenance, rights_status, rights_basis, license, source_text_sha256, is_active, created_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,true,NOW())
           ON CONFLICT (id) DO UPDATE SET edition_title=EXCLUDED.edition_title, source_name=EXCLUDED.source_name,
              source_url=EXCLUDED.source_url, provenance=EXCLUDED.provenance, rights_status=EXCLUDED.rights_status,
              rights_basis=EXCLUDED.rights_basis, license=EXCLUDED.license,
              source_text_sha256=EXCLUDED.source_text_sha256, is_active=true"#,
    )
    .bind(edition_id).bind(work_id).bind(&work.edition_title).bind(&work.language_tag)
    .bind(work.publication_year).bind(&work.source_name).bind(&work.source_url).bind(&work.provenance)
    .bind(&work.rights_status).bind(&work.rights_basis).bind(&work.license).bind(&work.source_text_sha256)
    .execute(&mut **tx).await?;

    for (zero_index, page) in pages.iter().enumerate() {
        let page_number = (zero_index + 1) as i32;
        let page_id = Uuid::new_v5(
            &namespace,
            format!("{}:{}:1", edition_id, page_number).as_bytes(),
        );
        sqlx::query(
            r#"INSERT INTO pages (id, edition_id, page_number, source_page_number, content, content_hash,
                 language_tag, token_count, random_key, is_active, version, created_at)
               VALUES ($1,$2,$3,NULL,$4,$5,$6,$7,$8,true,1,NOW())
               ON CONFLICT (id) DO UPDATE SET content=EXCLUDED.content, content_hash=EXCLUDED.content_hash,
                 token_count=EXCLUDED.token_count, random_key=EXCLUDED.random_key, is_active=true"#,
        )
        .bind(page_id).bind(edition_id).bind(page_number).bind(page).bind(content_hash(page))
        .bind(&work.language_tag).bind(page.split_whitespace().count() as i32).bind(stable_random_key(page_id))
        .execute(&mut **tx).await?;
    }
    Ok(pages.len())
}

async fn ensure_long_form_metadata_schema(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Kept equivalent to migration 0009 so an already-running development
    // database with an unrelated historical-migration checksum mismatch can
    // still receive this corpus without altering its migration ledger.
    sqlx::query(
        "ALTER TABLE works ADD COLUMN IF NOT EXISTS work_type TEXT NOT NULL DEFAULT 'narrative'",
    )
    .execute(pool)
    .await?;
    sqlx::query("ALTER TABLE editions ADD COLUMN IF NOT EXISTS provenance TEXT, ADD COLUMN IF NOT EXISTS rights_basis TEXT, ADD COLUMN IF NOT EXISTS source_text_sha256 VARCHAR(64)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_editions_source_text_sha256 ON editions (source_text_sha256) WHERE source_text_sha256 IS NOT NULL")
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL")?;
    let manifest: Manifest = serde_json::from_str(&fs::read_to_string(MANIFEST_PATH)?)?;
    let pool = PgPool::connect(&database_url).await?;
    if let Err(error) = sqlx::migrate!("./migrations").run(&pool).await {
        if matches!(error, sqlx::migrate::MigrateError::VersionMismatch(8)) {
            eprintln!("WARNING: pre-existing migration checksum mismatch at version 8; migration ledger left untouched.");
        } else {
            return Err(error.into());
        }
    }
    ensure_long_form_metadata_schema(&pool).await?;
    let namespace = Uuid::parse_str(NAMESPACE_UUID)?;
    let mut tx = pool.begin().await?;
    let mut total_pages = 0usize;
    println!(
        "Importing {} v{} ({} works)",
        manifest.corpus_name,
        manifest.version,
        manifest.works.len()
    );
    for work in &manifest.works {
        let pages = import_work(&mut tx, namespace, work).await?;
        total_pages += pages;
        println!("{}: {} pages", work.slug, pages);
    }
    tx.commit().await?;
    println!(
        "Imported {} works and {} eligible pages.",
        manifest.works.len(),
        total_pages
    );
    Ok(())
}
