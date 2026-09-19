# Image Docker pour l'API Bookfin (Axum) et les binaires de maintenance
# Curated V1 (ingest_curated_v1 / audit_railway_readonly /
# verify_curated_v1_railway), invoqués uniquement via un `railway ssh`
# explicitement autorisé — le processus démarré automatiquement reste
# `bookfin` (voir railway.toml). N'inclut pas l'UI Leptos ni les assets /pkg
# (voir railway.toml pour le contexte : Railway ne sert que /api/v1/* et
# /health pour l'alpha mobile).
#
# Railway's auto-detected Nixpacks/Railpack builder only ever copied the
# single `bookfin` binary it inferred from Cargo.toml into the final image,
# silently ignoring railway.toml's buildCommand for the actual compile/copy
# step (confirmed by `railway ssh` inspection on 2026-09-16/17: the extra
# --bin targets never reached the deployed container even after changing
# buildCommand, and the layer was reused unchanged across redeploys). This
# explicit Dockerfile takes over that packaging.

# ---- Stage 1 : build ----
FROM rust:1.98.1-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# sqlx::migrate! lit ./migrations au moment de la compilation et embarque son
# contenu dans le binaire : aucun accès à une base de données n'est requis ici,
# et le dossier migrations/ n'a pas besoin d'être présent dans l'image finale.
RUN cargo build --release \
    --bin bookfin \
    --bin ingest_curated_v1 \
    --bin audit_railway_readonly \
    --bin verify_curated_v1_railway

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
COPY --from=builder /app/target/release/ingest_curated_v1 /usr/local/bin/ingest_curated_v1
COPY --from=builder /app/target/release/audit_railway_readonly /usr/local/bin/audit_railway_readonly
COPY --from=builder /app/target/release/verify_curated_v1_railway /usr/local/bin/verify_curated_v1_railway

# ingest_curated_v1 et verify_curated_v1_railway lisent ces fichiers sur
# disque au runtime (chemins relatifs à ce WORKDIR) — contrairement aux
# migrations, leur contenu n'est pas embarqué à la compilation.
WORKDIR /app
COPY corpus/curated_v1/manifest.json corpus/curated_v1/manifest.json
COPY corpus/curated_v1/normalized corpus/curated_v1/normalized
COPY corpus/curated_v1/pages corpus/curated_v1/pages

# DATABASE_URL, PORT, BIND_ADDR sont lus depuis l'environnement au runtime
# (voir src/main.rs) : aucune valeur n'est fixée ici.
CMD ["/usr/local/bin/bookfin"]
