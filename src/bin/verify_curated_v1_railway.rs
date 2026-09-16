//! Post-import verifier for the explicitly confirmed Railway Curated V1 run.
//!
//! This is read-only. It verifies the 73 manifest work IDs through their
//! deterministic UUIDv5 identities, because those source slugs are not stored
//! as a database column.

use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use uuid::Uuid;

const MANIFEST: &str = "corpus/curated_v1/manifest.json";
const NAMESPACE: Uuid = Uuid::from_u128(0xe3b7724e72fe54cfa53fcbc1d6f4f6d0);
const VERIFY_CONFIRMATION: &str = "verify-curated-v1-production";

#[derive(Deserialize)]
struct Manifest {
    total_real_pages: usize,
    works: Vec<ManifestWork>,
}

#[derive(Deserialize)]
struct ManifestWork {
    work_id: String,
    language: String,
}

#[derive(Deserialize)]
struct PagesFile {
    pages: Vec<PageHash>,
}

#[derive(Deserialize)]
struct PageHash {
    page_sequence_number: i32,
    content_hash: String,
}

fn stable_work_id(work_id: &str) -> Uuid {
    Uuid::new_v5(&NAMESPACE, format!("work:{work_id}:v1").as_bytes())
}

fn stable_page_id(work_id: &str, sequence: i32) -> Uuid {
    Uuid::new_v5(
        &NAMESPACE,
        format!("page:{work_id}:{sequence}:2").as_bytes(),
    )
}

fn permitted(url: &str) -> bool {
    std::env::var("BOOKFIN_RAILWAY_TARGET").ok().as_deref() == Some("production")
        && std::env::var("BOOKFIN_RAILWAY_VERIFY_CONFIRMATION")
            .ok()
            .as_deref()
            == Some(VERIFY_CONFIRMATION)
        && url.contains("railway")
        && !url.contains("@localhost:")
        && !url.contains("@127.0.0.1:")
        && !url.contains("@::1:")
        && !url.contains(".neon.tech")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL required")?;
    if !permitted(&url) {
        return Err("refuses any target except explicitly confirmed Railway production".into());
    }
    let manifest: Manifest = serde_json::from_str(&fs::read_to_string(MANIFEST)?)?;
    if manifest.works.len() != 73 || manifest.total_real_pages != 12025 {
        return Err("unexpected local Curated V1 manifest".into());
    }
    let expected: BTreeSet<Uuid> = manifest
        .works
        .iter()
        .map(|work| stable_work_id(&work.work_id))
        .collect();
    let mut expected_hashes = BTreeMap::new();
    for work in &manifest.works {
        let path = format!(
            "corpus/curated_v1/pages/{}/{}_pages.json",
            work.language, work.work_id
        );
        let pages: PagesFile = serde_json::from_str(&fs::read_to_string(path)?)?;
        for page in pages.pages {
            expected_hashes.insert(
                stable_page_id(&work.work_id, page.page_sequence_number),
                page.content_hash,
            );
        }
    }
    if expected_hashes.len() != 12025 {
        return Err("unexpected number of local Curated V1 page hashes".into());
    }
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let actual: BTreeSet<Uuid> = sqlx::query_scalar(
        "SELECT DISTINCT w.id FROM works w JOIN editions e ON e.work_id = w.id WHERE e.source_name = 'Bookfin Curated V1'",
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .collect();
    let pages: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1'",
    )
    .fetch_one(&pool)
    .await?;
    let null_v2: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1' AND p.content_v2 IS NULL",
    )
    .fetch_one(&pool)
    .await?;
    let empty: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1' AND btrim(p.content) = ''",
    )
    .fetch_one(&pool)
    .await?;
    let sequence_breaks: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT p.edition_id, p.page_number, row_number() OVER (PARTITION BY p.edition_id ORDER BY p.page_number) AS n FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1') numbered WHERE page_number <> n",
    )
    .fetch_one(&pool)
    .await?;
    let duplicate_pages: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT p.edition_id, p.page_number FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1' GROUP BY p.edition_id, p.page_number HAVING count(*) > 1) duplicates",
    )
    .fetch_one(&pool)
    .await?;
    let languages: Vec<(String, i64)> = sqlx::query_as(
        "SELECT p.language_tag, count(DISTINCT p.edition_id) FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1' GROUP BY p.language_tag ORDER BY p.language_tag",
    )
    .fetch_all(&pool)
    .await?;
    let active_non_curated: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pages p JOIN editions e ON e.id = p.edition_id WHERE p.is_active = true AND e.source_name <> 'Bookfin Curated V1'",
    )
    .fetch_one(&pool)
    .await?;
    let actual_hashes: BTreeMap<Uuid, (String, Option<String>)> = sqlx::query_as(
        "SELECT p.id, p.content_hash, p.canonical_content_hash FROM pages p JOIN editions e ON e.id = p.edition_id WHERE e.source_name = 'Bookfin Curated V1'",
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .collect();
    let hashes_match = actual_hashes.len() == expected_hashes.len()
        && expected_hashes.iter().all(|(id, expected_hash)| {
            actual_hashes
                .get(id)
                .is_some_and(|(content_hash, canonical_hash)| {
                    content_hash == expected_hash
                        && canonical_hash.as_deref() == Some(expected_hash)
                })
        });

    if actual != expected
        || pages != 12025
        || null_v2 != 0
        || empty != 0
        || sequence_breaks != 0
        || duplicate_pages != 0
        || languages != vec![("en".into(), 26), ("es".into(), 21), ("fr".into(), 26)]
        || active_non_curated != 0
        || !hashes_match
    {
        return Err(format!(
            "verification failed: works={} pages={pages} null_v2={null_v2} empty={empty} sequence_breaks={sequence_breaks} duplicate_pages={duplicate_pages} languages={languages:?} active_non_curated={active_non_curated} hashes_match={hashes_match}",
            actual.len()
        )
        .into());
    }
    println!("RAILWAY CURATED V1 VERIFY PASS works=73 pages=12025 languages=en:26,fr:26,es:21");
    Ok(())
}
