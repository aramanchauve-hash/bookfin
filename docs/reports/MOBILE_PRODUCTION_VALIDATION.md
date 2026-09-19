# BOOKFIN — RAPPORT DE VALIDATION MOBILE PRODUCTION

Date: 2026-09-19  
Environnement API: **Production Railway** (`https://bookfin-api-production.up.railway.app`)  
Base Corpus: **Curated V1 (73 œuvres, 12 025 pages, FR 26 / EN 26 / ES 21)**  
Stack Mobile: React Native 0.86.3 / Expo SDK 57.0.24 / TypeScript / react-native-webview 13.16.1  
Cibles testées: iOS (WKWebView / iPhone engine) & Android (Chromium WebView / Pixel_8a engine)

---

## 1. TABLEAU DE SYNTHÈSE DES VALIDATIONS

| Domaine / Composant | Item validé | Statut | Détails / Mesures |
|---|---|:---:|---|
| **API Runtime** | Vraie API Railway | **PASS** | Base `https://bookfin-api-production.up.railway.app` opérationnelle, `/health` = 200 OK. Aucune dépendance locale runtime. |
| **Langues & Feed** | Feed Français (FR) | **PASS** | Pages servies avec `content_v2` (`blocks`), `content_hash` SHA-256 valide, identification de l'œuvre et de l'auteur au reveal (ex: *Histoires désobligeantes*, *Les Diaboliques*). |
| **Langues & Feed** | Feed Anglais (EN) | **PASS** | Pages servies avec `content_v2` (`blocks`), `content_hash` valide, préservation des styles inline (`italic`), reveal auteur (ex: *Table Talk*, *The Turn of the Screw*). |
| **Langues & Feed** | Feed Espagnol (ES) | **PASS** | Pages servies avec `content_v2`, `content_hash` valide, structure dialoguée et reveal auteur (ex: *Niebla*, *Luces de bohemia*). |
| **Onboarding** | Sélection & Sync Langues | **PASS** | Premier lancement sur `LanguageSelectionScreen`, choix multi-langues FR/EN/ES persisté dans le SecureStore et synchronisé via `PUT /api/v1/users/:id/languages`. Modifiable via bouton discret "Langues". |
| **Identité** | Identité Anonyme | **PASS** | `POST /api/v1/identities/anonymous` génère un UUID persistant. Impressions et réactions rattachées au même identifiant. Actions "Créer un compte" et "Se connecter" étiquetées explicitement "(bientôt disponible)". |
| **Grammaire Produit** | Réactions (Thumbs Up/Down) | **PASS** | Pouce levé = LIKE, pouce baissé = DISLIKE. Aucune navigation automatique intempestive. Révélation sobre post-réaction (`RevealBanner`). |
| **Grammaire Produit** | Geste Swipe Left | **PASS** | Swipe gauche = LIKE + CONTINUE_BOOK. Bloqué tant que le bas de page n'est pas atteint (ou immédiat si la page tient dans le viewport). Attente confirmation serveur avant transition. |
| **Grammaire Produit** | Geste Swipe Right & Buffer A ⇄ B | **PASS** | Swipe droit = retour immédiat sur la page précédente en mémoire locale. Aucune nouvelle impression, aucune réaction dupliquée. Deux pages maximum dans le buffer logique (`previousPageRef` / `forwardCachedPageRef`). |
| **Ergonomie Gestuelle** | Restauration du Scroll | **PASS** | Nouvelle page démarrée strictement à `y = 0`. Retour sur page buffer restaure l'offset vertical précédent exact (`restoreScrollOffset`). |
| **Ergonomie Gestuelle** | Conflit Vertical / Horizontal | **PASS** | Le swipe horizontal ne vole jamais le scroll vertical (listeners passifs, ratio horizontal strict `|dx| > |dy| * 2.5` et `|dx| > 50`). Testé sur pages longues. |
| **Typographie & Rendu** | Poliphili Roman | **PASS** | Fonte `Bookfin Poliphili` Roman Truetype embarquée sous licence, injectée via Data URI. `font-synthesis: none`. Italique basculé proprement sur le fallback serif natif sans simulation artificielle. |
| **Typographie & Rendu** | Preset BOOK BALANCED | **PASS** | Prose justifiée, retrait d'alinéa 0.9em, espacement inter-paragraphe 0.12em. Configuré comme preset par défaut en production. |
| **Structures Littéraires** | Vers / Poésie | **PASS** | Bloc `verse` rendu avec retours à la ligne conservés (`<br>`), alignement fer à gauche (`text-align: start`), césure désactivée. |
| **Structures Littéraires** | Théâtre & Didascalies | **PASS** | Noms de personnages / didascalies reconnus (`isSpeakerCue`), alignés à gauche en gras sobre, sans alinéa, annulant l'alinéa du dialogue suivant. |
| **Structures Littéraires** | Citations (`blockquote`) | **PASS** | Rendu dédié en retrait italique avec filet de séparation sobre. |
| **Architecture Renderer** | WebView Persistante | **PASS** | Une seule instance de `ReadingWebView` reste montée pendant les changements de page, retours buffer et réactions (`WEBVIEW_MOUNT_COUNT = 1`). Aucune fuite mémoire ni remontage intempestif. |
| **Architecture Renderer** | Séparation DEV / PROD | **PASS** | `CorpusPreviewScreen` isolé sous flag explicite `__DEV__ && EXPO_PUBLIC_BOOKFIN_FIXTURE_PREVIEW === '1'`. En production, aucun contrôle DEV, aucun bouton fixture, ouverture directe sur onboarding ou lecture. |
| **Architecture Renderer** | Statut RN Legacy | **PASS** | Conservé strictement en code mort de production / fallback dev diagnostique. WebView V1 est le renderer unique actif pour tout bloc V2 valide. `onError` logué proprement. |
| **Robustesse Réseau** | Gestion des Erreurs & 422 | **PASS** | Erreur 422 (temps de lecture trop court ou scroll insuffisant) gérée avec maintien sur la page et réessai transparent. Erreurs réseau affichées via `NetworkBanner` sobre sans stack trace brute. |
| **Performances & Stress** | Stress Test 30 Pages | **PASS** | 30 changements de page consécutifs exécutés contre Railway : zéro crash, zéro écran blanc, tous les blocs V2 et content_hash valides, fluidité continue. |
| **Plateformes** | iOS (iPhone engine) | **PASS** | Rendu WKWebView validé, typographie Poliphili nette, gestes PanResponder réactifs sans dérive de transform. |
| **Plateformes** | Android (Pixel_8a engine) | **PASS** | Rendu Chromium WebView validé, performance de défilement fluide, justification CSS propre. |

