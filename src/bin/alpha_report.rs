use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct TesterRow {
    #[allow(dead_code)]
    user_id: Uuid,
    code: String,
    note: Option<String>,
    app_version: Option<String>,
    build_version: Option<String>,
    sessions_count: i64,
    pages_served: i64,
    pages_rated: i64,
    likes: i64,
    dislikes: i64,
    continued_pages: i64,
    max_continuation_depth: i32,
    reached_end_count: i64,
    resumed_books: i64,
    total_reading_ms: i64,
    avg_reading_ms: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    println!("================================================================================");
    println!("                      RAPPORT D'OBSERVATION ALPHA BOOKFIN                       ");
    println!("================================================================================");

    let rows = sqlx::query_as::<_, TesterRow>(
        r#"
        SELECT
            aci.user_id,
            aic.code,
            aic.note,
            COALESCE(
                (SELECT pi.app_version FROM page_impressions pi WHERE pi.user_id = aci.user_id AND pi.app_version IS NOT NULL ORDER BY pi.served_at DESC LIMIT 1),
                aci.app_version
            ) AS app_version,
            COALESCE(
                (SELECT pi.build_version FROM page_impressions pi WHERE pi.user_id = aci.user_id AND pi.build_version IS NOT NULL ORDER BY pi.served_at DESC LIMIT 1),
                aci.build_version
            ) AS build_version,
            COALESCE(COUNT(DISTINCT pi.session_id), 0) AS sessions_count,
            COALESCE(COUNT(DISTINCT pi.id), 0) AS pages_served,
            COALESCE(COUNT(DISTINCT r.id), 0) AS pages_rated,
            COALESCE(COUNT(DISTINCT CASE WHEN r.reaction_type = 'like' THEN r.id END), 0) AS likes,
            COALESCE(COUNT(DISTINCT CASE WHEN r.reaction_type = 'dislike' THEN r.id END), 0) AS dislikes,
            COALESCE(COUNT(DISTINCT CASE WHEN pi.navigation_action = 'continue_book' THEN pi.id END), 0) AS continued_pages,
            COALESCE(MAX(pi.continuation_depth), 0) AS max_continuation_depth,
            COALESCE(COUNT(DISTINCT CASE WHEN pi.reached_end = true THEN pi.id END), 0) AS reached_end_count,
            COALESCE(COUNT(DISTINCT CASE WHEN pi.book_resumed = true THEN pi.id END), 0) AS resumed_books,
            COALESCE(SUM(r.reading_time_ms), 0) AS total_reading_ms,
            COALESCE(AVG(r.reading_time_ms), 0)::BIGINT AS avg_reading_ms
        FROM alpha_claimed_invites aci
        JOIN alpha_invite_codes aic ON aci.code_id = aic.id
        LEFT JOIN page_impressions pi ON pi.user_id = aci.user_id
        LEFT JOIN reactions r ON r.user_id = aci.user_id
        GROUP BY aci.user_id, aic.code, aic.note, aci.app_version, aci.build_version
        ORDER BY aic.code ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        println!("Aucun lecteur alpha n'a encore validé d'invitation.");
        return Ok(());
    }

    println!(
        "{:<10} {:<15} {:<8} {:<6} {:<8} {:<6} {:<6} {:<9} {:<6} {:<6} {:<6} {:<8} {:<10} {:<8}",
        "CODE", "LECTEUR", "VERSION", "SESS.", "SERVIES", "ÉVAL.", "LIKES", "DISLIKES", "CONT.", "PROF.", "FIN", "REPRISES", "DURÉE TOT.", "MOY/PAGE"
    );
    println!("{:-<125}", "");

    let mut tot_served = 0;
    let mut tot_rated = 0;
    let mut tot_likes = 0;
    let mut tot_dislikes = 0;
    let mut tot_continued = 0;
    let mut tot_reached_end = 0;
    let mut tot_resumed = 0;
    let mut tot_time_ms = 0;

    for r in &rows {
        tot_served += r.pages_served;
        tot_rated += r.pages_rated;
        tot_likes += r.likes;
        tot_dislikes += r.dislikes;
        tot_continued += r.continued_pages;
        tot_reached_end += r.reached_end_count;
        tot_resumed += r.resumed_books;
        tot_time_ms += r.total_reading_ms;

        let version_str = format!(
            "{}/{}",
            r.app_version.as_deref().unwrap_or("?"),
            r.build_version.as_deref().unwrap_or("?")
        );

        let total_time_str = format!("{}m {:02}s", r.total_reading_ms / 60000, (r.total_reading_ms % 60000) / 1000);
        let avg_time_str = format!("{:.1}s", (r.avg_reading_ms as f64) / 1000.0);

        println!(
            "{:<10} {:<15} {:<8} {:<6} {:<8} {:<6} {:<6} {:<9} {:<6} {:<6} {:<6} {:<8} {:<10} {:<8}",
            r.code,
            r.note.as_deref().unwrap_or("Anonyme"),
            version_str,
            r.sessions_count,
            r.pages_served,
            r.pages_rated,
            r.likes,
            r.dislikes,
            r.continued_pages,
            r.max_continuation_depth,
            r.reached_end_count,
            r.resumed_books,
            total_time_str,
            avg_time_str
        );
    }

    println!("{:-<125}", "");
    println!(
        "TOTAUX : {} lecteurs | {} pages servies | {} évaluées ({} likes, {} dislikes) | {} continuations | {} reprises | {} fins d'œuvres | {}m lecture totale",
        rows.len(),
        tot_served,
        tot_rated,
        tot_likes,
        tot_dislikes,
        tot_continued,
        tot_resumed,
        tot_reached_end,
        tot_time_ms / 60000
    );
    println!("================================================================================");

    Ok(())
}
