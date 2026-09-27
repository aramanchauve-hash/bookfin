# BOOKFIN — RAPPORT DE DÉPLOIEMENT RAILWAY & MIGRATION 0012

**Date** : 26 septembre 2026  
**Environnement** : Production Railway (`bookfin-api`, Projet ID `dcb2e128-0626-4d8b-b3ce-fffd956012ac`, Région `sfo`)  
**Commit déployé** : `754e842` (`origin/master`)  
**Railway Deployment ID** : `a0f09841-8c75-4b19-8155-8acdcaeec52b`  
**Statut global** : **SUCCÈS COMPLET**

---

## 1. Préflight en lecture seule

Toutes les vérifications préalables ont été effectuées via un tunnel SSH sécurisé sur PostgreSQL 18.6 sans aucune altération de données :

- **Disponibilité `/health`** : HTTP 200 OK
- **Migrations SQLx en production** :
  - Migrations 1 à 11 présentes et appliquées (`success = true`).
  - Migration 0012 absente.
  - **Sommes de contrôle** : 100 % conformes avec les fichiers locaux.
    - 0001 : `2a6ff87eca8054ba7c98ccddffee1c9840180b70f1ce77a1a6c8f72f84bb93203976ad540b8f0c885bddcaa07a4f5a90`
    - 0002 : `9106a95384d63045f8a17d13d36422521d308867bb9f9426e02df06f12914b264d74fe8d37a997e9fedc81fd8355af44` (conforme au fix `.gitattributes`)
    - 0003 à 0011 : concordance exacte au bit près.
- **Vérification du schéma** :
  - Tables `user_work_progress` et `reading_events` inexistantes.
  - Colonne `reading_context` inexistante sur `page_impressions`.
- **Inventaire initial des données (comptages de référence)** :
  - `users` : 33
  - `works` : 125
  - `editions` : 125
  - `pages` : 27 595
  - `reactions` : 20
  - `page_impressions` : 176
  - `alpha_claimed_invites` : 2
  - `alpha_invite_codes` : 5
  - `taste_profiles` : 0
  - `user_language_preferences` : 49
  - `_sqlx_migrations` : 11

---

## 2. Sauvegarde de la base de données (pg_dump)

Conformément à la consigne absolue de sécurité, un dump binaire complet a été extrait et validé avant toute mutation :

- **Chemin du fichier** : `C:\Users\arama\Bookfin-Railway-Backups\bookfin-pre-migration-0012-20260926-141822.dump`
- **Format** : PostgreSQL 18 custom dump (`pg_dump --format=custom -Fc`)
- **Taille** : 22 863 714 octets (~22,8 Mo)
- **SHA-256** : `E2EDC583C6940A8E1ABCFCD00F6E99A0B0CD7FC56E70A065C366E25D45369173`
- **Contrôle d'intégrité** : `pg_restore --list` exécuté avec succès (130 entrées TOC valides).

---

## 3. Déploiement Railway

- **Mécanisme utilisé** : `railway up --service bookfin-api --environment production -y` (dépôt local Bookfin).
- **Compilation Dockerfile** : Rust 1.98.1 slim, compilation des binaires en mode release en 53,06 s.
- **Application de la migration 0012** :
  - Déclenchée automatiquement au démarrage du conteneur par `run_migrations(&pool)`.
  - Appliquée une seule fois avec succès.
  - Somme de contrôle enregistrée : `1a406e6ac1c8acc2a70378c4f3b3711a84ce41113cbd05560aec432ed7ac45da9ed39b73c259cc9f1dce1538f42f99ef`.
- **Healthcheck Railway** : validé dès le premier essai (`[1/1] Healthcheck succeeded!`).
- **Endpoint public** : `GET https://bookfin-api-production.up.railway.app/health` renvoie HTTP 200 OK (`OK`).

---

## 4. Vérification post-déploiement de la base

- **Table `_sqlx_migrations`** : 12 entrées, toutes `success = t`.
- **Nouvelles tables** :
  - `user_work_progress` : PK `(user_id, work_id)`, FK en cascade, check `current_page_sequence_number > 0`.
  - `reading_events` : index `(user_id, work_id, created_at)`.
