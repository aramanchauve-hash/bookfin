//! Read-only verification for the explicitly marked isolated Curated V1 DB.

use bookfin::domain::models::PageContentV2;
use sqlx::{postgres::PgPoolOptions, types::Json};

fn permitted(url: &str) -> bool {
    std::env::var("BOOKFIN_ISOLATED_DB_PURPOSE").ok().as_deref() == Some("bookfin-curated-v1-dry-run")
        && url.contains(".neon.tech") && !url.contains("railway")
}
fn permitted_local_dry_run(url: &str) -> bool {
    (url.contains("@localhost:") || url.contains("@127.0.0.1:") || url.contains("@::1:"))
        && url.split('?').next().is_some_and(|base| base.ends_with("/bookfin_curated_v1_dry_run"))
        && !url.contains("railway") && !url.contains("neon")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL required")?;
    if !permitted(&url) && !permitted_local_dry_run(&url) { return Err("refuses any DB except the marked Neon or exact local dry-run database".into()); }
    let pool = PgPoolOptions::new().max_connections(1).connect(&url).await?;
    let works: i64 = sqlx::query_scalar("SELECT count(*) FROM works WHERE work_type IN ('LONG_FORM','SHORT_WORK')").fetch_one(&pool).await?;
    let pages: i64 = sqlx::query_scalar("SELECT count(*) FROM pages WHERE version=2 AND canonical_content_hash IS NOT NULL").fetch_one(&pool).await?;
    let null_v2: i64 = sqlx::query_scalar("SELECT count(*) FROM pages WHERE version=2 AND content_v2 IS NULL").fetch_one(&pool).await?;
    let sequence_breaks: i64 = sqlx::query_scalar("SELECT count(*) FROM (SELECT edition_id, page_number, row_number() over (PARTITION BY edition_id ORDER BY page_number) AS n FROM pages WHERE version=2) s WHERE page_number <> n").fetch_one(&pool).await?;
    let languages: Vec<(String, i64)> = sqlx::query_as("SELECT language_tag, count(DISTINCT edition_id) FROM pages WHERE version=2 GROUP BY language_tag ORDER BY language_tag").fetch_all(&pool).await?;
    let duplicate_ids: i64 = sqlx::query_scalar("SELECT count(*) FROM (SELECT edition_id,page_number,count(*) FROM pages WHERE version=2 GROUP BY edition_id,page_number HAVING count(*) > 1) d").fetch_one(&pool).await?;
    let structured: Vec<(String, i64)> = sqlx::query_as("SELECT kind,count(*) FROM (SELECT 'heading'::text AS kind FROM pages WHERE version=2 AND content_v2 @> '{\"blocks\":[{\"type\":\"heading\"}]}' UNION ALL SELECT 'verse' FROM pages WHERE version=2 AND content_v2 @> '{\"blocks\":[{\"type\":\"verse\"}]}' UNION ALL SELECT 'blockquote' FROM pages WHERE version=2 AND content_v2 @> '{\"blocks\":[{\"type\":\"blockquote\"}]}' UNION ALL SELECT 'italic' FROM pages WHERE version=2 AND content_v2::text LIKE '%\"italic\":true%') q GROUP BY kind ORDER BY kind").fetch_all(&pool).await?;
    let sample: Json<PageContentV2> = sqlx::query_scalar("SELECT content_v2 FROM pages WHERE version=2 AND content_v2 @> '{\"blocks\":[{\"type\":\"verse\"}]}' LIMIT 1").fetch_one(&pool).await?;
    if works != 73 || pages != 12025 || null_v2 != 0 || sequence_breaks != 0 || duplicate_ids != 0 || sample.0.blocks.is_empty() { return Err(format!("verification failed: works={works} pages={pages} null_v2={null_v2} sequence_breaks={sequence_breaks} duplicate_ids={duplicate_ids}").into()); }
    println!("DB VERIFY PASS works={works} pages={pages} null_v2={null_v2} sequence_breaks={sequence_breaks} duplicate_ids={duplicate_ids}");
    println!("languages={languages:?}");
    println!("structured={structured:?}; sample_blocks={}", sample.0.blocks.len());
    Ok(())
}
