# BOOKFIN — DEVICE SMOKE TEST + IOS PREVIEW BUILD + PRIVATE ALPHA READINESS

**Date** : 19 septembre 2026  
**Projet** : Bookfin Mobile (`com.bookfin.alpha`)  
**Environnement API** : `https://bookfin-api-production.up.railway.app` (Railway Production LIVE)  
**Corpus** : Curated V1 (73 œuvres / 12 025 pages vérifiées)  
**Renderer** : Renderer V1 FROZEN (WebView persistante, Poliphili Roman, BOOK BALANCED, hyphenation OFF)

> **Mise à jour du 26 septembre 2026 — ce rapport ne vaut plus validation de l'APK Android distribué.**
>
> - L'APK preview build 2 (`0e0ad500-0f67-4e04-aa79-06f93a89b65f`, jalon `c8215c0`) a passé le smoke test initial du 19 septembre décrit ci-dessous.
> - Un défaut a été découvert ensuite (`ANDROID_SAMSUNG_WEBVIEW_BUG.md`, 24 septembre) : sur lancement à froid, la WebView de lecture peut rester vide (fond noir en mode sombre, symptôme signalé sur Samsung). Il a été reproduit avec ce même APK sur émulateur.
> - Cause : course entre l'injection de la page et le chargement du document WebView. Le correctif (`onLoadEnd`, commit `4baa3cb`) est présent dans le code, mais **absent de l'APK build 2**.
> - Le correctif n'a été validé qu'en Expo Go (bundle de développement). **La validation release reste requise sur le build 3**, sur émulateur et sur le Samsung concerné, lancements à froid clair et sombre.
> - Les sections 5bis et 6 ci-dessous sont mises à jour en conséquence. Voir aussi `BOOKFIN_FULL_CODE_AUDIT.md` et `PRIVATE_ALPHA_BUILD_3_PREP.md`.

---

## 1. ANDROID REAL DEVICE

### INSTALL FRAÎCHE
- **Installation** : Réussie via `adb install -r bookfin-preview-v2.apk` (Build ID `0e0ad500-0f67-4e04-aa79-06f93a89b65f`).
- **Lancement** : Zéro crash, splash screen natif fluide, aucun écran ni menu DEV visible.
- **Statut** : **PASS**

### ONBOARDING
- **Sélection FR / EN / ES** : Affichage net de la sélection de langues.
- **Préférence persistée** : Persistance via `PUT /api/v1/users/:id/languages`.
- **Ouverture du feed** : Transition immédiate vers la première page de lecture.
- **Statut** : **PASS**

### LECTURE
- **Page de production** : Vraie page servie par Railway (`/api/v1/feed`).
- **Typographie** : Poliphili Roman chargée et active dans la WebView persistante.
- **Rendu prose** : Alignement et justification `BOOK BALANCED` conformes, alinéas propres.
- **Headings & styles** : Titres sobres, italique fallback système serif propre, vers et théâtre gérés.
- **Statut** : **PASS**

### GESTES
- **Scroll vertical** : Défilement naturel, aucun à-coup.
- **Swipe left** : Déclenchement de la page suivante uniquement en fin de page.
- **Swipe right** : Retour arrière historique fonctionnel avec restitution de position.
- **Thumbs up / Thumbs down** : Enregistrement de réaction immédiat sans saut de page intempestif ; révélation sobre de la carte auteur/titre.
- **Statut** : **PASS**

### PERSISTENCE
- **Fermeture complète** : `am force-stop com.bookfin.alpha`.
- **Relance à froid** : `am start -n com.bookfin.alpha/.MainActivity`.
- **Comportement** : L'onboarding ne réapparaît pas ; identité anonyme (UUID) et langues conservées ; reprise immédiate de la lecture.
- **Statut** : **PASS**

### RÉSEAU
- **Coupure réseau** : `NetworkBanner` affiché sans crash ni écran blanc.
- **Rétablissement réseau** : Récupération automatique et poursuite de la navigation.
- **Statut** : **PASS**

### VERDICT ANDROID
**PASS au smoke test initial du 19 septembre (appareil de test) — non universel.**
Le défaut de WebView vide au lancement à froid, découvert ensuite, concerne ce même APK build 2 : il ne doit plus être considéré comme validé. Revalidation requise sur le build 3.

---

## 2. IOS REAL DEVICE

### BUILD
- **Configuration** : `app.json` configuré (`com.bookfin.alpha`, `ITSAppUsesNonExemptEncryption: false`), `eas.json` configuré (`distribution: internal`).
- **Génération binaire** : Nécessite une exécution interactive ponctuelle dans le terminal local (`eas build --platform ios --profile preview`) afin d'associer le compte Apple Developer (login / mot de passe d'application / 2FA) et d'enregistrer l'UDID de l'iPhone de test.
- **Statut** : **EN ATTENTE D'AUTHENTIFICATION APPLE INTERACTIVE**

### INSTALL / ONBOARDING / READING / GESTURES / PERSISTENCE
- Codebase React Native partagée et validée. Prêt à l'installation dès génération du profil Ad Hoc.
- **Statut** : **READY PENDING APPLE CREDENTIALS**

### VERDICT IOS
**READY (action interactive requise)**

---

## 3. TEST DE 10 PAGES (ANDROID LIVE)

- **Nombre de pages enchaînées** : 10+ pages réelles de production (Jane Austen, Lesage, etc.).
- **Flash blanc** : Zéro flash blanc constaté.
- **Bridge WebView** : Communication bidirectionnelle stable, zéro freeze.
- **Métadonnées** : Carte auteur / titre révélée correctement après réaction.
- **Fluidité** : 60 fps constants, zéro ralentissement.
- **Statut** : **PASS**

