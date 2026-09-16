//! Read-only Railway database inventory for the Curated V1 import plan.
//!
//! It deliberately refuses localhost and Neon endpoints and requires an
//! explicit, one-shot environment marker. It issues SELECT statements only
//! and never prints DATABASE_URL or any credential.

use sqlx::{postgres::PgPoolOptions, Row};

const AUDIT_MARKER: &str = "bookfin-curated-v1-readonly-audit";

fn railway_url_is_permitted(url: &str) -> bool {
    if std::env::var("BOOKFIN_RAILWAY_AUDIT").ok().as_deref() != Some(AUDIT_MARKER) {
        return false;
    }
    let direct_railway = !url.contains("@localhost:")
        && !url.contains("@127.0.0.1:")
        && !url.contains("@::1:")
        && !url.contains(".neon.tech")
        && url.contains("railway");
    let explicitly_marked_ssh_tunnel = (url.contains("@localhost:") || url.contains("@127.0.0.1:"))
        && std::env::var("BOOKFIN_RAILWAY_TUNNEL_CONFIRMATION")
            .ok()
            .as_deref()
            == Some("postgres-production-ssh-tunnel");
    direct_railway || explicitly_marked_ssh_tunnel
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Railway injects DATABASE_URL for private-network consumers and, on the
    // Postgres service, DATABASE_PUBLIC_URL when a public proxy is enabled.
    // Prefer the latter for a local, read-only audit; neither value is logged.
    let url = std::env::var("DATABASE_PUBLIC_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| "DATABASE_PUBLIC_URL or DATABASE_URL required")?;
    if !railway_url_is_permitted(&url) {
        return Err("refuses any target except an explicitly marked Railway database".into());
    }

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let identity = sqlx::query("SELECT current_database() AS database, version() AS version")
        .fetch_one(&pool)
        .await?;
    println!("database={}", identity.get::<String, _>("database"));
    println!("postgres_version={}", identity.get::<String, _>("version"));

    let migrations: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await?;
    println!("migrations={migrations:?}");

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(&pool)
    .await?;
    println!("tables={tables:?}");

    let content_v2_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'pages' AND column_name = 'content_v2')",
    )
    .fetch_one(&pool)
    .await?;
    println!("pages.content_v2_present={content_v2_exists}");

    for table in [
        "users",
        "works",
        "editions",
        "pages",
        "extracts",
        "page_impressions",
        "extract_impressions",
        "reactions",
        "user_language_preferences",
        "taste_profiles",
        "user_affinities",
        "user_pair_affinities",
    ] {
        if tables.iter().any(|found| found == table) {
            let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
                .fetch_one(&pool)
                .await?;
            println!("count.{table}={count}");
        }
    }

    if tables.iter().any(|found| found == "pages") {
        let active_pages: i64 = sqlx::query_scalar("SELECT count(*) FROM pages WHERE is_active = true")
            .fetch_one(&pool)
            .await?;
        let languages: Vec<(String, i64)> = sqlx::query_as(
            "SELECT language_tag, count(*) FROM pages GROUP BY language_tag ORDER BY language_tag",
        )
        .fetch_all(&pool)
        .await?;
        let versions: Vec<(i32, bool, i64)> = sqlx::query_as(
            "SELECT version, is_active, count(*) FROM pages GROUP BY version, is_active ORDER BY version, is_active",
        )
        .fetch_all(&pool)
        .await?;
        let v2_non_null: i64 = if content_v2_exists {
            sqlx::query_scalar("SELECT count(*) FROM pages WHERE content_v2 IS NOT NULL")
                .fetch_one(&pool)
                .await?
        } else {
            0
        };
        println!("count.pages_active={active_pages}");
        println!("page_languages={languages:?}");
        println!("page_versions_active={versions:?}");
        println!("pages.content_v2_non_null={v2_non_null}");
    }

    if ["works", "editions", "pages"]
        .iter()
        .all(|required| tables.iter().any(|found| found == required))
    {
        let editions: Vec<(String, String, bool, i64, i64, i64)> = sqlx::query_as(
            "SELECT COALESCE(e.source_name, '(none)'), COALESCE(e.provenance, '(none)'), e.is_active, count(DISTINCT w.id), count(DISTINCT e.id), count(p.id) FROM editions e JOIN works w ON w.id = e.work_id LEFT JOIN pages p ON p.edition_id = e.id GROUP BY 1, 2, 3 ORDER BY 1, 2, 3",
        )
        .fetch_all(&pool)
        .await?;
        println!(
            "edition_inventory(source,provenance,is_active,works,editions,pages)={editions:?}"
        );
    }

    println!("READ_ONLY_AUDIT_COMPLETE");
    Ok(())
}
