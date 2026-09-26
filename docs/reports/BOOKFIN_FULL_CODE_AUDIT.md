# BOOKFIN — AUDIT COMPLET DU CODE AVANT NOUVEAUX BUILDS DE TEST

**Date** : 26 septembre 2026
**Périmètre** : worktree local `C:\Users\arama\Dev\Bookfin` (branche `master`, HEAD `24ce418` + modifications non commitées), backend Railway en production (sondes GET en lecture seule), APK preview déjà distribué.
**Nature** : audit. Un seul correctif local (bug confirmé, § 6), aucun build EAS, aucun déploiement Railway, aucune migration appliquée en production.

---

## 0. Statut après corrections post-audit (26 septembre 2026)

> Les sections 1 à 20 ci-dessous sont l'audit initial, conservé tel quel. Ce tableau donne l'état de chaque constat après la mission de corrections. Détails et plan : `PRIVATE_ALPHA_BUILD_3_PREP.md`.

| ID | Constat | Statut | Comment |
| --- | --- | --- | --- |
| P0-1 | Backend et migration 0012 absents de la production | **REMAINS** (hors code) | Code prêt et testé ; déploiement Railway contrôlé à exécuter par un humain (plan dans `PRIVATE_ALPHA_BUILD_3_PREP.md`) |
| P0-2 | Worktree non commité, 0012 non figée | **FIXED** | 0012 réécrite et définitive, tout est commité en commits cohérents |
| P1-1 (S1) | Impressions START/RESUME enregistrées en `random_page` | **FIXED** | Nouvelle colonne `page_impressions.reading_context` (`discovery` / `sequential`) ; actions `start_work` / `resume_work` ; les continuations héritent du contexte de l'impression parente |
| P1-2 (S2) | Un `continue` aléatoire écrase la progression | **FIXED** | `sync_sequential_progress` : seule une continuation `sequential` met à jour `user_work_progress` ; test de régression Niebla 47/103 |
| P1-3 (S3) | « Commencer depuis le début » réinitialise silencieusement | **FIXED** | API : START sur œuvre commencée → 409 `work_already_started`, rien n'est modifié ; `restart: true` explicite uniquement. UI : « Reprendre (p. N) » en principal, « Recommencer » secondaire avec confirmation, aucune action pendant la lecture suivie de l'œuvre |
| P1-4 | Fix Samsung non validé en release / sur Samsung | **REMAINS** | Correctif `onLoadEnd` conservé ; validation obligatoire sur le build 3 (checklist) |
| P1-5 | Retour découverte → 409 | **FIXED** (audit) | Correctif conservé ; test de régression présent et vert |
| S4 | START_WORK non dédoublonné | **FIXED** | `INSERT … ON CONFLICT DO NOTHING` ; un seul événement `start_work` ; test |
| S5 | « Page au hasard » en lecture suivie sans sortie de mode | **FIXED** | Le hasard quitte explicitement la lecture suivie (signet abandonné) |
| S6 | Échec start/resume puis « Réessayer » tire une autre page | **FIXED** | « Réessayer » ramène au signet de découverte ; en lecture suivie, il rejoue la page suivante |
| S7 | work / edition non vérifiés | **FIXED** | `edition_belongs_to_work` → 404 ; identité inconnue → 404 (au lieu de 500) |
| S8 | Chaque page tournée en lecture suivie crée un LIKE | **FIXED** | Règle figée : lecture suivie, swipe = page suivante uniquement ; pouces masqués en lecture suivie ; barre « page suivante » permanente |
| S9 | `completed_at` jamais renseigné | REMAINS (P3) | Sans impact sur les builds de test |
| — | Tests backend des nouvelles routes absents | **FIXED** | `tests/work_progress_routes_tests.rs` : 9 tests HTTP |
| — | `version 0.1.0-alpha` invalide pour TestFlight | **FIXED** | `0.1.0`, `versionCode` 3, `buildNumber` 3, `appVersionSource: local` |
| — | `PRIVATE_ALPHA_READINESS.md` contredit le rapport Samsung | **FIXED** | Rapport annoté, verdict révoqué jusqu'au build 3 |
| — | Rapport alpha comptant la lecture suivie comme découverte | **FIXED** | `alpha_report` filtre `reading_context = 'discovery'` |
| P2 | Chaînes codées en dur (NetworkBanner, indice de validation, messages serveur FR) | REMAINS | Non bloquant ; à traiter hors de ce lot |
| P2 | WebView : `READY` ignoré, pas de récupération après crash du renderer, `color-scheme` / `userInterfaceStyle` | REMAINS | Renderer gelé ; à vérifier humainement sur le build 3 |
| P2 | Échec de synchro des langues dans Réglages non géré | REMAINS | Non bloquant |
| P3 | Badge bibliothèque, flash `en`, labels de langues FR, `AlphaGateScreen` mort, PNG racine suivis, `expo` 57.0.25, profils EAS `production`/`development` | REMAINS | Non bloquants |

**CODE READY FOR CONTROLLED RAILWAY DEPLOYMENT : YES**
**CODE READY FOR EAS BUILD 3 : YES** (distribution conditionnée au déploiement backend puis à la validation Samsung)

---

## 1. Executive summary

La lecture suivie, la bibliothèque, Profil/Réglages et l'i18n (6 locales) existent dans le code local, mais **rien de tout cela n'est commité ni déployé**. Tous les contrôles automatisés passent : typecheck, 108 tests jest, 54 tests Rust (Postgres local), expo-doctor 20/21.

Le feed aléatoire n'a pas été modifié. Sa requête SQL est inchangée : filtre de langues, exclusion des pages déjà vues, ordre par `random_key`. On ne trouve ni pondération, ni personnalisation, ni ranking.

Constats principaux :