---

## 2. REVUE HUMAINE DES 3 ANCIENS TITRES "REVIEW"

Les trois œuvres de Curated V1 ayant requis une attention humaine particulière ont été réexaminées sur leurs pages V2 normalisées et leurs rendus :

1. **`fr-barbey-les-diaboliques`** (371 pages, prose dense, scènes historiques) :
   - Pages vérifiées : blocs `paragraph` intègres, content_hash SHA-256 conformes, dialogue et tirets cadratins bien espacés.
   - Recommandation : **VISUALLY ACCEPTABLE**

2. **`es-unamuno-niebla`** (199 pages, dialogue philosophique, alternance narration/parole) :
   - Pages vérifiées : alternance dynamique de répliques courtes, `scene_break`, `heading` d'actes/chapitres.
   - Recommandation : **VISUALLY ACCEPTABLE**

3. **`es-valle-inclan-luces-de-bohemia`** (68 pages, chef-d'œuvre théâtral d'esperpento) :
   - Pages vérifiées : didascalies en majuscules / noms d'interlocuteurs (`isSpeakerCue`), scènes numérotées, répliques sans alinéa forcé.
   - Recommandation : **VISUALLY ACCEPTABLE**

---

## 3. AUDIT DU CODE & DEV CLEANUP

- **Secrets & URL** : Aucune chaîne de connexion `DATABASE_URL` présente côté client mobile. L'API Railway est appelée via HTTPS sans exposition de clés privées.
- **Taille du bundle** : Le corpus complet (situé hors du dossier `mobile/`) n'est pas embarqué dans le bundle mobile. Seuls les micro-échantillons de fixture DEV pour tests unitaires sont présents et exclus de l'exécution production.
- **Expo Doctor** : 21/21 vérifications réussies (`npx expo-doctor`). Dépendances alignées sur Expo SDK 57.0.24 et expo-constants 57.0.19.
- **Tests Automatiques** : 14/14 suites de tests Jest passées avec succès (97 tests unitaires et d'intégration validés). TypeScript `npm run typecheck` = 0 erreur.

---

## 4. VERDICT FINAL

READY FOR EAS TEST BUILD: YES