- **Colonne `reading_context`** :
  - Ajoutée à `page_impressions` avec défaut `'discovery'`.
  - `SELECT reading_context, count(*) FROM page_impressions GROUP BY 1` → 176 lignes, 100 % en `discovery`.

---

## 5. Tests live de l'API & intégrité

Exécutés sur le domaine de production en conditions réelles à l'aide d'identités jetables :

1. **Création d'identités** (`POST /api/v1/identities/anonymous`) :
   - Création de User 1 et User 2 → 201 Created.
2. **Préférences de langues** (`PUT /api/v1/users/:id/languages`) :
   - Définition de `["en", "fr"]` pour User 1 → 200 OK.
3. **Feed découverte** (`GET /api/v1/feed/next`) :
   - Page servie conforme aux langues du lecteur, impression enregistrée en `reading_context = 'discovery'`, `navigation_action = 'random_page'`.
4. **Continuation dans le feed** (`POST /api/v1/pages/:id/continue`) :
   - Page suivante servie en contexte `discovery`.
   - `user_work_progress` reste strictement vide (aucune pollution).
5. **Démarrage lecture suivie** (`POST /api/v1/works/:id/start`) :
   - Œuvre « Daisy Miller » démarrée à la page 1 (200 OK).
   - `user_work_progress` créé à la page 1.
   - Événement `start_work` enregistré dans `reading_events`.
   - Impression enregistrée en `reading_context = 'sequential'`.
6. **Tentative de start en doublon** (`POST /api/v1/works/:id/start` sans restart) :
   - Réponse : **409 Conflict** (`work_already_started`, `current_page_sequence_number = 1`).
   - Aucune impression ni progression modifiée.
7. **Progression séquentielle** (`POST /api/v1/pages/:id/continue` avec parent séquentiel) :
   - Page 2 servie, `user_work_progress` mis à jour à 2.
8. **Reprise de lecture** (`POST /api/v1/works/:id/resume`) :
   - Page 2 servie (200 OK), progression maintenue à 2, événement `resume_work` enregistré.
9. **Invariant de séparation découvert / lecture suivie** :
   - Depuis le feed aléatoire, un appel `continue` sur une autre œuvre n'affecte en rien la progression de « Daisy Miller » qui reste figée à 2.
10. **Redémarrage explicite** (`POST /api/v1/works/:id/start` avec `restart: true`) :
    - Page 1 servie, progression réinitialisée à 1, événement `restart_work` enregistré.
11. **Bibliothèque** (`GET /api/v1/me/library`) :
    - User 1 voit exactement 1 œuvre avec progression 1 et total 79.
12. **Isolation stricte des utilisateurs** :
    - User 2 voit une bibliothèque vide (`[]`).
    - User 2 appelant `/works/:id/resume` reçoit 404 (non commencée).

---

## 6. Nettoyage et vérification finale des données

Les utilisateurs jetables de test ont été supprimés par `DELETE FROM users WHERE id IN (...)` (suppression en cascade de toutes leurs impressions, préférences, progressions et événements de test).

Comptage final en base de production après nettoyage :

| Table | Préflight | Post-déploiement | Delta |
| --- | --- | --- | --- |
| `users` | 33 | 33 | 0 |
| `works` | 125 | 125 | 0 |
| `editions` | 125 | 125 | 0 |
| `pages` | 27 595 | 27 595 | 0 |
| `reactions` | 20 | 20 | 0 |
| `page_impressions` | 176 | 176 | 0 |
| `user_language_preferences` | 49 | 49 | 0 |
| `alpha_claimed_invites` | 2 | 2 | 0 |
| `alpha_invite_codes` | 5 | 5 | 0 |
| `taste_profiles` | 0 | 0 | 0 |
| `user_work_progress` | 0 | 0 | 0 |
| `reading_events` | 0 | 0 | 0 |
| `_sqlx_migrations` | 11 | **12** | **+1** (0012) |

Aucune corruption, aucune perte de données, aucun lecteur contaminé.

---

## 7. Statut final

```
RAILWAY READY FOR BUILD 3: YES
```