- **Production non alignée.** Railway ne connaît ni `/works/:id/start`, ni `/works/:id/resume`, ni `/me/library` (404). La migration `0012` n'y est donc pas appliquée. Un nouvel APK utilisé contre la prod aurait une bibliothèque en erreur permanente.
- **Bug Samsung.** La cause est comprise et le correctif (`4baa3cb`) est dans le code. Il n'a été validé qu'en Expo Go sur émulateur AOSP. Il reste à le vérifier sur un build release et sur un vrai Samsung.
- **Lecture suivie : plusieurs défauts de sémantique de données** (P1) :
  - les impressions START/RESUME sont étiquetées `random_page` ;
  - un « continuer » depuis le feed aléatoire écrase la progression suivie ;
  - « Commencer depuis le début » remet silencieusement la progression à la page 1, y compris en pleine lecture suivie.
- **Retour à la découverte : bug confirmé, corrigé localement.** Au retour, la page signet revenait « non réagie », et toute nouvelle réaction donnait un 409 puis un écran d'erreur.

**Verdict : NOT READY — BLOCKERS REMAIN** (voir § 20).

---

## 2. Repo state

| Élément | État |
| --- | --- |
| Branche | `master` uniquement (locale + `origin/master`) |
| Commits locaux non poussés | 7 (`79faf6d` → `24ce418`), dont le fix Samsung `4baa3cb` |
| Fichiers modifiés non commités | 19 : backend (6 `.rs`), mobile (12), `PRIVATE_ALPHA_READINESS.md` |
| Fichiers non suivis | migration `0012`, i18n (8 fichiers), `LibraryScreen`/`ProfileScreen`/`SettingsScreen`, 3 tests jest, rapport Samsung + assets, 3 rapports JEV, `scripts/experiments/` |
| Stash | aucun |
| Artefacts ignorés présents | `mobile/bookfin-preview-v2.apk` (ignoré), `.env`, `mobile/.env` (ignorés, non suivis) |
| Artefacts DEV **suivis** | 26 captures PNG à la racine (`mobile_*.png`) + `ui.xml` (dump UIAutomator) |
| Code expérimental non lié | `scripts/experiments/jev_*.py` + `results/*.json` (benchmark TypeSafe/JEV, sans lien avec l'app ; clé lue depuis l'environnement) |

L'ensemble de la fonctionnalité (backend + mobile + migration) tient dans un seul lot non commité. **Risque** : une perte du worktree ferait perdre tout le travail. Il faut commiter avant tout build, car EAS construit à partir du worktree local.

### Cartographie

- **Mobile (Expo SDK 57, RN 0.86.3)** :
  - entrypoint `mobile/index.ts` → `App.tsx` (restauration identité/langues → `LanguageSelectionScreen` → `IdentityChoiceScreen` → `ReadingScreen`) ;
  - Profil, Bibliothèque et Réglages sont des modales ouvertes depuis `ReadingScreen` ;
  - état : `state/readingReducer.ts`, `state/useReadingSession.ts` ;
  - lecture : `components/ReadingWebView.tsx`, `lib/reader/bookfinHtml.ts`, `lib/reader/webViewBridge.ts` ;
  - i18n : `src/i18n/` (contexte React maison, 6 locales typées `Translations`).
- **Backend (Rust, axum + Leptos SSR, sqlx)** :
  - entrypoint `src/main.rs` (exécute `run_migrations` au démarrage) ;
  - routes `src/web/api_v1.rs` ;
  - use cases `src/application/use_cases.rs` ;
  - SQL `src/infrastructure/repositories.rs`.
- **Migrations** : `0001` → `0012` (voir § 10).
- **Code mort** : `AlphaGateScreen.tsx` n'est plus importé nulle part (chaînes FR codées en dur, sans impact en build).

---

## 3. Mobile architecture

| Domaine | Constat |
| --- | --- |
| Navigation | Machine d'états linéaire dans `App.tsx`, puis modales `pageSheet`. Pas de navigation impossible détectée. Le bouton « retour » Android ferme la modale (`onRequestClose`). |
| Identité anonyme | `POST /identities/anonymous`, UUID en SecureStore, restauré au démarrage. OK. |
| SecureStore | `bookfin_user_id`, `bookfin_languages`, `bookfin_app_language`. OK. |
| Langues de lecture | Persistées localement **avant** la synchro serveur (choix volontaire). Dans Réglages, un échec de `syncReadingLanguages` n'est pas intercepté (`try/finally` sans `catch`). La langue locale change alors mais pas celle du serveur, qui filtre le feed : le lecteur garde les anciennes langues sans le savoir (P2). |
| Branches DEV | `CorpusPreviewScreen` reste derrière `__DEV__ && EXPO_PUBLIC_BOOKFIN_FIXTURE_PREVIEW==='1'`. Les logs `console.*` sont tous derrière `__DEV__`/`isDevBuild()`. `getApiBaseUrl()` refuse localhost en release. OK. |
| Logs sensibles | Aucun. Les logs dev ne contiennent que des métriques numériques. |
| Persistance après relance | Identité, langues et langue d'interface persistent. Le mode lecture suivie **n'est pas** persisté : au redémarrage, on revient au feed aléatoire, la reprise passe par la Bibliothèque. C'est acceptable. |

Détails P2/P3 :

- `getAppVersions()` envoie `android.versionCode` comme `buildVersion`, y compris sur iOS (P3).
- Les messages d'erreur serveur (en français) sont affichés tels quels à l'écran (voir § 8).

---

## 4. Random feed integrity

**Chemin tracé** :

1. `fetchNextRandomPage()` → `GET /api/v1/feed/next?user_id=`.
2. `FeedNextPageUseCase` → SQL `pages` avec `is_active`, langue ∈ `user_language_preferences` (ou toutes si aucune), `NOT EXISTS page_impressions(user, page)`, `random_key >= rnd ORDER BY random_key` puis wrap-around.
3. Impression `random_page` enregistrée.
4. `POST /reactions` (validation temps/scroll, idempotence `event_id`, unicité par impression).
5. `GET /pages/:id/reveal`.
6. Suite : `POST /pages/:id/continue` ; hasard : nouvelle requête.

