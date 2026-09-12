# Bookfin — Guide de Démarrage & Première Version Exécutable

Bienvenue dans le projet **Bookfin** ! Ce guide est conçu pour vous accompagner pas à pas sous **Windows** avec **VS Code**, même si vous découvrez Rust, SQLx et Docker.

---

## Table des matières

1. [Vue d'ensemble de l'application](#vue-densemble-de-lapplication)
2. [Prérequis](#1-prérequis)
3. [Installation de Rust et des composants WebAssembly](#2-installation-de-rust-et-des-composants-webassembly)
4. [Installation de cargo-leptos](#3-installation-de-cargo-leptos)
5. [Démarrage de PostgreSQL](#4-démarrage-de-postgresql)
6. [Configuration du fichier d'environnement (.env)](#5-configuration-du-fichier-denvironnement-env)
7. [Exécution des migrations de schéma](#6-exécution-des-migrations-de-schéma)
8. [Peuplement des données de test (Seed)](#7-peuplement-des-données-de-test-seed)
9. [Lancement de l'application](#8-lancement-de-lapplication)
10. [Test du parcours utilisateur dans le navigateur](#9-test-du-parcours-utilisateur-dans-le-navigateur)
11. [Arrêt propre des services](#10-arrêt-propre-des-services)
12. [Architecture et fichiers créés](#architecture-et-fichiers-créés)

---

## Vue d'ensemble de l'application

Bookfin est une application de découverte littéraire à l'aveugle :
- L'utilisateur découvre un court extrait littéraire centré et épuré.
- Le titre et l'auteur sont initialement masqués et conservés côté serveur (ils ne sont pas envoyés au navigateur pour éviter toute triche).
- L'utilisateur peut réagir : **Like**, **Skip**, **Save**, ou **Reveal**.
- Le clic sur **Reveal** dévoile le titre, l'auteur, l'année et la langue, puis affiche un bouton **Next**.
- Les réactions sont enregistrées de façon idempotente (un rejeu ou double clic ne crée aucun doublon grâce à un identifiant unique `event_id`).
- La sélection des extraits s'appuie sur la stratégie `UnseenRandomStrategy` avec rebouclage (*wrap-around*) sur `random_key`, garantissant qu'un extrait déjà vu ne réapparaît pas immédiatement.

---

## 1. Prérequis

Avant de commencer, vérifiez que vous disposez sur votre machine Windows :
1. **VS Code** (avec l'extension recommandée `rust-analyzer`).
2. **Terminal PowerShell** (intégré dans VS Code : menu *Terminal > Nouveau terminal*).
3. Au choix pour PostgreSQL :
   - **Docker Desktop** (avec prise en charge WSL2), OU
   - **PostgreSQL 16/17/18** installé localement sous Windows.

---

## 2. Installation de Rust et des composants WebAssembly

Si Rust n'est pas encore installé sur votre système :

1. Téléchargez et lancez l'installateur officiel : [rustup-init.exe](https://rustup.rs/).
2. Dans PowerShell, vérifiez l'installation :
   ```powershell
   rustc --version
   cargo --version
   ```
3. Ajoutez la cible de compilation client WebAssembly (`wasm32-unknown-unknown`) :
   ```powershell
   rustup target add wasm32-unknown-unknown
   ```

---

## 3. Installation de cargo-leptos

`cargo-leptos` est l'outil officiel de build et de rechargement à chaud (*hot-reload*) pour Leptos SSR.

Pour l'installer sous Windows :
```powershell
cargo install cargo-leptos --locked
```

> **Astuce** : Si vous utilisez `cargo-binstall` pour une installation instantanée en binaire précompilé :
> ```powershell
> cargo binstall cargo-leptos -y
> ```

---

## 4. Démarrage de PostgreSQL

### Option A : Avec Docker Desktop (Recommandé)

À la racine du projet `Bookfin`, exécutez :
```powershell
docker compose up -d
```
*Ce que vous devez voir :*
Le conteneur `bookfin-postgres` démarre en arrière-plan et expose le port `5432`.

Pour vérifier que le conteneur tourne :
```powershell
docker ps
```

### Option B : Avec PostgreSQL installé localement sous Windows

Si vous utilisez PostgreSQL natif, connectez-vous avec `psql` et créez la base de données ainsi que l'utilisateur :
```sql
CREATE USER bookfin WITH PASSWORD 'bookfin' SUPERUSER;
CREATE DATABASE bookfin OWNER bookfin;
```

---

## 5. Configuration du fichier d'environnement (.env)

Copiez le fichier `.env.example` en `.env` :
```powershell
Copy-Item .env.example .env
```

Le fichier `.env` contient par défaut :
```ini
DATABASE_URL=postgres://bookfin:bookfin@localhost:5432/bookfin
LEPTOS_OUTPUT_NAME=bookfin
LEPTOS_SITE_ROOT=target/site
LEPTOS_SITE_ADDR=127.0.0.1:3000
LEPTOS_RELOAD_PORT=3001
RUST_LOG=info
```

Si votre mot de passe PostgreSQL local diffère, modifiez `DATABASE_URL` dans `.env`.

---

## 6. Exécution des migrations de schéma

Les migrations SQL créent les tables nécessaires : `users`, `user_language_preferences`, `books`, `extracts`, `extract_impressions`, `reactions`, `taste_profiles`, `user_affinities`.

Le fichier de migration est situé dans :
`migrations/0001_initial_schema.sql`

Les migrations sont appliquées **automatiquement** au démarrage du serveur ou peuvent être jouées avec SQLx CLI si vous l'avez installé.

---

## 7. Peuplement des données de test (Seed)

> **Important** : Conformément aux règles d'architecture, le seed ne fait **jamais** partie du chemin de migration automatique. Il s'exécute explicitement via son propre binaire dédié.

Lancez la commande :
```powershell
cargo run --bin seed
```

*Ce que vous devez voir dans le terminal :*
```
==================================================
  Bookfin - Peuplement de développement (Seed)
==================================================
Connexion à : postgres://bookfin:bookfin@localhost:5432/bookfin
✓ Utilisateur fixe configuré : 00000000-0000-0000-0000-000000000001
✓ Préférences linguistiques ordonnées insérées (en -> fr -> es -> de -> ja).
✓ 15 livres et 25 extraits littéraires insérés.
✓ Clés random_key distribuées équitablement entre 0.0 et 1.0.
==================================================
  Seed terminé avec succès !
==================================================
```

Le corpus inséré contient 25 extraits multilingues :
- Majorité en anglais (~14 extraits)
- Plusieurs en français (5 extraits)
- Extraits en espagnol et allemand
- Deux extraits utilisant des alphabets non latins (Japonais et Grec)

---

## 8. Lancement de l'application

Pour lancer l'application en mode développement avec compilation SSR et assets :

### Méthode 1 : Avec `cargo-leptos` (Rechargement automatique)
```powershell
cargo leptos watch
```

### Méthode 2 : Directement avec Cargo
```powershell
cargo run
```

*Ce que vous devez voir :*
```
Connexion à PostgreSQL sur postgres://bookfin:bookfin@localhost:5432/bookfin
Exécution des migrations SQLx...
Migrations terminées avec succès.
Démarrage du serveur Bookfin sur http://127.0.0.1:3000
```

---

## 9. Test du parcours utilisateur dans le navigateur

1. Ouvrez votre navigateur web sur :
   👉 **`http://127.0.0.1:3000`**

2. **Écran principal (Découverte à l'aveugle)** :
   - Vous voyez le titre épuré `EXTRAIT LITTÉRAIRE`.
   - Le texte d'un extrait littéraire s'affiche dans une typographie serif élégante.
   - Le titre et l'auteur ne sont **pas** visibles (et ne sont pas présents dans le code source HTML envoyé au client).
   - Les boutons d'action sont affichés : `♡ Like`, `× Skip`, `☆ Save`, et `Reveal`.

3. **Cliquer sur `Reveal`** :
   - Le titre de l'œuvre, son auteur, sa langue d'origine et son année apparaissent sous l'extrait.
   - Les boutons de réaction laissent place au bouton **`Next`**.

4. **Cliquer sur `Next`** (ou directement sur `Like` / `Skip` / `Save` sur l'écran précédent) :
   - L'impression et la réaction sont enregistrées en base PostgreSQL.
   - Un nouvel extrait non vu est immédiatement chargé.
   - Si vous actualisez la page, l'extrait déjà vu n'est pas représenté.

---

## 10. Arrêt propre des services

- **Arrêter l'application Rust** :
  Appuyez sur `Ctrl + C` dans le terminal où tourne `cargo run` ou `cargo leptos watch`.
- **Arrêter PostgreSQL Docker** :
  ```powershell
  docker compose down
  ```
  *(Vos données restent conservées dans le volume Docker `bookfin_pg_data`)*.

---

## 11. Exécution des tests automatisés

Pour vérifier l'intégrité de l'architecture et du code :

1. **Vérification du formatage** :
   ```powershell
   cargo fmt --check
   ```
2. **Analyse statique Clippy** :
   ```powershell
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Tests unitaires et d'intégration** :
   ```powershell
   cargo test -- --nocapture
   ```

Les tests valident notamment :
- Le rejet des tags de langue vides (`DomainError::EmptyLanguageTag`).
- Le wrap-around sur `random_key` dans `UnseenRandomStrategy`.
- La persistance en base d'une réaction.
- L'idempotence stricte (rejouer le même `event_id` n'ajoute aucun doublon).
- L'exclusion des extraits déjà vus pour l'utilisateur.

---

## 12. Architecture et fichiers créés

```
Bookfin/
├── Cargo.toml                       # Configuration du projet, dépendances Leptos 0.7, Axum, SQLx
├── docker-compose.yml               # Service PostgreSQL 16 persistant
├── .env.example                     # Modèle de variables d'environnement
├── .env                             # Variables locales actives (DATABASE_URL, etc.)
├── style/
│   └── main.css                     # Feuille de styles minimaliste et typographique
├── migrations/
│   └── 0001_initial_schema.sql      # Schéma initial PostgreSQL (sans aucune donnée de seed)
├── src/
│   ├── lib.rs                       # Racine de la bibliothèque + point d'ancrage hydration wasm
│   ├── main.rs                      # Serveur Axum, montage des routes Leptos SSR et migrations
│   ├── bin/
│   │   └── seed.rs                  # Binaire autonome de peuplement multilingue (25 extraits)
│   ├── domain/                      # Couche Domaine (pur Rust, sans SQLx ni Leptos ni Axum)
│   │   ├── mod.rs
│   │   ├── errors.rs                # DomainError
│   │   └── models.rs                # LanguageTag, User, Book, Extract, Reaction, ReactionType
│   ├── application/                 # Couche Application (ports et use cases)
│   │   ├── mod.rs
│   │   ├── dtos.rs                  # ExtractCardDto (masqué), BookMetadataDto, ReactionResponseDto
│   │   ├── ports.rs                 # Traits ExtractRepository, ImpressionRepository, ReactionRepository
│   │   └── use_cases.rs             # GetNextExtractUseCase, RecordReactionUseCase (idempotent)
│   ├── recommendation/              # Couche Recommandation
│   │   ├── mod.rs
│   │   └── strategy.rs              # UnseenRandomStrategy avec logique de wrap-around
│   ├── infrastructure/              # Couche Infrastructure (implémentations SQLx PostgreSQL)
│   │   ├── mod.rs
│   │   ├── db.rs                    # Pool PostgreSQL et exécution des migrations
│   │   └── repositories.rs          # PostgresExtractRepository, PostgresReactionRepository, etc.
│   └── web/                         # Couche Présentation & Transport Leptos
│       ├── mod.rs
│       ├── server_fns.rs            # Server Functions Leptos (/api/get_next_extract, /api/record_reaction)
│       └── app.rs                   # Composant racine App & Shell HTML
└── tests/
    ├── domain_tests.rs              # Tests unitaires du modèle et de validation
    ├── recommendation_tests.rs      # Tests unitaires de wrap-around et d'exclusion des vus
    └── integration_tests.rs         # Tests d'intégration PostgreSQL (persistance & idempotence)
```

