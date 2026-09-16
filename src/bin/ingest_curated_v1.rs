//! Curated V1 importer.  Default mode is a filesystem-only V2 round-trip
//! dry-run; apply modes are deliberately target-restricted.

use bookfin::domain::models::{BlockV2, PageContentV2};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{
    postgres::{PgConnection, PgPoolOptions},
    types::Json,
    Connection, Postgres, Transaction,
};
use std::{fs, path::Path};
use uuid::Uuid;

const MANIFEST: &str = "corpus/curated_v1/manifest.json";
const NAMESPACE: Uuid = Uuid::from_u128(0xe3b7724e72fe54cfa53fcbc1d6f4f6d0);
const RAILWAY_IMPORT_CONFIRMATION: &str = "replace-alpha-with-curated-v1";

#[derive(Deserialize)]
struct Manifest {
    total_real_pages: usize,
    works: Vec<ManifestWork>,
}
#[derive(Deserialize)]
struct ManifestWork {
    work_id: String,
    author: String,
    title: String,
    language: String,
    work_kind: String,
}
#[derive(Deserialize)]
struct Normalized {
    source_sha256: String,
    document_sha256: String,
}
#[derive(Deserialize)]
struct PagesFile {
    document_sha256: String,
    pages_count: usize,
    pages: Vec<SourcePage>,
}
#[derive(Deserialize)]
struct SourcePage {
    page_sequence_number: i32,
    content_hash: String,
    blocks: Vec<BlockV2>,
}

fn stable_id(kind: &str, work_id: &str, suffix: &str) -> Uuid {
    Uuid::new_v5(&NAMESPACE, format!("{kind}:{work_id}:{suffix}").as_bytes())
}
fn random_key(id: Uuid) -> f64 {
    let digest = Sha256::digest(id.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(bytes) >> 11) as f64 / (1_u64 << 53) as f64
}
fn local_database_url(url: &str) -> bool {
    (url.contains("@localhost:") || url.contains("@127.0.0.1:") || url.contains("@::1:"))
        && url
            .split('?')
            .next()
            .is_some_and(|base| base.ends_with("/bookfin_curated_v1_dry_run"))
}
fn explicitly_isolated_neon_url(url: &str) -> bool {
    std::env::var("BOOKFIN_ISOLATED_DB_PURPOSE").ok().as_deref()
        == Some("bookfin-curated-v1-dry-run")
        && url.contains(".neon.tech")
        && !url.contains("railway")
}
fn explicitly_confirmed_railway_url(url: &str) -> bool {
    std::env::var("BOOKFIN_RAILWAY_TARGET").ok().as_deref() == Some("production")
        && std::env::var("BOOKFIN_RAILWAY_IMPORT_CONFIRMATION")
            .ok()
            .as_deref()
            == Some(RAILWAY_IMPORT_CONFIRMATION)
        && url.contains("railway")
        && !url.contains("@localhost:")
        && !url.contains("@127.0.0.1:")
        && !url.contains("@::1:")
        && !url.contains(".neon.tech")
}