| Vérification | Résultat |
| --- | --- |
| Pondération par œuvres commencées | **Absente** (le SQL ne référence ni `user_work_progress` ni `reading_events`) |
| Pondération par likes / popularité / score | **Absente** |
| Personnalisation / ML | **Absente** |
| Biais START_WORK | **Absent** dans la sélection |
| Filtres | Langues éligibles + exclusion « déjà vu » (préexistant) |
| Diff du SQL feed | **Aucune modification** dans le worktree |

**Effet de bord à connaître (P3, préexistant par conception)** : les pages lues en lecture suivie reçoivent une impression. Elles sont donc exclues du tirage aléatoire ultérieur, exactement comme les pages atteintes par « continuer » aujourd'hui. Ce n'est pas un biais de sélection, mais la lecture suivie réduit le pool aléatoire d'un lecteur assidu.

Le test `feedNeutrality.test.ts` vérifie cette neutralité **par analyse statique des sources**. C'est utile, mais ce n'est pas un test comportemental.

---

## 5. Reading WebView

- Architecture conforme au Renderer V1 gelé :
  - une seule WebView montée, shell HTML statique ;
  - injection `BookfinReader.setPage` à chaque changement de payload ;
  - Poliphili en data URI (~120 Ko) ;
  - signalement `FONT_STATUS`, gestes `SWIPE_LEFT/RIGHT` ;
  - `scrollOffset` restauré via `requestAnimationFrame`.
