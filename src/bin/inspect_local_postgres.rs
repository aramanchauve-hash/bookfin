//! Read-only inventory of the PostgreSQL connection supplied in DATABASE_URL.

use sqlx::{
    postgres::{PgConnection, PgPoolOptions},
    Connection, Row,
};

const DRY_RUN_DB: &str = "bookfin_curated_v1_dry_run";

fn local_url(url: &str) -> bool {
    url.contains("@localhost:") || url.contains("@127.0.0.1:") || url.contains("@::1:")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL required")?;
    if !local_url(&url) || url.contains("railway") || url.contains("neon") {
        return Err("inspection refuses non-local database URLs".into());
    }
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let identity = sqlx::query(
        "SELECT current_database() AS database, current_user AS role, version() AS version",
    )
    .fetch_one(&pool)
    .await?;
    let migrations: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let latest: Option<i64> = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap_or(None);
    let works: Option<i64> = sqlx::query_scalar("SELECT count(*) FROM works")
        .fetch_one(&pool)
        .await
        .ok();
    let pages: Option<i64> = sqlx::query_scalar("SELECT count(*) FROM pages")
        .fetch_one(&pool)
        .await
        .ok();
    println!("database={}", identity.get::<String, _>("database"));
    println!("role={}", identity.get::<String, _>("role"));
    println!("version={}", identity.get::<String, _>("version"));
    println!("migrations_applied={migrations}; latest={latest:?}");
    println!("works={works:?}; pages={pages:?}");

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.iter().any(|arg| arg == "--create-dry-run")
        && arguments.iter().any(|arg| arg == "--drop-dry-run")
    {
        return Err("choose either --create-dry-run or --drop-dry-run".into());
    }

    if arguments.iter().any(|arg| arg == "--create-dry-run") {
        if identity.get::<String, _>("database") != "bookfin" {
            return Err(
                "creation requires inspecting the known existing bookfin database first".into(),
            );
        }
        let (prefix, _) = url.rsplit_once('/').ok_or("invalid local DATABASE_URL")?;
        let maintenance_url = format!("{prefix}/postgres");
        let mut connection = PgConnection::connect(&maintenance_url).await?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(DRY_RUN_DB)
                .fetch_one(&mut connection)
                .await?;
        if exists {
            return Err(format!("refusing to reuse existing {DRY_RUN_DB}").into());
        }
        sqlx::query("CREATE DATABASE bookfin_curated_v1_dry_run")
            .execute(&mut connection)
            .await?;
        connection.close().await?;
        println!("created_isolated_database={DRY_RUN_DB}");
    }

    if arguments.iter().any(|arg| arg == "--drop-dry-run") {
        if identity.get::<String, _>("database") != "bookfin" {
            return Err(
                "cleanup requires inspecting the known existing bookfin database first".into(),
            );
        }
        let (prefix, _) = url.rsplit_once('/').ok_or("invalid local DATABASE_URL")?;
        let maintenance_url = format!("{prefix}/postgres");
        let mut connection = PgConnection::connect(&maintenance_url).await?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(DRY_RUN_DB)
                .fetch_one(&mut connection)
                .await?;
        if !exists {
            return Err(format!("refusing cleanup: {DRY_RUN_DB} does not exist").into());
        }
        sqlx::query("DROP DATABASE bookfin_curated_v1_dry_run")
            .execute(&mut connection)
            .await?;
        connection.close().await?;
        println!("dropped_isolated_database={DRY_RUN_DB}");
    }
    Ok(())
}