fn expected_active_alpha_pages() -> Result<i64, Box<dyn std::error::Error>> {
    let expected = std::env::var("BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES")
        .map_err(|_| "BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES required for Railway import")?
        .parse::<i64>()?;
    if expected <= 0 {
        return Err("BOOKFIN_EXPECTED_ACTIVE_ALPHA_PAGES must be positive".into());
    }
    Ok(expected)
}
fn read<T: for<'a> Deserialize<'a>>(
    path: impl AsRef<Path>,
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

async fn import_work(
    tx: &mut Transaction<'_, Postgres>,
    work: &ManifestWork,
    normalized: &Normalized,
    pages: &PagesFile,
) -> Result<usize, Box<dyn std::error::Error>> {
    let work_row = stable_id("work", &work.work_id, "v1");
    let edition_row = stable_id("edition", &work.work_id, "curated-v1");
    sqlx::query("INSERT INTO works (id,title,author,original_language_tag,work_type,created_at) VALUES ($1,$2,$3,$4,$5,NOW()) ON CONFLICT (id) DO UPDATE SET title=EXCLUDED.title,author=EXCLUDED.author,original_language_tag=EXCLUDED.original_language_tag,work_type=EXCLUDED.work_type")
        .bind(work_row).bind(&work.title).bind(&work.author).bind(&work.language).bind(&work.work_kind).execute(&mut **tx).await?;
    sqlx::query("INSERT INTO editions (id,work_id,edition_title,language_tag,source_name,provenance,rights_status,rights_basis,license,source_text_sha256,is_active,created_at) VALUES ($1,$2,$3,$4,'Bookfin Curated V1','Curated V1 source HTML','public_domain','public_domain','public_domain',$5,true,NOW()) ON CONFLICT (id) DO UPDATE SET edition_title=EXCLUDED.edition_title,source_text_sha256=EXCLUDED.source_text_sha256,is_active=true")
        .bind(edition_row).bind(work_row).bind(&work.title).bind(&work.language).bind(&normalized.source_sha256).execute(&mut **tx).await?;
    for source in &pages.pages {
        let content_v2 = PageContentV2 {
            blocks: source.blocks.clone(),
        };
        let api_round_trip: PageContentV2 =
            serde_json::from_value(serde_json::to_value(&content_v2)?)?;
        if api_round_trip != content_v2 {
            return Err(format!(
                "{} page {} V2 round-trip mismatch",
                work.work_id, source.page_sequence_number
            )
            .into());
        }
        let page_id = stable_id(
            "page",
            &work.work_id,
            &format!("{}:2", source.page_sequence_number),
        );
        let affected = sqlx::query("INSERT INTO pages (id,edition_id,page_number,content,content_v2,content_hash,canonical_content_hash,language_tag,token_count,random_key,is_active,version,created_at) VALUES ($1,$2,$3,$4,$5,$6,$6,$7,$8,$9,true,2,NOW()) ON CONFLICT (id) DO UPDATE SET content=EXCLUDED.content,content_v2=EXCLUDED.content_v2,content_hash=EXCLUDED.content_hash,canonical_content_hash=EXCLUDED.canonical_content_hash,token_count=EXCLUDED.token_count,random_key=EXCLUDED.random_key,is_active=true,version=2 WHERE pages.content_hash=EXCLUDED.content_hash AND pages.content_v2=EXCLUDED.content_v2")
            .bind(page_id).bind(edition_row).bind(source.page_sequence_number).bind(content_v2.legacy_text()).bind(Json(content_v2)).bind(&source.content_hash).bind(&work.language).bind(source.blocks.iter().flat_map(|b| &b.spans).map(|s| s.text.split_whitespace().count()).sum::<usize>() as i32).bind(random_key(page_id)).execute(&mut **tx).await?.rows_affected();
        if affected != 1 {
            return Err(format!(
                "{} page {} conflicts with a different immutable row",
                work.work_id, source.page_sequence_number
            )
            .into());
        }
    }
    Ok(pages.pages.len())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let apply_local = args.iter().any(|arg| arg == "--apply-local");
    let apply_isolated = args.iter().any(|arg| arg == "--apply-isolated");
    let apply_railway = args.iter().any(|arg| arg == "--apply-railway");
    if [apply_local, apply_isolated, apply_railway]
        .into_iter()
        .filter(|enabled| *enabled)
        .count()
        > 1
    {
        return Err("choose only one apply mode".into());
    }
    let manifest: Manifest = read(MANIFEST)?;
    if manifest.works.len() != 73 {
        return Err(format!("expected 73 works, found {}", manifest.works.len()).into());
    }
    let mut total = 0_usize;
    let mut validated = Vec::new();
    for work in &manifest.works {
        let normalized: Normalized = read(format!(
            "corpus/curated_v1/normalized/{}/{}.json",
            work.language, work.work_id
        ))?;
        let pages: PagesFile = read(format!(
            "corpus/curated_v1/pages/{}/{}_pages.json",
            work.language, work.work_id
        ))?;
        if pages.document_sha256 != normalized.document_sha256
            || pages.pages_count != pages.pages.len()
            || pages.pages.iter().enumerate().any(|(i, p)| {
                p.page_sequence_number != (i + 1) as i32 || p.content_hash.len() != 64
            })
        {
            return Err(format!("{} invalid canonical V2 pages", work.work_id).into());
        }
        for page in &pages.pages {
            let _: PageContentV2 =
                serde_json::from_value(serde_json::json!({"blocks": page.blocks}))?;
        }
        total += pages.pages.len();
        validated.push((work, normalized, pages));
    }
    if total != manifest.total_real_pages {
        return Err(format!(
            "expected {} pages, found {total}",
            manifest.total_real_pages
        )
        .into());
    }
    println!("Curated V1 V2 round-trip: 73 works / {total} pages");
    if !apply_local && !apply_isolated && !apply_railway {
        println!("DRY RUN ONLY: no database connection opened. Use --apply-local, --apply-isolated, or the explicitly confirmed --apply-railway mode.");
        return Ok(());
    }
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set for apply mode")?;
    if apply_local && !local_database_url(&database_url) {
        return Err(
            "--apply-local requires localhost/127.0.0.1 and database bookfin_curated_v1_dry_run"
                .into(),
        );
    }
    if apply_isolated && !explicitly_isolated_neon_url(&database_url) {
        return Err("--apply-isolated requires a Neon URL and BOOKFIN_ISOLATED_DB_PURPOSE=bookfin-curated-v1-dry-run".into());
    }
    if apply_railway && !explicitly_confirmed_railway_url(&database_url) {
        return Err("--apply-railway requires a Railway URL, BOOKFIN_RAILWAY_TARGET=production, and the explicit import confirmation".into());
    }
    let expected_alpha_pages = if apply_railway {
        Some(expected_active_alpha_pages()?)
    } else {
        None
    };
    // Neon may provision this throwaway compute with a single connection. Run
    // migrations in a dedicated pool and close it before opening the atomic
    // import transaction; this avoids the migration advisory-lock connection
    // starving the importer.
    println!("Applying migrations on isolated database...");
    let mut migration_connection = PgConnection::connect(&database_url).await?;
    sqlx::migrate!("./migrations")
        .run_direct(&mut migration_connection)
        .await?;
    migration_connection.close().await?;
    println!("Starting atomic Curated V1 import...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    let mut tx = pool.begin().await?;
    if let Some(expected) = expected_alpha_pages {
        let actual: i64 =
            sqlx::query_scalar("SELECT count(*) FROM pages WHERE version = 1 AND is_active = true")
                .fetch_one(&mut *tx)
                .await?;
        if actual != expected {
            return Err(format!("Railway alpha precondition failed: expected {expected} active V1 pages, found {actual}").into());
        }
    }
    let mut imported = 0;
    for (work, normalized, pages) in validated {
        imported += import_work(&mut tx, work, &normalized, &pages).await?;
    }
    if imported != total {
        return Err("import count mismatch".into());
    }
    if let Some(expected) = expected_alpha_pages {
        let deactivated = sqlx::query(
            "UPDATE pages SET is_active = false WHERE version = 1 AND is_active = true",
        )
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;
        if deactivated != expected {
            return Err(format!("Railway alpha replacement changed during transaction: expected {expected}, deactivated {deactivated}").into());
        }
        sqlx::query("UPDATE editions e SET is_active = false WHERE e.is_active = true AND NOT EXISTS (SELECT 1 FROM pages p WHERE p.edition_id = e.id AND p.is_active = true)")
            .execute(&mut *tx).await?;
        let active_v2: i64 = sqlx::query_scalar("SELECT count(*) FROM pages WHERE version = 2 AND is_active = true AND content_v2 IS NOT NULL")
            .fetch_one(&mut *tx).await?;
        let remaining_active_non_v2: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pages WHERE is_active = true AND version <> 2",
        )
        .fetch_one(&mut *tx)
        .await?;
        if active_v2 != 12025 || remaining_active_non_v2 != 0 {
            return Err(format!("Railway postcondition failed: active_v2={active_v2}, active_non_v2={remaining_active_non_v2}").into());
        }
    }
    tx.commit().await?;
    if apply_railway {
        println!(
            "RAILWAY IMPORT COMPLETE: 73 works / {imported} pages; alpha V1 pages deactivated"
        );
    } else {
        println!("LOCAL IMPORT COMPLETE: 73 works / {imported} pages");
    }
    Ok(())
}
