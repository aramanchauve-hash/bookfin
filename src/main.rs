#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use axum::extract::Extension;
    use axum::Router;
    use bookfin::infrastructure::db::{init_db_pool, run_migrations};
    use bookfin::web::app::{shell, App};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::ServeDir;

    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,bookfin=debug".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://bookfin:bookfin@localhost:5432/bookfin".to_string());

    log!("Connexion à PostgreSQL sur {}", database_url);
    let pool = init_db_pool(&database_url).await?;

    log!("Exécution des migrations SQLx...");
    run_migrations(&pool).await?;
    log!("Migrations terminées avec succès.");

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    for (path, method) in server_fn::axum::server_fn_paths() {
        log!("Registered server function: {} {:?}", path, method);
    }

    let site_root = leptos_options.site_root.clone();
    let pkg_path = format!("{}/pkg", site_root);
    let pkg_service = ServeDir::new(pkg_path);

    let v1_router = bookfin::web::api_v1::create_v1_router();
    let cors = tower_http::cors::CorsLayer::permissive();

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || {
                    provide_context(pool.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .nest("/api/v1", v1_router)
        .nest_service("/pkg", pkg_service)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(cors)
        .layer(Extension(pool))
        .with_state(leptos_options);

    let bind_addr = std::env::var("BIND_ADDR")
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or(addr);

    log!("Démarrage du serveur Bookfin sur http://{}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Non-SSR fallback (client-side only target)
}