- **Double injection** : effet React sur `payload` **et** `onLoadEnd`. Le script est gardé par `window.BookfinReader&&…`, donc une injection prématurée est sans effet et la seconde la rattrape. C'est la correction Samsung (§ 16).
- **READY handshake** : le shell émet `READY`, `parseReaderBridgeMessage` le parse, mais **`ReadingWebView` l'ignore**. Il n'existe ni vérification que la page affichée correspond bien à `page_id`, ni réinjection si `READY` n'arrive pas (P2).
- **Crash du processus de rendu** : aucun handler `onRenderProcessGone` (Android) ni `onContentProcessDidTerminate` (iOS). Si le renderer WebView meurt (mémoire, appareil Samsung d'entrée de gamme), la WebView reste vide sans récupération (P2, recommandé avant diffusion large).
- **Couleurs / mode sombre** :
  - `html, body { background: transparent }` ; le fond vient du style natif de la WebView ;
  - le texte utilise la variable `--bf-fg` injectée ;
  - aucune déclaration `color-scheme` ni `<meta name="color-scheme">` ;
  - `app.json` impose `userInterfaceStyle: "light"`, alors que le code choisit la palette via `useColorScheme()`. Cette incohérence de configuration est à trancher, et le rendu clair/sombre doit être vérifié sur Samsung, dont le « forcer le mode sombre » est réputé agressif (P2, vérification humaine).
- Aucun changement du renderer dans le worktree. Il n'a pas été rouvert.

---

## 6. Sequential reading

**Flux** :

1. « Commencer depuis le début » (`RevealBanner`, visible seulement quand `metadata.work_id && metadata.edition_id`, donc en état `revealed` ou `end_of_edition`).
2. `handleStartWork` sauvegarde le signet aléatoire (`randomFeedBookmarkRef`).
3. `POST /works/:work_id/start {user_id, edition_id}` renvoie la page 1.
4. `ENTER_SEQUENTIAL_MODE`, puis réaction et « suite » comme dans le feed. Chaque `continue` met à jour `user_work_progress`.
5. « Retour à la découverte » restaure le signet.

| Vérification | Résultat |
| --- | --- |
| Page 1 réellement servie | Oui (`page_number = 1 AND is_active`) |
| Progression séquentielle | Oui : `update_progress` après chaque `continue`, avec la page **servie** (N+1). Reprise = page servie la dernière fois. **Pas d'off-by-one.** |
| Persistance / reprise | `user_work_progress` (PK `user_id, work_id`) → `POST /works/:id/resume` |
| Retour random sans recharger une nouvelle page | Oui, le signet est réutilisé |
| **Retour random : état du signet** | **BUG CONFIRMÉ — CORRIGÉ LOCALEMENT** (voir ci-dessous) |

**Bug corrigé.**

- **Cause** : `EXIT_SEQUENTIAL_MODE` remettait `metadata`, `userReaction` et `restoreScrollOffset` à zéro, alors que le signet (`BufferedPage`) les contient. Comme « Commencer depuis le début » n'existe qu'après réaction, le signet est toujours une page déjà réagie.
- **Effet** : la page revenait en statut `reading` avec la barre Like/Dislike. Une nouvelle réaction sur la même impression était rejetée (409), ce qui menait à `REACT_ERROR`, un écran d'erreur, puis « Réessayer » tirait une autre page. Le signet était perdu et le scroll revenait en haut.
- **Correctif** :
  - `readingReducer.ts` : `EXIT_SEQUENTIAL_MODE` accepte `metadata`, `reaction` et `scrollOffset` ; statut `revealed` si des métadonnées existent ;
  - `useReadingSession.ts` : transmet ces trois champs.
- **Test ajouté** : `libraryAndFollowedReading.test.ts`, « restaure le signet révélé avec sa réaction et son scroll ».

**Défauts restants (non corrigés, décision requise)** :

| # | Constat | Sévérité |
| --- | --- | --- |
| S1 | `StartWorkUseCase` et `ResumeWorkUseCase` appellent `record_impression()`, qui insère `navigation_action = 'random_page'`, `continuation_depth = 0`, `root_page_id = page`. **Les impressions de lecture suivie sont indiscernables d'une impression aléatoire**, ce qui pollue toute analyse RANDOM_PAGE_IMPRESSION. | P1 |
| S2 | `continue_reading_handler` appelle `update_progress(user, edition)` pour **tout** `continue`, y compris depuis le feed aléatoire. Un lecteur qui a commencé l'œuvre X à la page 5 puis tombe au hasard sur la page 200 de X et fait « suite » voit sa progression sauter à 201. Elle peut aussi reculer. | P1 |
| S3 | `record_start_work` fait `ON CONFLICT DO UPDATE … current_page_sequence_number = 1`. Or `RevealBanner` affiche « Commencer depuis le début » **aussi en mode suivi** : après chaque réaction, la page 40 d'une lecture suivie propose ce bouton. Un tap remet la progression à 1 sans confirmation. Même effet depuis le feed aléatoire si l'œuvre est déjà dans la bibliothèque. Perte de progression utilisateur. | P1 |
| S4 | Un événement `start_work` est inséré à chaque appel, sans dédoublonnage serveur (le verrou client n'empêche que le double-tap). | P2 |
| S5 | En mode suivi, « Une autre page au hasard » (NavigationToolbar, y compris en fin d'édition) appelle `loadRandomPage` **sans sortir du mode suivi**. `readingMode` reste `sequential`, « Retour à la découverte » reste affiché et le signet reste en mémoire. | P2 |
| S6 | Si `startWork` ou `resumeWork` échoue, `FETCH_ERROR` s'affiche, puis « Réessayer » (`handleRetry`) tire une **nouvelle** page aléatoire au lieu de revenir au signet. | P2 |
| S7 | `work_id` (chemin) et `edition_id` (corps) ne sont pas vérifiés l'un contre l'autre côté serveur (`editions.work_id`). Un client incohérent peut créer une progression (work A, edition B). | P2 |
| S8 | En mode suivi, chaque page tournée par swipe crée un **LIKE** (grammaire existante : swipe = LIKE + suite). Les likes de lecture suivie se mêlent aux likes de découverte. Seule l'impression (`continue_book`) les distingue. C'est une décision produit : la grammaire est-elle voulue en mode suivi ? | P2 (humain) |
| S9 | `completed_at` n'est jamais renseigné (fin d'édition non répercutée dans `user_work_progress`). | P3 |

---

## 7. Library

| Vérification | Résultat |
| --- | --- |
| Œuvres commencées uniquement | Oui : `FROM user_work_progress WHERE user_id = $1`. **Pas de catalogue déguisé.** |
| Ordre | `last_read_at DESC` |
| Progression / total | `current_page_sequence_number` / `COUNT(*) pages actives de l'édition` |
| Empty state | Présent et traduit (`library.empty_title` / `empty_desc`) |
| Erreur réseau | Message traduit + « Réessayer » |
| Bouton Reprendre | `handleResumeWork` → `POST /works/:id/resume`, modale fermée |
| Persistance après relance | Côté serveur, donc oui (une fois déployé) |
| **Badge compteur** | `ProfileScreen` accepte `libraryCount`, mais `ReadingScreen` ne le passe jamais : **le badge n'apparaît jamais** (P3) |
| Œuvre commencée jamais poursuivie | Apparaît avec « Page 1 / N ». OK. |
| Progression pointant vers une page inexistante | Le resume renvoie 404 avec un message serveur brut en français (« Page X introuvable pour l'édition … ») affiché via `FETCH_ERROR`. « Réessayer » tire alors une page aléatoire. Aucun affichage n'est borné (`current > total` possible). (P2) |
| Reprendre pendant qu'une action est verrouillée | `handleResumeWork` retourne silencieusement, mais la modale est déjà fermée : rien ne se passe (P3) |

---

## 8. i18n

- **Couverture** : 6 locales (fr, en, es, ru, zh, ja), **51 clés chacune**, typées `Translations`, donc une clé manquante casse `tsc`. Vérification scriptée : 0 clé manquante, 0 valeur vide, 0 écart de placeholders (`{current}`, `{total}`, `{count}`, `{title}`, `{author}`).
- **Détection système** : `Intl…locale`, puis repli iOS/Android natif, puis `en`. Préférence manuelle persistée (`bookfin_app_language`) et prioritaire.
- **Indépendance interface / lecture** : la langue d'interface (SecureStore, contexte i18n) est distincte des langues de lecture (serveur + SecureStore). OK.
- **Flash de langue** : le contexte démarre en `en` puis bascule après lecture asynchrone de SecureStore. Un lecteur francophone voit brièvement l'anglais (P3).

**Chaînes encore codées en dur (P2)** :

| Fichier | Texte |
| --- | --- |
| `NetworkBanner.tsx` | « Réessayer » + label d'accessibilité « Réessayer laction » (apostrophe manquante) |
| `reactionErrorPolicy.ts` | `READING_VALIDATION_HINT` en français, alors que les clés `reading_too_fast_hint` / `insufficient_scroll_hint` existent dans les 6 locales **mais ne sont pas utilisées** |
| `useReadingSession.ts` | Messages de repli FR (« Impossible de charger… », « Erreur lors du chargement de la suite », « Impossible d'ouvrir le début… », « Impossible de reprendre… ») |
| `readingReducer.ts` | « Fin de cette édition » |
| `ReadingContent.tsx` | label d'accessibilité « Séparation de scène » |
| Serveur | Tous les messages d'erreur (`err.message` / `payload.message`) sont en français et remontés tels quels dans `NetworkBanner` et `NavigationToolbar` (fin d'édition) |
| `i18n/types.ts` | `SUPPORTED_APP_LANGUAGES[].label` en français (« Espagnol », « Russe »…) affiché en sous-titre quelle que soit la langue d'interface |

- **Clés inutilisées** : `reading.languages_button`, `reading.followed_reading`, `reading.reading_too_fast_hint`, `reading.insufficient_scroll_hint`.
- **Commentaires résiduels** : `ReactionToolbar.tsx` contient des commentaires JSX `{/* accessibilityLabel="…" */}` (P3).
- **Longueurs** : les chaînes les plus longues (`identity.note` es 82 car., `settings.reading_languages_desc` es 80) sont dans des blocs multi-lignes et n'ont rien d'alarmant. zh/ja sont courtes. Le seul point fragile est la barre supérieure (« Retour à la découverte » + « Profil » sur une ligne, en 13 px) : **à vérifier visuellement en ru/es sur petit écran**.

---

## 9. Backend/API

**Correspondance client → route → schéma** :

| Appel mobile | Route backend | Schéma | Statut local | Statut prod |
| --- | --- | --- | --- | --- |
| `fetchNextRandomPage` | `GET /feed/next` | FeedPageDto | ✅ | ✅ |
| `continueReading` | `POST /pages/:id/continue` | ContinueReadingRequestDto | ✅ | ✅ |
| `submitReaction` | `POST /reactions` | SubmitReactionRequestDto | ✅ | ✅ |
| `revealPageMetadata` | `GET /pages/:id/reveal` | PageRevealDto + `work_id?`, `edition_id?` | ✅ | ⚠️ sans `work_id`/`edition_id` (bouton masqué, dégradation propre) |
| `createAnonymousIdentity` | `POST /identities/anonymous` | | ✅ | ✅ |
| `syncReadingLanguages` | `PUT /users/:id/languages` | | ✅ | ✅ |
| `startWork` | `POST /works/:work_id/start` `{user_id, edition_id}` | StartWorkRequestDto | ✅ | ❌ 404 |
| `resumeWork` | `POST /works/:work_id/resume` `{user_id}` | ResumeWorkRequestDto | ✅ | ❌ 404 |
| `fetchUserLibrary` | `GET /me/library?user_id=` | `UserLibraryItemDto[]` | ✅ | ❌ 404 (HTML Leptos) |

- Les noms de champs et la nullabilité concordent :
  - `completed_at` : `Option` avec `skip_serializing_if` côté serveur, `?: string | null` côté client ;
  - `total_pages` : `i64` casté en `i32`, puis `number`.
- Erreurs possibles :
  - start avec une édition inconnue → 404 (PageNotFound) ;
  - start avec `work_id` inconnu → violation de FK → **500** (P3) ;
  - resume sans progression → 404.
- **Isolation utilisateur** : comme toutes les routes existantes, l'identité est un UUID passé en clair, sans authentification. `/me/library` retombe sur `DEFAULT_DEV_USER_ID` en l'absence de `user_id`, comme `/feed/next`. C'est cohérent avec l'alpha anonyme, et c'est un risque connu (P2 préexistant).
- `get_first_page_of_edition` et `get_page_by_edition_and_number` mappent les erreurs SQL en `NotFound`, ce qui peut masquer une vraie panne en 404 (P3, même convention que le code existant).

---

## 10. Migrations

| # | Fichier | Rôle |
| --- | --- | --- |
| 0001 | initial_schema | users, works, editions, préférences langues, extracts |
| 0002 | social_reading_pipeline | affinités, propositions de connexion |
| 0003 | page_architecture | pages, page_impressions |
| 0004 | reactions_page_compat | réactions ↔ pages |
| 0005 | continuation_and_reading_signals | chaîne de continuation, signaux, `navigation_action` |
| 0006 | hardening_and_integrity | index, unicité réaction/impression, contraintes pages |
| 0007 | reached_end_signal | `reached_end` |
| 0008 | alpha_invitations | invitations + versions app |
| 0009 | long_form_corpus_provenance | provenance corpus |
| 0010 | reader_language_preferences | préférences de langue |
| 0011 | page_content_v2 | JSON V2 + contraintes |
| **0012** | **user_work_progress** (**non suivi par git**) | `user_work_progress` + `reading_events` |

**Analyse de 0012** :

- Strictement additive : `CREATE TABLE/INDEX IF NOT EXISTS`, aucune modification de table existante. Compatible avec la prod actuelle.
- FK vers `users`, `works`, `editions` (`ON DELETE CASCADE`) et `pages` (`ON DELETE SET NULL`). Un trigger de `0006` empêche déjà de supprimer des pages ayant des impressions, ce qui limite le risque de cascade.
- Index : `(user_id, last_read_at DESC)` et `reading_events (user_id, work_id, created_at)`. Suffisants pour la bibliothèque.
- Manques (P3) :
  - pas de `CHECK (current_page_sequence_number > 0)` ;
  - pas de `CHECK` sur `event_type` ;
  - pas de contrainte garantissant que `edition_id` appartient à `work_id`.

**Points critiques pour la production** :

1. `src/main.rs` exécute `run_migrations` **au démarrage**. Le prochain déploiement Railway appliquera donc `0012` automatiquement.
2. sqlx enregistre la somme de contrôle de chaque migration. **Toute correction de `0012` (contraintes, S7…) doit être faite AVANT le premier déploiement.** Après, il faudra une migration `0013`, sinon le démarrage échoue (`VersionMismatch`).
3. `0012` n'est pas commitée : il faut la commiter avec le backend correspondant.

**Oui, le backend de production doit être migré et redéployé avant que les nouvelles fonctionnalités puissent fonctionner.**

---

## 11. Events/data model

| Événement | Stockage réel | Remarque |
| --- | --- | --- |
| RANDOM_PAGE_IMPRESSION | `page_impressions.navigation_action = 'random_page'`, depth 0 | **Contaminé** par START/RESUME (S1) |
| CONTINUE_READING | `page_impressions.navigation_action = 'continue_book'`, parent, root, depth | OK |
| LIKE / DISLIKE | `reactions` (unique par impression, `event_id` idempotent) | Distinct de START_WORK. **Pas de fusion LIKE / START_WORK.** |
| START_WORK | `reading_events.event_type = 'start_work'` + upsert `user_work_progress` | Insertion best-effort (`let _ =`) ; répété à chaque appel (S4) |
| RESUME_WORK | `reading_events.event_type = 'resume_work'` | Best-effort |
| Progression séquentielle | `user_work_progress` (état courant uniquement) | `reading_events` prévoit le type `'progress'`, mais **aucun code ne l'émet**, donc pas d'historique de progression. Écrasée par les `continue` aléatoires (S2). |

- **Double comptage** : START/RESUME créent une impression plus un événement. Ce n'est pas un double comptage si l'on distingue les impressions, ce qui n'est pas le cas aujourd'hui (S1).
- **Effet sur le feed aléatoire** : aucun sur la sélection (§ 4).
- **Données pour analyses futures** : exploitables après correction de S1 et S2. Pour distinguer « tourner la page en lecture suivie » d'un « continuer » après découverte, l'impression devrait porter l'origine (par ex. `navigation_action = 'start_work' / 'resume_work'` ou une colonne `reading_mode`).

L'algorithme affinitaire n'a pas été abordé.

---

## 12. Tests

**Exécutés** :

| Commande | Résultat |
| --- | --- |
| `npm run typecheck` | ✅ 0 erreur |
| `npm test` (jest) | ✅ 17 suites, **108 tests** (107 + 1 ajouté) |
| `npx expo-doctor` | ⚠️ 20/21 : `expo` 57.0.24 alors que `~57.0.25` est attendu (patch) |
| `cargo test --lib` | ✅ 4 tests |
| `cargo test --tests --no-run` | ✅ compilation |
| `cargo test` (Postgres local) | ✅ 54 tests (lib 4, bins 5, intégration 45) |

**Couverture** :

| Domaine | Couvert | Lacune |
| --- | --- | --- |
| Mobile : grammaire de réaction / swipe | ✅ swipeGrammar, reactionErrorRecovery, stateMachineStrictness | — |
| Mobile : WebView / race | ✅ webViewReader (partiel) | Pas de test de la course effet/`onLoadEnd` |
| Mobile : navigation arrière | ✅ swipeNavigationReset, readingScrollReset | — |
| Mobile : lecture suivie | ⚠️ reducer uniquement (4 tests) | Aucun test de `handleStartWork` / `handleResumeWork` / `handleReturnToDiscovery`, ni de S5/S6 |
| Mobile : bibliothèque | ❌ | Aucun test de `LibraryScreen` (vide, erreur, reprise) |
| Mobile : i18n | ✅ clés + détection + persistance | Pas de test anti-chaînes codées en dur |
| Mobile : feed random | ⚠️ statique (grep des sources) | — |
| Mobile : erreurs réseau | ✅ apiClient | — |
| **Backend : start / resume / library / progression** | ❌ **aucun test** | Il manque : start sur une édition inconnue, double start, `update_progress` depuis un continue aléatoire (S2), resume sans progression, isolation entre deux utilisateurs, cohérence work/edition |
| Backend : feed | ✅ integration/hardening | — |

**La lacune critique est l'absence totale de tests backend pour `WorkProgressRepository` et les trois nouvelles routes.** Aucun test n'a été ajouté côté backend : les corrections S1 à S3 changeront ces comportements, il vaut donc mieux écrire les tests avec elles.

---

## 13. Security/secrets

Scan `git grep --untracked` (hors `node_modules`, `target`, lockfiles, corpus) avec les motifs api_key, secret, token, password, bearer, `sk-`, `AKIA`, EXPO_TOKEN, APPLE_ID, private_key et URL postgres avec identifiants :

- **Aucun secret réel trouvé** dans les fichiers suivis ou non suivis.
- `.env.example` et `README.md` contiennent l'identifiant de dev local `postgres://bookfin:***@localhost` : placeholder local, acceptable.
- `.env` (localhost) et `mobile/.env` (URL Railway publique) sont ignorés et non suivis.
- `scripts/experiments/jev_*.py` lisent `TYPESAFE_API_KEY` depuis l'environnement. Aucune clé n'est présente dans les scripts ni dans `results/*.json`.
- `EXPO_PUBLIC_*` : uniquement l'URL d'API publique (et `EXPO_PUBLIC_DEV_USER_ID` dans `.env.example`, qui est l'UUID de dev public). Pas de secret embarqué dans le bundle.
- Aucun fichier de credentials (`*.p8`, `*.p12`, `*.jks`, `credentials.json`, `google-services.json`). `.gitignore` les couvre.
- `projectId` EAS et `owner` dans `app.json` : identifiants publics, pas des secrets.

---

## 14. Build config

| Élément | Valeur | Commentaire |
| --- | --- | --- |
| iOS `bundleIdentifier` | `com.bookfin.alpha` | OK |
| Android `package` | `com.bookfin.alpha` | OK |
| `version` | `0.1.0-alpha` | ⚠️ iOS exige un `CFBundleShortVersionString` purement numérique (`0.1.0`). Acceptable en Ad Hoc, **refusé par App Store Connect / TestFlight** (P2, P1 si TestFlight). |
| iOS `buildNumber` | `"2"` | À incrémenter (→ `3`) pour distinguer le nouveau build |
| Android `versionCode` | `2` | À incrémenter (→ `3`). L'APK distribué est en `2`. |
| `appVersionSource` / `autoIncrement` | absents | Numéros gérés à la main dans `app.json`. À expliciter. |
| Profils `preview` / `alpha` | `distribution: internal`, APK, API prod | OK |
| Profil `production` | pas de `distribution` (donc store) mais `buildType: apk` | Incohérent pour un store (P3). Ne pas l'utiliser pour les tests. |
| `development` | `developmentClient: true` alors que `expo-dev-client` n'est pas installé | Profil inutilisable en l'état (P3) |
| API prod | `https://bookfin-api-production.up.railway.app` dans les 3 profils | OK. `getApiBaseUrl()` refuse localhost en release. |
| DEV en prod | `CorpusPreviewScreen` exclu hors `__DEV__` | OK |
| SDK | Expo 57 / RN 0.86.3 / WebView 13.16.1 | `expo` 57.0.24 → 57.0.25 recommandé (patch) |
| `userInterfaceStyle` | `"light"` | Incohérent avec la palette sombre du code (§ 5) |

---

## 15. Production parity

| Couche | État |
| --- | --- |
| **A. Code local** | Toutes les fonctionnalités présentes, **non commitées** ; 7 commits non poussés |
| **B. Railway (prod)** | `/health` → 200. Routes existantes présentes (`GET /reactions` → 405). **`/works/:id/start`, `/works/:id/resume` et `/me/library` → 404 : non déployées.** Migration 0012 donc absente (inférence : le binaire l'appliquerait au démarrage). `reveal` ne renvoie pas `work_id`/`edition_id`. |
| **C. APK Android distribué** | Build EAS `0e0ad500…` = jalon `c8215c0`, **antérieur au fix Samsung `4baa3cb`**. Pas de lecture suivie, pas d'i18n. |
| **D. iOS** | Aucun build produit (identifiants Apple interactifs en attente selon `PRIVATE_ALPHA_READINESS.md`). |

Incohérence documentaire : `PRIVATE_ALPHA_READINESS.md` (modifié, non commité) déclare « Android real device : PASS », avec « Poliphili chargée » et « zéro flash blanc », pour **le même APK** que `ANDROID_SAMSUNG_WEBVIEW_BUG.md`, qui y reproduit une WebView vide au lancement propre. Les deux rapports ne peuvent pas être vrais simultanément pour un lancement à froid. Il faut corriger ou annoter le rapport de readiness.

---

## 16. Samsung bug status

- **Rapport** : `ANDROID_SAMSUNG_WEBVIEW_BUG.md`. La WebView est vide au premier montage, en clair comme en sombre (en sombre, le fond noir donne le symptôme Samsung). Aucun crash.
- **Cause établie** : course entre l'effet React qui injecte la page et le chargement du shell HTML. L'injection se perdait si `window.BookfinReader` n'existait pas encore.
- **Code actuel** : le correctif `onLoadEnd={injectCurrentPage}` est **présent** dans `ReadingWebView.tsx` (commit `4baa3cb`). Le code correspond au rapport.
- **Validation effectuée** : Expo Go (bundle dev) sur émulateur AOSP Pixel 8a / API 35, clair et sombre.
- **Non validé** :
  1. un **build release** (APK EAS) contenant le correctif. Le rapport lui-même exige un nouveau build ;
  2. un **appareil Samsung réel** (One UI, WebView Samsung éventuelle, « forcer le mode sombre ») ;
  3. l'absence de handler de crash du renderer et l'absence d'usage du `READY` (§ 5), qui laissent un écran vide non récupérable si le renderer meurt.

**Statut : COMPRIS et CORRIGÉ DANS LE CODE, NON VÉRIFIÉ en release ni sur Samsung.** Ce n'est pas un blocker pour *produire* un nouvel APK, puisque ce build est justement l'étape de vérification. C'en est un pour le *diffuser* aux testeurs Samsung sans rejouer les lancements à froid clair/sombre.

---

## 17. Findings by severity

### P0 — BLOCKER

| ID | Constat |
| --- | --- |
| P0-1 | Backend de production sans les routes start/resume/library ni la migration 0012. Un nouveau build mobile pointant sur la prod aurait une bibliothèque en erreur permanente, et la lecture suivie serait inaccessible. |
| P0-2 | Tout le travail (backend, mobile, migration, i18n, tests) est non commité. EAS construit depuis le worktree, donc il n'existe aucune traçabilité du binaire et le risque de perte est total. `0012` doit être figée (commitée) avant le premier déploiement (checksum sqlx). |

### P1 — IMPORTANT

| ID | Constat |
| --- | --- |
| P1-1 (S1) | Les impressions START/RESUME sont enregistrées en `random_page`, ce qui contamine les données RANDOM_PAGE_IMPRESSION. |
| P1-2 (S2) | Un `continue` depuis le feed aléatoire écrase la progression de lecture suivie de la même édition. |
| P1-3 (S3) | « Commencer depuis le début » remet silencieusement la progression à 1. Le bouton est visible en pleine lecture suivie et sur une œuvre déjà en bibliothèque. |
| P1-4 | Fix Samsung non vérifié en build release ni sur appareil Samsung (§ 16). |
| ~~P1-5~~ | ~~Retour à la découverte : signet restauré sans réaction ni révélation, d'où un 409 puis une erreur~~ : **corrigé localement** (§ 6). |

### P2 — SHOULD FIX

- S4 : START_WORK non dédoublonné.
- S5 : « page au hasard » en mode suivi sans sortie de mode.
- S6 : échec start/resume, puis « Réessayer » tire une autre page.
- S7 : work/edition non vérifiés entre eux.
- S8 : chaque page tournée en lecture suivie crée un LIKE (décision produit).
- Bibliothèque : progression hors bornes, erreur serveur brute en français.
- Réglages : échec de synchro des langues non géré (divergence local/serveur silencieuse).
- Chaînes codées en dur et messages serveur FR non traduits (§ 8).
- WebView : `READY` ignoré, pas de récupération après crash du renderer, `color-scheme` absent, `userInterfaceStyle: light` incohérent avec la palette sombre.
- `version: 0.1.0-alpha` invalide pour TestFlight.
- Aucun test backend pour les nouvelles routes.
- `PRIVATE_ALPHA_READINESS.md` contredit le rapport Samsung.

### P3 — NICE TO HAVE

- Badge bibliothèque jamais alimenté.
- Flash `en` au démarrage.
- Labels de langues en français.
- Clés i18n inutilisées.
- Commentaires JSX résiduels.
- `completed_at` jamais écrit.
- Pas de `CHECK` dans 0012.
- FK `work_id` inconnue → 500.
- Erreurs SQL mappées en 404.
- `AlphaGateScreen` mort.
- 26 PNG + `ui.xml` suivis à la racine.
- Profils EAS `production` / `development` incohérents.
- `expo` patch 57.0.25.
- `buildVersion` Android envoyé depuis iOS.
- Lecture suivie qui réduit le pool aléatoire (par conception).

---

## 18. Required actions before next Android build

1. **Décider et corriger S1, S2, S3** avant le premier déploiement, puisque ces points touchent la sémantique des données enregistrées dès le premier jour. Pistes minimales :
   - S1 : impression START/RESUME avec une `navigation_action` dédiée ;
   - S2 : ne mettre à jour la progression que si le `continue` provient d'une chaîne de lecture suivie (le client envoie le mode, ou le parent descend d'une impression START/RESUME) ;
   - S3 : masquer « Commencer depuis le début » en mode suivi, et ne pas réinitialiser une progression existante sans confirmation (ou proposer « Reprendre »).
2. **Figer `0012`** (ajouter éventuellement les `CHECK`), puis **commiter** backend, migration, mobile et tests. Pousser `master`.
3. **Ajouter les tests backend minimaux** : start, double start, resume, library, isolation utilisateur, `continue` aléatoire sans effet sur la progression.
4. **Déployer le backend sur Railway** (décision humaine ; la migration 0012 s'applique au démarrage). Vérifier ensuite en lecture :
   - `GET /api/v1/me/library?user_id=<uuid test>` → 200 `[]` ;
   - `reveal` renvoie `work_id`/`edition_id`.
5. **Incrémenter `android.versionCode` à 3** (et `ios.buildNumber` à 3).
6. Construire l'APK `preview`, puis rejouer **lancement à froid clair et sombre** sur émulateur **et sur le Samsung concerné**, avant diffusion.
7. Recommandé (P2) :
   - traduire `NetworkBanner` et l'indice de validation (les clés existent) ;
   - gérer l'échec de synchro des langues dans Réglages ;
   - sortir du mode suivi sur « page au hasard » (S5).
8. Corriger ou annoter `PRIVATE_ALPHA_READINESS.md` (Android PASS contredit par le rapport Samsung).

## 19. Required actions before iOS build

1. Toutes les actions 1 à 5 du § 18 (le backend est commun).
2. `version` → `0.1.0` (numérique) si une distribution TestFlight/App Store est envisagée. En Ad Hoc interne, ce n'est pas bloquant, mais c'est à trancher maintenant.
3. `ios.buildNumber` → `"3"`.
4. Session interactive `eas build --platform ios --profile preview` : identifiants Apple, 2FA, enregistrement de l'UDID.
5. Vérification humaine sur iPhone :
   - rendu WebView au premier lancement (l'équivalent iOS de la course a été corrigé par le même `onLoadEnd`) ;
   - clair/sombre (`userInterfaceStyle: light` vs palette) ;
   - justification ;
   - modale `pageSheet` Profil/Bibliothèque.

---

## 20. Final verdict

# NOT READY — BLOCKERS REMAIN

La base est saine : feed aléatoire intact, i18n complète, suites de tests vertes, aucun secret, cause du bug Samsung comprise et corrigée dans le code. Mais les conditions posées pour un « READY » ne sont pas remplies : **les migrations et routes requises ne sont pas en production**, et le travail n'est pas commité.

**Blockers**

- P0-1 : backend et migration 0012 non déployés en production.
- P0-2 : worktree non commité ; `0012` doit être figée avant son premier déploiement.

**Fixes nécessaires (avant de déployer, car ils fixent la forme des données)**

- S1 : impressions START/RESUME étiquetées `random_page`.
- S2 : `continue` aléatoire qui écrase la progression suivie.
- S3 : réinitialisation silencieuse de la progression par « Commencer depuis le début ».
- Tests backend des nouvelles routes.
- `versionCode` / `buildNumber` → 3.

**Points à vérifier humainement**

- Lancement à froid clair/sombre du nouvel APK sur un **vrai Samsung**.
- Décision produit : la grammaire « swipe = LIKE » s'applique-t-elle en lecture suivie (S8) ?
- Décision produit : que doit faire « Commencer depuis le début » sur une œuvre déjà commencée ?
- Mise en page de la barre supérieure en ru/es sur petit écran.
- Stratégie de version iOS (`0.1.0-alpha` vs TestFlight).
- Déploiement Railway : moment et fenêtre (migration appliquée automatiquement au démarrage).

---

### Annexe — modification apportée pendant l'audit

| Fichier | Changement |
| --- | --- |
| `mobile/src/state/readingReducer.ts` | `EXIT_SEQUENTIAL_MODE` accepte `metadata`, `reaction` et `scrollOffset` et restaure le statut `revealed` |
| `mobile/src/state/useReadingSession.ts` | `handleReturnToDiscovery` transmet ces champs depuis le signet |
| `mobile/__tests__/libraryAndFollowedReading.test.ts` | Test de régression ajouté |

Après modification : `npm run typecheck` ✅, `npm test` ✅ 108/108.

Aucun build EAS lancé. Aucun déploiement Railway. Aucune migration appliquée en production. Seules des requêtes GET en lecture seule ont été envoyées à la production.