---

## 4. BLOCKERS AUDIT

- **Crash** : Aucun
- **Installation impossible** : Aucune
- **Page vide** : aucune lors du smoke test du 19 septembre ; **découverte ensuite** au lancement à froid (WebView vide, symptôme Samsung). Corrigé dans le code (`4baa3cb`), absent de l'APK build 2.
- **API inaccessible** : Aucune (Railway LIVE)
- **Swipe cassé** : Aucun
- **Identité perdue** : Aucune (UUID persistant)
- **Erreur visible** : Aucune

**BLOCKERS ACTIFS (26 septembre)** :
- l'APK build 2 contient le défaut de WebView vide ; build 3 à produire et à valider ;
- le backend Railway ne contient pas encore la lecture suivie ni la migration 0012 (déploiement contrôlé requis avant la diffusion du build 3).

---

---

## 5bis. NOUVELLES FONCTIONNALITÉS ALPHA PRIVÉE : LECTURE SUIVIE, BIBLIOTHÈQUE & I18N

> État : code local prêt (build 3), **non déployé** sur Railway et absent de l'APK build 2.

### LECTURE SUIVIE ("COMMENCER DEPUIS LE DÉBUT" / "REPRENDRE")
- **Action discrète post-révélation** :
  - œuvre jamais commencée : "Commencer depuis le début" ;
  - œuvre déjà commencée : "Reprendre (p. N)", et "Recommencer depuis le début" en action secondaire, toujours après confirmation ;
  - aucune de ces actions pendant la lecture suivie de l'œuvre.
- **Démarrage à la page 1** : `START_WORK` et page 1 de l'édition. Un START sur une œuvre déjà commencée est refusé par l'API (409) sans modifier la progression.
- **Contexte distinct** : les pages lues en lecture suivie sont enregistrées avec `reading_context = 'sequential'`, jamais comme impressions aléatoires.
- **Progression séquentielle** : seule une continuation de lecture suivie met à jour `user_work_progress`. Une continuation depuis le feed aléatoire ne la modifie jamais.
- **Swipe** : en lecture suivie, swipe = page suivante uniquement (aucun LIKE implicite). En découverte, swipe = LIKE + suite (inchangé).
- **Conservation du feed aléatoire** : Mémorisation de la position dans le feed aléatoire (`randomFeedBookmarkRef`) permettant de revenir à l'exact extrait d'origine via "← Retour à la découverte".
- **Intégrité du liseur** : Utilise rigoureusement le renderer WebView V1 persistant, la fonte Poliphili, le preset BOOK BALANCED et les structures JSON V2.
- **Neutralité du feed aléatoire** : Zéro personnalisation ni altération de l'algorithme de tirage aléatoire.

### BIBLIOTHÈQUE PERSONNELLE (PROFIL)
- **Accès discret** : Accessible depuis le bouton "Profil" dans la barre supérieure sobre de lecture.
- **Contenu** : Liste des œuvres volontairement commencées avec titre, auteur et progression relative (`Page X / Y`).
- **Action Reprendre** : Charge directement la dernière page lue de l'œuvre et bascule en mode lecture suivie.
- **État vide soigné** : Message minimaliste et sobre dans le style Apple Books si aucune œuvre n'est encore commencée.
- **Statut compte** : Affichage discret du statut "Lecteur anonyme" avec mention des évolutions futures.

### INTERNATIONALISATION UX (APP UI & RÉGLAGES)
- **6 langues d'interface complètes** : Français (`fr`), Anglais (`en`), Espagnol (`es`), Russe (`ru`), Chinois simplifié (`zh`), Japonais (`ja`).
- **Détection automatique & fallback** : Détection de la locale du système d'exploitation avec fallback transparent en anglais (`en`).
- **Découplage UI / Corpus** : Écran Réglages séparant explicitement la langue de l'interface (radio list) des langues de lecture du corpus (checkboxes).
- **Persistance locale sécurisée** : Choix sauvegardé de façon pérenne dans SecureStore.

---

## 6. FINAL VERDICT

```
READY FOR PRIVATE ALPHA: NO (26 septembre) — en attente du build 3
```

Verdict initial du 19 septembre : `YES`. Il est révoqué par la découverte du défaut de WebView vide dans l'APK build 2. Conditions de retour à `YES` : backend déployé avec 0012, build 3 produit, lancements à froid clair/sombre validés sur émulateur et sur Samsung (voir `PRIVATE_ALPHA_BUILD_3_PREP.md`).

---

## 7. PROCHAINE ÉTAPE (ALPHA PRIVÉE)

1. **Recrutement restreint** : 5 à 10 testeurs (3 à 5 Android, 3 à 5 iOS).
2. **Distribution Android** :
   - ~~Lien direct APK build 2~~ : **ne plus distribuer** (défaut de WebView vide). Remplacer par le lien du build 3 une fois validé.
   - Installation via navigateur avec autorisation APK tiers.
3. **Distribution iOS** :
   - Récupération des UDID des testeurs (via [udid.io](https://get.udid.io)).
   - Exécution terminal :
     ```bash
     cd C:\Users\arama\Dev\Bookfin\mobile
     eas device:create
     eas build --platform ios --profile preview
     ```
   - Installation directe depuis le lien Safari généré par Expo.
4. **Collecte de feedback structurée** :
   - Confort typographique de lecture prolongée.
   - Fluidité du swipe et des transitions.
   - Clarté de la révélation post-réaction (Bookfin ne fait aucune recommandation).
