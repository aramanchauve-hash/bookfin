# BOOKFIN — PRÉPARATION PRIVATE ALPHA BUILD 3

**Date** : 26 septembre 2026
**Objet** : corrections post-audit (`BOOKFIN_FULL_CODE_AUDIT.md`), état du code, checklist et plans de déploiement backend et de build 3.
**Rien n'a été exécuté en production** : ni déploiement Railway, ni migration, ni build EAS. Les seuls accès Railway ont été des lectures (`/health`, statut du service).

---

## 1. Anciens blockers

| Blocker de l'audit | Statut |
| --- | --- |
| P0-1 — backend et 0012 absents de la production | **FIXED** : déployé sur Railway (déploiement `a0f09841-8c75-4b19-8155-8acdcaeec52b`), migration 0012 appliquée, validé live (§ 6) |
| P0-2 — worktree non commité, 0012 non figée | **FIXED** |
| P1-1 — impressions START/RESUME en `random_page` | **FIXED** |
| P1-2 — continue aléatoire qui écrase la progression | **FIXED** |
| P1-3 — réinitialisation silencieuse par « Commencer » | **FIXED** |
| P1-4 — bug Samsung non validé en release | **REMAINS** : validation sur le build 3 (§ 5) |
| P1-5 — retour découverte → 409 | **FIXED** (correctif de l'audit conservé, test vert) |

Le statut de tous les autres constats figure dans `BOOKFIN_FULL_CODE_AUDIT.md` § 0.

---

## 2. Règles désormais figées

### Deux contextes de lecture

| | Découverte (feed aléatoire) | Lecture suivie |
| --- | --- | --- |
| Entrée | `GET /feed/next` | `POST /works/:id/start`, `POST /works/:id/resume` |
| Impression | `reading_context = 'discovery'`, `random_page` puis `continue_book` | `reading_context = 'sequential'`, `start_work` / `resume_work` puis `continue_book` |
| Page suivante | hérite `discovery` | hérite `sequential` (de l'impression parente, côté serveur) |
| `user_work_progress` | **jamais modifié** | avancé à chaque page suivante |
| Swipe gauche en bas de page | LIKE + CONTINUE_READING | **page suivante uniquement**, sans LIKE |
| Pouces Like/Dislike | oui (obligatoires pour la révélation) | masqués |
| « Une autre page au hasard » | nouvelle page aléatoire | quitte la lecture suivie, puis page aléatoire |

Le tirage aléatoire est inchangé : même SQL, filtre de langues, exclusion des pages déjà vues, `random_key`. Une page déjà lue en lecture suivie reste exclue du tirage aléatoire, comme toute page déjà vue.

### Sémantique de l'API `POST /api/v1/works/:work_id/start`

Corps : `{ "user_id", "edition_id", "restart"?: bool = false }`

| Situation | Réponse | Effet |
| --- | --- | --- |
| Œuvre jamais commencée | 200 + page 1 | progression créée à la page 1, événement `start_work` |
| Œuvre déjà commencée, `restart` absent ou false | **409** `{"error":"work_already_started","current_page_sequence_number":N}` | **aucun** (ni progression, ni événement, ni impression) |
| `restart: true` (confirmé par le lecteur) | 200 + page 1 | progression remise à 1, `completed_at` effacé, événement `restart_work` |
| Œuvre inconnue, ou édition d'une autre œuvre | 404 | aucun |
| Identité inconnue | 404 | aucun |

Autres routes :

- `POST /works/:id/resume` : sert la page enregistrée sans modifier la progression (événement `resume_work`). 404 sans progression.
- `GET /me/library` : œuvres commencées par ce lecteur uniquement.
- `GET /pages/:id/reveal` : expose en plus `work_progress_page_sequence_number` quand le lecteur a déjà commencé l'œuvre. Le client s'en sert pour choisir « Commencer » ou « Reprendre ».

### Interface

- Œuvre jamais commencée : « Commencer depuis le début ».
- Œuvre déjà commencée : « Reprendre (p. N) » en principal. « Recommencer depuis le début » en secondaire, avec confirmation (`Alert`) avant `restart: true`.
- Pendant la lecture suivie de l'œuvre : aucune de ces actions.
- Si le serveur répond 409 à un « Commencer » (course entre deux appareils), le client reprend au lieu de réinitialiser.
- Nouvelles clés i18n dans les 6 langues : `common.cancel`, `reading.resume_work`, `reading.restart_from_beginning`, `reading.restart_confirm_title`, `reading.restart_confirm_message`, `reading.restart_confirm_action`.

---

## 3. Migration 0012 (définitive)

Relue intégralement et réécrite **avant** son premier déploiement. Elle n'a jamais été appliquée en production. Elle est strictement additive :

1. `page_impressions.reading_context VARCHAR(16) NOT NULL DEFAULT 'discovery'` avec `CHECK IN ('discovery','sequential')`.
   - Toutes les impressions existantes sont correctement étiquetées « découverte ».
   - Sur PostgreSQL ≥ 11, l'ajout d'une colonne avec défaut constant ne réécrit pas la table (la prod est en Postgres 18).
2. `user_work_progress` :
   - PK `(user_id, work_id)` ;
   - FK `users` / `works` / `editions` en CASCADE, `pages` en SET NULL ;
   - `CHECK (current_page_sequence_number > 0)` ;
   - index `(user_id, last_read_at DESC)`.
3. `reading_events` :
   - `edition_id NOT NULL` ;
   - `CHECK event_type IN ('start_work','resume_work','restart_work')` (le type `'progress'`, jamais émis, a été retiré) ;
   - index `(user_id, work_id, created_at)`.

Idempotence de la migration :

- `IF NOT EXISTS` partout ;
- la contrainte de `page_impressions` est posée dans un bloc `DO` gardé par `pg_constraint`.

Idempotence applicative :

- `ON CONFLICT DO NOTHING` pour START ;
- upsert explicite uniquement pour RESTART.

Validation : appliquée sans erreur à la base locale existante (598 lecteurs, ~17 900 pages, impressions réelles).

L'ancienne version locale de 0012 (tables vides) a été retirée de la base **locale** uniquement, pour que sqlx accepte la nouvelle somme de contrôle.

> **Après le premier déploiement de production, 0012 ne doit plus jamais être modifiée.** Toute évolution passera par `0013+`, sinon le démarrage échoue (`VersionMismatch`).

---

## 4. Vérifications exécutées (26 septembre)

| Commande | Résultat |
| --- | --- |
| `npm run typecheck` | ✅ 0 erreur |
| `npm test` | ✅ **18 suites, 124 tests** |
| `npx expo-doctor` | ⚠️ **20/21** : seul écart `expo` 57.0.24, alors que `~57.0.25` est attendu (patch, non bloquant, inchangé depuis l'audit) |
| `cargo test --lib` | ✅ 4 tests |
| `cargo test` (Postgres local) | ✅ **64 tests** : lib 4, bins 5, `alpha_integration` 2, `domain` 12, `hardening` 12, `integration` 3, `recommendation` 4, `social_integration` 3, `social_pipeline` 10, **`work_progress_routes` 9** |
| `cargo build --all-targets` | ✅ seuls les avertissements préexistants (collision `.pdb`, `proc-macro-error2`) |

Nouveaux tests backend (`tests/work_progress_routes_tests.rs`, HTTP via le routeur `/api/v1`) :

1. start valide → progression page 1, impression `sequential`/`start_work`, aucune impression aléatoire ;
2. œuvre inexistante / édition d'une autre œuvre / identité inconnue → 404, aucune progression ;
3. start déjà commencé → 409, progression intacte (47), un seul `start_work` ;
4. restart explicite → page 1, événement `restart_work` ;
5. resume → page enregistrée, progression inchangée, impression `sequential`/`resume_work` ; resume sans progression → 404 ;
6. continuation suivie → progression 2 puis 3, contexte `sequential` ;
7. **régression Niebla** : progression 47, le feed sert la page 103, deux continuations (104, 105) → progression **toujours 47**, contexte `discovery`, resume → 47 ;
8. bibliothèque : vide, puis une œuvre (progression 47, total 110) ; un autre lecteur voit une bibliothèque vide et ne peut pas reprendre ;
9. reveal : aucun champ de progression avant START, 47 après.

Nouveaux tests mobiles (`readingModeSeparation.test.ts`, 16 tests) : sémantique du swipe par mode, absence de réaction dans la page suivante en lecture suivie, pouces masqués, matrice Commencer/Reprendre/Recommencer, confirmation obligatoire du restart, contrat HTTP de `startWork` (`restart:false` par défaut, 409 reconnu), sortie de mode sur le hasard, reprise d'erreur vers le signet.

---

## 5. Checklist du build 3

### Avant le build

- [x] Backend déployé et vérifié selon le § 6, **avant** toute diffusion du build 3. Déploiement `a0f09841-8c75-4b19-8155-8acdcaeec52b`, migration 0012 appliquée, /health 200, API validée.
- [x] `version` `0.1.0` (le statut « private alpha » est un statut de distribution, pas de version native).
- [x] `android.versionCode` 3, `ios.buildNumber` "3".
- [x] `eas.json` : `cli.appVersionSource: "local"` explicite, pour que ces numéros d'`app.json` fassent foi.
- [x] Correctif Samsung `onLoadEnd` présent dans `ReadingWebView.tsx`.

### Validation obligatoire du build 3 (Android)

- [ ] **Bug Samsung** : lancement à froid (`am force-stop`, puis ouverture) en **clair** et en **sombre**, sur émulateur **et** sur le Samsung qui a signalé le défaut. Le texte doit être visible dès la première page. Le bug n'est pas déclaré clos avant cette étape.
- [ ] Découverte : réaction, révélation, suite, hasard ; swipe = LIKE + suite.
- [ ] Commencer depuis le début → page 1 ; swipe = page suivante sans révélation ni pouce.
- [ ] Retour à la découverte → même page révélée, même réaction, même scroll (pas de 409).
- [ ] Même œuvre retrouvée au hasard → « Reprendre (p. N) » ; « Recommencer » demande une confirmation ; « Annuler » ne change rien.
- [ ] Bibliothèque : l'œuvre apparaît avec la bonne page ; Reprendre ; relance à froid → progression conservée.
- [ ] Continuer depuis le feed une page d'une œuvre commencée → la bibliothèque affiche toujours l'ancienne page.
- [ ] Interface en `ru` et `es` sur petit écran : la barre supérieure (« Retour à la découverte » / « Profil ») ne se chevauche pas.

### iOS

- [ ] Session interactive `eas build --platform ios --profile preview` (identifiants Apple, 2FA, UDID).
- [ ] Mêmes vérifications que ci-dessus, plus la modale `pageSheet` et la boîte de confirmation native.

---

## 6. Déploiement Railway contrôlé (EXÉCUTÉ LE 26 SEPTEMBRE 2026)

Le déploiement a été exécuté avec succès selon le plan contrôlé :

1. **Préflight (lecture seule)**
   - `git status` propre ; HEAD = commit `754e842`.
   - `GET https://bookfin-api-production.up.railway.app/health` → 200 OK.
   - Tunnel SSH ouvert vers PostgreSQL de production (port 55432).
   - `_sqlx_migrations` : 11 migrations présentes, 0012 absente.
   - **Sommes de contrôle** : 100 % conformes. Les SHA-384 des migrations 0001 à 0011 en base correspondaient exactement aux fichiers locaux (notamment 0002 : `9106a953...`).
   - Schéma : tables `user_work_progress` et `reading_events` absentes, colonne `page_impressions.reading_context` absente.
   - Données initiales enregistrées : 33 users, 125 works, 125 editions, 27595 pages, 20 reactions, 176 impressions, 49 language preferences.

2. **Sauvegarde**
   - Timestamp : 2026-09-26 14:18:22 CEST
   - Fichier : `C:\Users\arama\Bookfin-Railway-Backups\bookfin-pre-migration-0012-20260926-141822.dump`
   - Format : PostgreSQL 18 custom dump (`pg_dump -Fc`), 22 863 714 octets
   - SHA-256 : `E2EDC583C6940A8E1ABCFCD00F6E99A0B0CD7FC56E70A065C366E25D45369173`
   - Validation : `pg_restore --list` vérifié avec succès (130 entrées).

3. **Déploiement**
   - Commande : `railway up --service bookfin-api --environment production -y`
   - Deployment ID : `a0f09841-8c75-4b19-8155-8acdcaeec52b` (Statut : SUCCESS)
   - Exécution de 0012 au démarrage : appliquée une seule fois avec succès (`run_migrations`).
   - Somme de contrôle 0012 enregistrée : `1a406e6ac1c8acc2a70378c4f3b3711a84ce41113cbd05560aec432ed7ac45da9ed39b73c259cc9f1dce1538f42f99ef`.
   - Healthcheck Railway : `[1/1] Healthcheck succeeded!`, `/health` public répond 200 OK.

4. **Vérifications post-déploiement en base**
   - `_sqlx_migrations` : 12 migrations, toutes `success = true`, aucune migration antérieure altérée.
   - Tables créées : `user_work_progress` (PK `user_id, work_id`), `reading_events` (avec index).
   - Colonne `reading_context` présente sur `page_impressions` ; les 176 impressions préexistantes sont toutes en `discovery`.

5. **Validation live de l'API (identités de test jetables)**
   - `POST /api/v1/identities/anonymous` → 201 OK.
   - `PUT /api/v1/users/:id/languages` → 200 OK (filtre `["en", "fr"]`).
   - `GET /api/v1/feed/next` → 200 OK (page servie dans les langues choisies, impression `discovery`).
   - `POST /api/v1/pages/:id/continue` (feed) → 200 OK (page suivante servie, `user_work_progress` reste strictement vide).
   - `POST /api/v1/works/:id/start` (initial) → 200 OK (page 1, progression créée, événement `start_work`, impression `sequential`).
   - `POST /api/v1/works/:id/start` (duplicate) → 409 Conflict (`{"error":"work_already_started","current_page_sequence_number":1}`).
   - `POST /api/v1/pages/:id/continue` (sequential) → 200 OK (page 2, progression avancée à 2).
   - `POST /api/v1/works/:id/resume` → 200 OK (reprend page 2, progression inchangée à 2, événement `resume_work`).
   - `POST /api/v1/pages/:id/continue` (feed) pendant une œuvre commencée → 200 OK (progression séquentielle toujours à 2, aucune pollution).
   - `POST /api/v1/works/:id/start` (`restart: true`) → 200 OK (page 1, progression réinitialisée à 1, événement `restart_work`).
   - `GET /api/v1/me/library` (User 1) → 200 OK (1 œuvre présente avec bonne page et total).
   - Isolation User 2 : `GET /api/v1/me/library` vide (0 œuvre), `POST /works/:id/resume` renvoie 404.
   - Nettoyage des données de test : utilisateurs jetables supprimés par CASCADE. Comptages finaux rigoureusement identiques au préflight initial.

---

## 7. Plan du build 3 (PRÊT POUR EXÉCUTION)

1. Backend déployé et vérifié (§ 6) : **FAIT**.
2. `cd mobile && eas build --platform android --profile preview` (APK, `distribution: internal`, API de production).
3. Installer l'APK sur l'émulateur et sur le Samsung, puis dérouler la checklist du § 5.
4. Seulement ensuite : remplacer le lien de distribution (le build 2 ne doit plus être distribué) et lancer le build iOS en session interactive.

---

## 8. Verdicts

```
RAILWAY DEPLOYMENT & MIGRATION 0012: SUCCESS
RAILWAY READY FOR BUILD 3: YES
CODE READY FOR EAS BUILD 3: YES
```

- Le backend de production Railway est désormais 100 % opérationnel avec la migration 0012 et les routes de lecture suivie.
- Le build 3 peut être lancé dès que souhaité. Sa **diffusion** reste conditionnée à la validation Samsung du § 5.
