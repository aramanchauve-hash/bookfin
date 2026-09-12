use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct InviteRow {
    code: String,
    max_uses: i32,
    uses_count: i32,
    is_active: bool,
    note: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let cli_db_url = args
        .iter()
        .position(|a| a == "--database-url")
        .and_then(|idx| args.get(idx + 1).cloned());

    let database_url = cli_db_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| {
            dotenv().ok();
            env::var("DATABASE_URL").expect("DATABASE_URL must be set")
        });

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "create" => {
            let mut code = None;
            let mut max_uses = 1;
            let mut note = None;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--code" if i + 1 < args.len() => {
                        code = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--max-uses" if i + 1 < args.len() => {
                        max_uses = args[i + 1].parse().unwrap_or(1);
                        i += 2;
                    }
                    "--note" if i + 1 < args.len() => {
                        note = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            let code = code.unwrap_or_else(|| {
                let random_suffix: String = Uuid::new_v4().to_string()[..6].to_uppercase();
                format!("ALPHA-{}", random_suffix)
            });

            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO alpha_invite_codes (id, code, max_uses, uses_count, is_active, note)
                VALUES ($1, $2, $3, 0, true, $4)
                ON CONFLICT (code) DO NOTHING
                "#,
            )
            .bind(id)
            .bind(&code)
            .bind(max_uses)
            .bind(&note)
            .execute(&pool)
            .await?;

            println!("Code d'invitation créé avec succès :");
            println!("  Code      : {}", code);
            println!("  Max uses  : {}", max_uses);
            println!("  Note      : {}", note.as_deref().unwrap_or("-"));
        }
        "list" => {
            let rows = sqlx::query_as::<_, InviteRow>(
                r#"
                SELECT code, max_uses, uses_count, is_active, note
                FROM alpha_invite_codes
                ORDER BY created_at ASC
                "#,
            )
            .fetch_all(&pool)
            .await?;

            println!("{:<15} {:<10} {:<10} {:<8} {:<25}", "CODE", "UTILISÉ", "MAX", "ACTIF", "NOTE");
            println!("{:-<70}", "");
            for r in rows {
                println!(
                    "{:<15} {:<10} {:<10} {:<8} {:<25}",
                    r.code,
                    r.uses_count,
                    r.max_uses,
                    if r.is_active { "OUI" } else { "NON" },
                    r.note.as_deref().unwrap_or("-")
                );
            }
        }
        "revoke" => {
            let mut code = None;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--code" && i + 1 < args.len() {
                    code = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }

            if let Some(code) = code {
                let rows = sqlx::query(
                    r#"
                    UPDATE alpha_invite_codes
                    SET is_active = false
                    WHERE code = $1
                    "#,
                )
                .bind(&code)
                .execute(&pool)
                .await?
                .rows_affected();

                if rows > 0 {
                    println!("Code {} révoqué.", code);
                } else {
                    println!("Code {} introuvable.", code);
                }
            } else {
                eprintln!("Usage: alpha_invite revoke --code <code>");
            }
        }
        _ => print_usage(),
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: alpha_invite <commande> [options]");
    println!("Commandes :");
    println!("  create --code <CODE> [--max-uses 1] [--note <NOTE>]");
    println!("  list");
    println!("  revoke --code <CODE>");
}
