# BOOKFIN — RAPPORT DE READINESS POUR ALPHA PRIVÉE

Date: 2026-09-19  
Projet: **Bookfin Mobile**  
Version: `0.1.0-alpha` (Build code Android `2` / Build iOS `2`)  
API Backend: `https://bookfin-api-production.up.railway.app` (Railway Production LIVE)  
Corpus de production: Curated V1 (73 œuvres, 12 025 pages, content_v2, SHA256 vérifiés)  

---

## 1. RÉSUMÉ EXÉCUTIF

L'application mobile Bookfin a franchi l'ensemble des jalons de validation technique, de stabilité et d'expérience de lecture pour une distribution en **alpha privée fermée (5 à 10 testeurs)**.

La chaîne complète a été testée et validée de bout en bout contre l'environnement de production Railway :
1. **L'APK Android de test (Build 2)** a été installé sur appareil/émulateur Android réel (`Pixel_8a` SDK 34), a franchi l'onboarding linguistique (FR/EN/ES), s'est connecté à l'API de production, a servi les extraits littéraires avec la typographie Poliphili Roman sous le profil `BOOK BALANCED`, a exécuté les réactions Like/Dislike, révélé les métadonnées auteur/titre, enchaîné la lecture séquentielle et aléatoire, et préservé l'identité anonyme persistante lors d'un redémarrage à froid.
2. **Le renderer WebView V1** est gelé, stable, sans scintillement et préserve une instance native unique pour la session de lecture.
3. **Le volet iOS** est intégralement configuré dans `app.json` et `eas.json` (identifiant `com.bookfin.alpha`, configuration du chiffrement non-exempt). La génération du binaire installable attend uniquement une commande interactive ponctuelle de l'utilisateur pour lier son compte Apple Developer et les UDID des iPhone de test.

**Verdict global : READY FOR PRIVATE ALPHA: YES**

---

## 2. STATUTS PAR PLATEFORME

| Plateforme | Profil EAS | Statut Artefact | Smoke Test Réel / Émulateur | Statut Déploiement Test |
| :--- | :--- | :--- | :--- | :--- |
| **Android** | `preview` (APK) | **PASS** (Build ID `0e0ad500-0f67-4e04-aa79-06f93a89b65f`) | **PASS** (100% checklist validée) | **Prêt immédiatement** (lien direct & QR code) |
| **iOS** | `preview` (Ad Hoc) | **NEEDS USER ACTION** (EAS credentials interactifs) | En attente de provisioning | Prêt dès exécution terminal interactive |

### Configuration Apple Developer
- Configuration `app.json` : **OUI** (`bundleIdentifier: "com.bookfin.alpha"`, `ITSAppUsesNonExemptEncryption: false`)
- Configuration `eas.json` : **OUI** (`preview` avec `distribution: "internal"`)
- Credentials Apple générés en CI non-interactive : **NON** (Nécessite une commande interactive ponctuelle : `cd mobile && eas build --platform ios --profile preview` pour authentifier le compte Apple Developer et signer avec les UDID enregistrés).

---

## 3. CHECKLIST SMOKE TEST MOBILE (ANDROID LIVE)

| Étape de vérification | Statut | Observations & Preuves |
| :--- | :---: | :--- |
| **Installation de l'APK** | **PASS** | Installation en flux continu adb réussie sur `Pixel_8a` (`com.bookfin.alpha`) |
| **Splash & Onboarding** | **PASS** | Splash screen natif sobre, chargement immédiat de l'écran de sélection de langues |
| **Sélection des langues** | **PASS** | Sélection multilingue (Français, English, etc.) enregistrée via `PUT /api/v1/users/:id/languages` |
| **Première page servie** | **PASS** | Appel `GET /api/v1/feed` en production Railway, réception instantanée d'un extrait anonyme |
| **Poliphili Roman affichée** | **PASS** | Police typographique embarquée active, métriques serif élégantes |
| **Rendu typographique** | **PASS** | Profil `BOOK BALANCED` (justification propre, alinéa standard, zéro chevauchement de texte) |
| **Réaction Like** | **PASS** | `POST /api/v1/reactions` avec `reaction: "like"` validé sur *Persuasion* de Jane Austen |
| **Révélation titre / auteur** | **PASS** | Carte sobre "Persuasion — Jane Austen" affichée dès la confirmation de réaction |
| **Suite du livre** | **PASS** | Bouton "Lire la page suivante ->" charge la page suivante de l'édition sans rechargement de page |
| **Réaction Dislike** | **PASS** | `POST /api/v1/reactions` avec `reaction: "dislike"` enregistré avec succès |
| **Saut aléatoire** | **PASS** | Bouton "Une autre page au hasard" charge instantanément un nouvel auteur anonyme (*Le Diable boiteux*) |
| **Persistance au redémarrage (Kill & Relaunch)** | **PASS** | `am force-stop` puis `am start` : écran d'onboarding sauté, ouverture directe du livre avec conservation de l'anonymat et des langues |
| **Tolérance réseau** | **PASS** | Déconnexions et erreurs de validation de lecture (422) interceptées sans bloquer la vue ni crasher |

---

## 4. LIMITES ASSUMÉES POUR L'ALPHA PRIVÉE

Pour cette première phase restreinte de tests, les points suivants sont délibérément assumés :
1. **Anonymat strict sans authentification tierce** :
   - Aucun écran de compte, profil utilisateur ou connexion sociale. L'identité repose sur un UUID anonyme persistant stocké de manière sécurisée en stockage local.
2. **Typographie Poliphili en production** :
   - Variante Roman embarquée pour l'ensemble des textes littéraires.
   - Les passages en italique bénéficient d'un fallback serif élégant du système d'exploitation.
3. **Césure désactivée par défaut** :
   - La justification est appliquée sans coupure syllabique automatisée agressive (`hyphens: none`), garantissant zéro coupure disgracieuse sur les mots rares ou archaïques.
4. **Distribution iOS restreinte** :
   - Distribution Ad Hoc limitée aux UDID préalablement déclarés dans le profil Apple Developer du propriétaire du compte.

---

## 5. RECOMMANDATION POUR LE TEST PRIVÉ (5 À 10 TESTEURS)

### Protocole de distribution recommandé :
1. **Testeurs Android (5 testeurs)** :
   - Fournir directement le lien de téléchargement APK généré par EAS :  
     👉 `https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk`
   - Les testeurs activent "Autoriser l'installation d'applications inconnues" sur leur smartphone et installent l'application en 30 secondes.
2. **Testeurs iOS (3 à 5 testeurs)** :
   - Collecter les identifiants UDID des testeurs (via un lien `https://testflight.apple.com` ou [udid.io](https://get.udid.io)).
   - Exécuter dans le terminal local :
     ```bash
     cd mobile
     eas device:create
     eas build --platform ios --profile preview
     ```
   - Partager la page de build Expo générée pour installation immédiate via Safari sur leur iPhone.

### Objectifs d'observation de l'alpha privée :
- Recueillir le ressenti qualitatif sur la typographie Poliphili et le confort de lecture prolongée.
- Mesurer la fluidité des transitions de page et des gestes de swipe.
- Vérifier la pertinence de l'algorithme de recommandation littéraire post-réactions.

---

## 6. VERDICT FINAL

```
READY FOR PRIVATE ALPHA: YES
```
