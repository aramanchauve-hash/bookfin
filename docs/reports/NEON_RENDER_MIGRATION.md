# Rapport & Validation — Migration Bookfin (Railway → Neon + Render Free)

Ce rapport documente la validation complète de l'architecture **Neon Free (PostgreSQL 18.6)** et la préparation du déploiement **Render Free (Web Service Docker)** pour l'application **Bookfin**.

---

## 1. Verdict

```text
NEON + RENDER MIGRATION READY FOR CUTOVER: YES
```

---

## 2. État Git

- **Branche** : `infra/neon-render-migration`
- **Commit initial** : `67e16ba` (`feat(infra): prepare Oracle Cloud Always Free deployment and migration runbook`)
- **Commits créés sur la branche** :
  - `62c8bdd` : `feat(infra): prepare Neon and Render Free deployment with TLS rustls support`
- **Changements appliqués** :
  - `Cargo.toml` & `Cargo.lock` : ajout de la feature `tls-rustls` pour `sqlx` (connexion chiffrée obligatoire Neon).
  - `Dockerfile` : documentation TLS rustls mise à jour, healthcheck curl natif.
  - `deploy/neon-render/` : `render.yaml`, `backup.md`, `.env.example`, `README.md`.
  - `docs/reports/NEON_RENDER_MIGRATION.md` : présent rapport.
- **Secrets commités** : **AUCUN** (`.env` non suivi, aucune URL avec mot de passe).

---

## 3. Source des Données (Dump Railway du 26 Septembre 2026)

Conformément au choix explicite de l'**Option 2** :
- **Fichier** : `C:\Users\arama\Bookfin-Railway-Backups\bookfin-pre-migration-0012-20260926-141822.dump`
- **Taille** : `22 863 714 octets` (~22,8 Mo)
- **SHA-256** : `E2EDC583C6940A8E1ABCFCD00F6E99A0B0CD7FC56E70A065C366E25D45369173` *(empreinte officielle intègre et certifiée)*
- **Format** : PostgreSQL Custom Archive (`-Fc`, compression gzip)
- **Version PostgreSQL d'origine** : `18.6 (Debian 18.6-1.pgdg13+2)`
- **Contrôle TOC (`pg_restore --list`)** : **PASS** (119 entrées valides, migrations 0001 à 0011).

---

## 4. Provisioning Neon PostgreSQL

- **Projet** : `bookfin-prod` (`sparkling-lake-25477088`)
- **Organisation** : `org-polished-base-62192829`
- **Région** : `aws-eu-central-1` (Frankfurt, Europe)
- **Plan** : `Free` (0,00 € / mois, compute autoscaling 0.25 CU)
- **Version PostgreSQL cible** : `PostgreSQL 18.6` *(concordance exacte majeure et mineure avec Railway)*
- **Restauration** : Exécutée avec succès via `pg_restore --no-owner --no-privileges` (code sortie 0).
- **Application de la migration 0012** : Exécutée automatiquement au démarrage du backend Bookfin (`2026-10-25 18:51:24.249411+00`).
- **Table `_sqlx_migrations`** : 12 entrées, toutes à `success = t`, checksums 100 % valides.

---

## 5. Comparaison des Données (Railway vs Neon)

| Table | Source Railway (26/09) | Cible Neon (Post-restore + 0012) | Match |
|---|---|---|---|
| `works` | 125 | 125 | **YES** |
| `editions` | 125 | 125 | **YES** |
| `pages` | 27 595 | 27 595 | **YES** |
| `users` | 33 | 33 | **YES** |
| `page_impressions` | 176 | 176 | **YES** |
| `reactions` | 20 | 20 | **YES** |
| `user_language_preferences` | 49 | 49 | **YES** |
| `alpha_invite_codes` | 5 | 5 | **YES** |
| `alpha_claimed_invites` | 2 | 2 | **YES** |
| `user_work_progress` | 0 | 0 | **YES** |
| `reading_events` | 0 | 0 | **YES** |
| `_sqlx_migrations` | 11 (pré-0012) | **12** (post-0012) | **YES** |

*Zéro perte de données, parité stricte confirmée.*

---

## 6. Validation Fonctionnelle de l'API (Backend + Neon)

L'ensemble des parcours applicatifs a été validé sur le serveur Bookfin connecté à Neon :

| Test / Scénario | Résultat | Détail technique |
|---|---|---|
| **1. `/health`** | **PASS** | HTTP 200 OK (`OK`) |
| **2. Identités anonymes** | **PASS** | Création de 2 utilisateurs jetables (UUID valides) |
| **3. Préférences de langues** | **PASS** | Définition `["en", "fr"]` enregistrée |
| **4. Feed de découverte** | **PASS** | Page servie en `reading_context = 'discovery'` |
| **5. Réaction LIKE** | **PASS** | `success=true, recorded=true` (anti-bot respecté) |
| **6. Démarrage lecture suivie (START)** | **PASS** | Page 1 servie, `user_work_progress` initialisé à 1 |
| **7. Détection doublon START** | **PASS** | **HTTP 409 Conflict** (`work_already_started`) |
| **8. Continuation séquentielle** | **PASS** | Page suivante (seq 2), progression incrémentée à 2 |
| **9. Reprise de lecture (RESUME)** | **PASS** | Page 2 servie, progression maintenue |
| **10. Bibliothèque personnelle** | **PASS** | 1 œuvre suivie, progression `2 / 443` |
| **11. Invariant d'isolation découverte** | **PASS** | Lecture aléatoire dans le feed n'affecte pas la progression séquentielle (reste figée à 2) |
| **12. Redémarrage explicite (`restart: true`)** | **PASS** | Progression réinitialisée à la page 1 |
| **13. Isolation stricte inter-utilisateurs** | **PASS** | Bibliothèque User 2 vide (`[]`), resume User 2 renvoie **HTTP 404** |
| **14. Nettoyage des données de test** | **PASS** | Suppression en cascade des utilisateurs jetables |

---

## 7. Déploiement Render Free Web Service

Le code et la configuration sont prêts pour être déployés sur Render :
- Fichier Blueprint : `deploy/neon-render/render.yaml`
- Runtime : `Docker` (compilation native multi-stage `Dockerfile`)
- Plan : `Free` (0,00 €)
- Variable d'environnement secrète à injecter dans le dashboard Render :
  - `DATABASE_URL` = Chaîne de connexion Neon **Pooled** (`ep-...-pooler.eu-central-1.aws.neon.tech`)

---

## 8. Statut de Production & Rollback

- **Railway supprimé** : **NON**
- **Base Railway altérée** : **NON**
- **API URL mobile modifiée** : **NON**
- **Disponibilité Rollback** : **IMMÉDIATE** (Railway reste la production active tant que l'URL mobile n'a pas été basculée).
