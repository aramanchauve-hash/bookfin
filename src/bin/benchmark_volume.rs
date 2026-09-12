use std::time::Instant;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());

    println!("=== BOOKFIN SYNTHETIC VOLUME BENCHMARK ===");
    println!("Connecting to PostgreSQL at {}...", db_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Creating isolated schema 'bench_temp'...");
    sqlx::query("DROP SCHEMA IF EXISTS bench_temp CASCADE").execute(&pool).await?;
    sqlx::query("CREATE SCHEMA bench_temp").execute(&pool).await?;

    let ddl_statements = [
        r#"CREATE TABLE bench_temp.users (
            id UUID PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE bench_temp.works (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT NOT NULL,
            original_language_tag TEXT NOT NULL
        )"#,
        r#"CREATE TABLE bench_temp.editions (
            id UUID PRIMARY KEY,
            work_id UUID NOT NULL,
            edition_title TEXT NOT NULL,
            language_tag TEXT NOT NULL
        )"#,
        r#"CREATE TABLE bench_temp.pages (
            id UUID PRIMARY KEY,
            edition_id UUID NOT NULL,
            page_number INTEGER NOT NULL,
            content TEXT NOT NULL,
            content_hash CHAR(64) NOT NULL,
            language_tag TEXT NOT NULL,
            token_count INTEGER NOT NULL,
            random_key DOUBLE PRECISION NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT true,
            version INTEGER NOT NULL DEFAULT 1,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            source_page_number TEXT
        )"#,
        r#"CREATE INDEX idx_pages_random_lang ON bench_temp.pages (language_tag, is_active, random_key)"#,
        r#"CREATE UNIQUE INDEX uq_edition_page_active ON bench_temp.pages (edition_id, page_number) WHERE is_active = true"#,
        r#"CREATE TABLE bench_temp.page_impressions (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            page_id UUID NOT NULL,
            served_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            trigger TEXT NOT NULL,
            parent_impression_id UUID,
            client_request_id UUID
        )"#,
        r#"CREATE INDEX idx_page_impressions_user_page ON bench_temp.page_impressions (user_id, page_id)"#,
        r#"CREATE INDEX idx_page_impressions_served_at ON bench_temp.page_impressions (served_at)"#,
        r#"CREATE INDEX idx_page_impressions_page_id ON bench_temp.page_impressions (page_id)"#,
        r#"CREATE TABLE bench_temp.reactions (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            page_id UUID NOT NULL,
            reaction_type TEXT NOT NULL,
            reaction_action TEXT,
            dwell_time_ms INTEGER NOT NULL,
            scroll_depth_percent REAL NOT NULL,
            reading_speed_wpm INTEGER NOT NULL,
            event_id UUID NOT NULL,
            impression_id UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE INDEX idx_reactions_user_page ON bench_temp.reactions (user_id, page_id)"#,
        r#"CREATE INDEX idx_reactions_page_id ON bench_temp.reactions (page_id)"#,
        r#"CREATE INDEX idx_reactions_user_id ON bench_temp.reactions (user_id)"#,
        r#"CREATE UNIQUE INDEX uq_reactions_impression_id ON bench_temp.reactions (impression_id) WHERE impression_id IS NOT NULL"#,
    ];

    for stmt in ddl_statements {
        sqlx::query(stmt).execute(&pool).await?;
    }

    println!("[1/4] Generating 100 users and 500 editions...");
    let t0 = Instant::now();
    sqlx::query("INSERT INTO bench_temp.users (id) SELECT gen_random_uuid() FROM generate_series(1, 100)").execute(&pool).await?;
    sqlx::query("INSERT INTO bench_temp.works (id, title, author, original_language_tag) SELECT gen_random_uuid(), 'Oeuvre ' || i, 'Auteur ' || (i % 50), 'fr' FROM generate_series(1, 500) i").execute(&pool).await?;
    sqlx::query("INSERT INTO bench_temp.editions (id, work_id, edition_title, language_tag) SELECT id, id, title || ' - Edition Standard', 'fr' FROM bench_temp.works").execute(&pool).await?;
    println!("      Done in {:.2?}", t0.elapsed());

    println!("[2/4] Generating 100,000 synthetic pages (200 pages per edition)...");
    let t1 = Instant::now();
    sqlx::query(
        r#"
        WITH numbered_editions AS (
            SELECT id AS edition_id, row_number() OVER () AS ed_num
            FROM bench_temp.editions
        )
        INSERT INTO bench_temp.pages (
            id, edition_id, page_number, content, content_hash, language_tag,
            token_count, random_key, is_active, version, created_at, source_page_number
        )
        SELECT
            gen_random_uuid(),
            ne.edition_id,
            p.page_num,
            'Texte de la page ' || p.page_num || ' pour l’édition ' || ne.ed_num || '. Le train filait dans la nuit froide.',
            md5(ne.ed_num::text || '-' || p.page_num::text) || md5(p.page_num::text || '-' || ne.ed_num::text),
            'fr',
            180,
            ((((ne.ed_num - 1) * 200 + (p.page_num - 1))::double precision) / 100000.0),
            true,
            1,
            NOW(),
            p.page_num::text
        FROM numbered_editions ne
        CROSS JOIN generate_series(1, 200) p(page_num);
        "#
    ).execute(&pool).await?;
    println!("      Done in {:.2?}", t1.elapsed());

    println!("[3/4] Generating 1,000,000 synthetic impressions (10,000 impressions per user)...");
    let t2 = Instant::now();
    sqlx::query(
        r#"
        WITH user_list AS (
            SELECT id AS u_id, row_number() OVER () AS u_idx FROM bench_temp.users
        ),
        page_sample AS (
            SELECT id AS p_id, row_number() OVER () AS p_idx FROM bench_temp.pages
        )
        INSERT INTO bench_temp.page_impressions (
            id, user_id, page_id, served_at, trigger
        )
        SELECT
            gen_random_uuid(),
            u.u_id,
            ps.p_id,
            NOW() - ((s.seq || ' seconds')::interval),
            CASE WHEN s.seq % 5 = 0 THEN 'continue' ELSE 'random' END
        FROM user_list u
        CROSS JOIN generate_series(1, 10000) s(seq)
        JOIN page_sample ps ON ps.p_idx = (( (u.u_idx * 997 + s.seq * 31) % 100000 ) + 1);
        "#
    ).execute(&pool).await?;
    println!("      Done in {:.2?}", t2.elapsed());

    println!("[4/4] Generating 200,000 synthetic reactions from impressions...");
    let t3 = Instant::now();
    sqlx::query(
        r#"
        INSERT INTO bench_temp.reactions (
            id, user_id, page_id, reaction_type, reaction_action,
            dwell_time_ms, scroll_depth_percent, reading_speed_wpm,
            event_id, impression_id, created_at
        )
        SELECT
            gen_random_uuid(),
            pi.user_id,
            pi.page_id,
            CASE WHEN (row_number() OVER ()) % 2 = 0 THEN 'like' ELSE 'dislike' END,
            CASE WHEN (row_number() OVER ()) % 3 = 0 THEN 'continue_book' ELSE 'random_page' END,
            15000 + ((row_number() OVER ()) % 20000)::integer,
            85.0 + (((row_number() OVER ()) % 15)::real),
            220 + (((row_number() OVER ()) % 60)::integer),
            gen_random_uuid(),
            pi.id,
            pi.served_at + interval '20 seconds'
        FROM (
            SELECT id, user_id, page_id, served_at
            FROM bench_temp.page_impressions
            LIMIT 200000
        ) pi;
        "#
    ).execute(&pool).await?;
    println!("      Done in {:.2?}", t3.elapsed());

    println!("Running ANALYZE to update PostgreSQL optimizer statistics...");
    sqlx::query("ANALYZE bench_temp.pages").execute(&pool).await?;
    sqlx::query("ANALYZE bench_temp.page_impressions").execute(&pool).await?;
    sqlx::query("ANALYZE bench_temp.reactions").execute(&pool).await?;

    // Retrieve a sample test user and edition
    let sample_user: Uuid = sqlx::query_scalar("SELECT id FROM bench_temp.users LIMIT 1").fetch_one(&pool).await?;
    let sample_edition: Uuid = sqlx::query_scalar("SELECT id FROM bench_temp.editions LIMIT 1").fetch_one(&pool).await?;

    println!("\n==================================================");
    println!("QUERY 1: RANDOM_PAGE Unseen Selection (NOT EXISTS Anti-Join)");
    println!("==================================================");
    let q1_sql = format!(
        r#"
        EXPLAIN (ANALYZE, BUFFERS)
        SELECT id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key, is_active, version, created_at, source_page_number
        FROM bench_temp.pages p
        WHERE p.language_tag = 'fr' AND p.is_active = true
          AND p.random_key >= 0.42
          AND NOT EXISTS (
              SELECT 1 FROM bench_temp.page_impressions pi
              WHERE pi.user_id = '{}' AND pi.page_id = p.id
          )
        ORDER BY p.random_key ASC
        LIMIT 1;
        "#,
        sample_user
    );
    let rows = sqlx::query(&q1_sql).fetch_all(&pool).await?;
    for row in rows {
        let line: String = row.get(0);
        println!("{}", line);
    }

    println!("\n==================================================");
    println!("QUERY 2: CONTINUE_BOOK Next Page Lookup in Edition");
    println!("==================================================");
    let q2_sql = format!(
        r#"
        EXPLAIN (ANALYZE, BUFFERS)
        SELECT id, edition_id, page_number, content, content_hash, language_tag, token_count, random_key, is_active, version, created_at, source_page_number
        FROM bench_temp.pages
        WHERE edition_id = '{}' AND page_number = 42 AND is_active = true
        LIMIT 1;
        "#,
        sample_edition
    );
    let rows = sqlx::query(&q2_sql).fetch_all(&pool).await?;
    for row in rows {
        let line: String = row.get(0);
        println!("{}", line);
    }

    println!("\n==================================================");
    println!("QUERY 3: Single Page Impression Insertion Latency");
    println!("==================================================");
    let sample_page: Uuid = sqlx::query_scalar("SELECT id FROM bench_temp.pages LIMIT 1").fetch_one(&pool).await?;
    let q3_sql = format!(
        r#"
        EXPLAIN (ANALYZE, BUFFERS)
        INSERT INTO bench_temp.page_impressions (id, user_id, page_id, served_at, trigger)
        VALUES ('{}', '{}', '{}', NOW(), 'random');
        "#,
        Uuid::new_v4(),
        sample_user,
        sample_page
    );
    let rows = sqlx::query(&q3_sql).fetch_all(&pool).await?;
    for row in rows {
        let line: String = row.get(0);
        println!("{}", line);
    }

    println!("\n==================================================");
    println!("QUERY 4: Single Reaction Insertion Latency");
    println!("==================================================");
    let q4_sql = format!(
        r#"
        EXPLAIN (ANALYZE, BUFFERS)
        INSERT INTO bench_temp.reactions (
            id, user_id, page_id, reaction_type, reaction_action,
            dwell_time_ms, scroll_depth_percent, reading_speed_wpm,
            event_id, impression_id, created_at
        ) VALUES (
            '{}', '{}', '{}', 'like', 'continue_book',
            22000, 95.0, 240, '{}', '{}', NOW()
        );
        "#,
        Uuid::new_v4(),
        sample_user,
        sample_page,
        Uuid::new_v4(),
        Uuid::new_v4()
    );
    let rows = sqlx::query(&q4_sql).fetch_all(&pool).await?;
    for row in rows {
        let line: String = row.get(0);
        println!("{}", line);
    }

    println!("\n==================================================");
    println!("QUERY 5: Inverted Index Overlap / Affinity Candidate Aggregation");
    println!("==================================================");
    let q5_sql = format!(
        r#"
        EXPLAIN (ANALYZE, BUFFERS)
        SELECT r2.user_id, count(*) as overlap_count
        FROM bench_temp.reactions r1
        JOIN bench_temp.reactions r2 ON r1.page_id = r2.page_id AND r1.user_id <> r2.user_id
        WHERE r1.user_id = '{}'
        GROUP BY r2.user_id
        HAVING count(*) >= 3
        ORDER BY overlap_count DESC
        LIMIT 50;
        "#,
        sample_user
    );
    let rows = sqlx::query(&q5_sql).fetch_all(&pool).await?;
    for row in rows {
        let line: String = row.get(0);
        println!("{}", line);
    }

    println!("\n==================================================");
    println!("CLEANUP: Dropping isolated 'bench_temp' schema...");
    sqlx::query("DROP SCHEMA bench_temp CASCADE").execute(&pool).await?;
    println!("Cleanup completed. Zero persistent footprint.");
    println!("==================================================");

    Ok(())
}
