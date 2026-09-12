# Image Docker pour l'API Bookfin (Axum) uniquement.
# N'inclut pas l'UI Leptos ni les assets /pkg (voir railway.toml pour le
# contexte : Railway ne sert que /api/v1/* et /health pour l'alpha mobile).

# ---- Stage 1 : build ----
FROM rust:1.98.1-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# sqlx::migrate! lit ./migrations au moment de la compilation et embarque son
# contenu dans le binaire : aucun accès à une base de données n'est requis ici,
# et le dossier migrations/ n'a pas besoin d'être présent dans l'image finale.
RUN cargo build --release --bin bookfin

# ---- Stage 2 : runtime ----
FROM debian:bookworm-slim AS runtime

# ca-certificates : requis pour toute connexion TLS sortante éventuelle.
# Pas de libssl/openssl : le binaire n'est lié à aucune bibliothèque TLS
# (sqlx est compilé avec la feature "runtime-tokio" sans backend
# tls-native-tls / tls-rustls, donc aucune dépendance OpenSSL n'existe
# dans l'arbre de dépendances actuel).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bookfin /usr/local/bin/bookfin

# DATABASE_URL, PORT, BIND_ADDR sont lus depuis l'environnement au runtime
# (voir src/main.rs) : aucune valeur n'est fixée ici.
CMD ["/usr/local/bin/bookfin"]
