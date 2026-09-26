# BOOKFIN — PRÉPARATION PRIVATE ALPHA BUILD 3

**Date** : 26 septembre 2026
**Objet** : corrections post-audit (`BOOKFIN_FULL_CODE_AUDIT.md`), état du code, checklist et plans de déploiement backend et de build 3.
**Rien n'a été exécuté en production** : ni déploiement Railway, ni migration, ni build EAS. Les seuls accès Railway ont été des lectures (`/health`, statut du service).

---

## 1. Anciens blockers

| Blocker de l'audit | Statut |
| --- | --- |
| P0-1 — backend et 0012 absents de la production | **REMAINS** : le code est prêt, le déploiement contrôlé reste à exécuter (§ 6) |
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

- [ ] Backend déployé et vérifié selon le § 6, **avant** toute diffusion du build 3. Sans lui, la bibliothèque est en erreur et les actions de lecture suivie restent masquées.
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

## 6. Plan de déploiement Railway contrôlé (NON EXÉCUTÉ)

Le service `bookfin-api` n'est **pas** relié à GitHub (`source.repo = None`, vérifié en lecture). Il ne se déploie que par `railway up`, et un `git push` ne déploie rien.

1. **Préflight (lecture seule)**
   - `git status` propre ; HEAD = commit revu.
   - `GET https://bookfin-api-production.up.railway.app/health` → 200.
   - Tunnel `railway connect Postgres --environment production --ssh --tunnel-only --port 55432`, puis `SELECT max(version) FROM _sqlx_migrations` → **11** attendu.
   - **Sommes de contrôle** : `SELECT version, encode(checksum,'hex') FROM _sqlx_migrations ORDER BY version` doit correspondre, pour 0001 à 0011, au SHA-384 des fichiers locaux (`migrations/*.sql`, octets exacts). sqlx ne normalise pas les fins de ligne, et un seul octet différent empêche le démarrage (`VersionMismatch`).
     - Contexte : lors de cette mission, le blob git de `0002` différait des octets réellement déployés (CRLF normalisés par git).
     - Correction : `.gitattributes` (`migrations/*.sql -text`) fige désormais les octets exacts, et `0002` est commitée telle que déployée (commit `30c789b`). Les fichiers locaux correspondent aux sommes de contrôle de la base locale pour les 12 migrations.
     - Déployer depuis ce dépôt, jamais depuis une copie dont les fins de ligne auraient été converties.
2. **Sauvegarde** : `pg_dump -Fc` de la base de production via le tunnel (même procédure que `CURATED_V1_RAILWAY_IMPORT_PLAN.md`). Vérifier la taille et le SHA-256, et conserver le fichier hors du dépôt.
3. **Déploiement** : `railway up --service bookfin-api --environment production` depuis le commit revu. `bookfin` applique 0012 au démarrage (`run_migrations`), puis le healthcheck `/health` doit passer.
4. **Vérifications post-déploiement**
   - En lecture seule :
     - `_sqlx_migrations` : version 12, `success = true` ;
     - `\d page_impressions` : `reading_context` présent ;
     - `SELECT reading_context, count(*) FROM page_impressions GROUP BY 1` → uniquement `discovery` ;
     - `GET /api/v1/works/<uuid>/start` → **405** (la route existe ; elle renvoyait 404 avant).
   - Smoke test avec une identité de test dédiée (écriture volontaire, documentée) :
     - `POST /identities/anonymous` ;
     - START sur une œuvre → page 1 ; second START → 409 ; resume ; `GET /me/library` → une œuvre.
5. **Compatibilité** : l'APK build 2 reste fonctionnel face au nouveau backend. Les changements sont additifs, et `reveal` ne fait qu'ajouter un champ optionnel.
6. **Rollback — attention.** L'ancien binaire embarque seulement les migrations 0001 à 0011. Face à une base où 12 est appliquée, `sqlx::migrate!().run()` échoue (`VersionMissing(12)`) et **l'ancien binaire ne démarrera pas**. Il y a deux options :
   - (a) préférée : corriger en avant avec un nouveau binaire (nouvelle migration `0013` si le schéma doit changer) ;
   - (b) retour arrière : `DELETE FROM _sqlx_migrations WHERE version = 12;` puis redéployer l'ancien commit. Le schéma additif de 0012 (tables, colonne) peut rester en place, puisque l'ancien code l'ignore. Le supprimer n'est utile qu'en cas d'abandon définitif. Et si 0012 est réappliquée plus tard, ses `IF NOT EXISTS` la rendent sûre.

## 7. Plan du build 3 (NON EXÉCUTÉ)

1. Backend déployé et vérifié (§ 6).
2. `cd mobile && eas build --platform android --profile preview` (APK, `distribution: internal`, API de production).
3. Installer l'APK sur l'émulateur et sur le Samsung, puis dérouler la checklist du § 5.
4. Seulement ensuite : remplacer le lien de distribution (le build 2 ne doit plus être distribué) et lancer le build iOS en session interactive.

---

## 8. Verdicts

```
CODE READY FOR CONTROLLED RAILWAY DEPLOYMENT: YES
CODE READY FOR EAS BUILD 3: YES
```

- Le build 3 peut être produit dès que le backend est déployé. Sa **diffusion** reste conditionnée à la validation Samsung du § 5.
- Restent ouverts, non bloquants (P2/P3) : chaînes codées en dur et messages serveur en français, robustesse WebView (`READY`, crash du renderer, `color-scheme`), erreurs de synchro des langues, `completed_at`, badge bibliothèque, patch `expo`.
