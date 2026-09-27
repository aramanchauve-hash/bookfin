# BOOKFIN — RAPPORT DE BUILD EAS DE TEST

Date: 27 septembre 2026 (Mise à jour Build 3)  
Projet: **Bookfin Mobile**  
Compte EAS: `rmnchv` (`aramanchauve@gmail.com`) / Project ID: `aa201c92-31a3-4f32-b9df-1c7b7d20a320`  
Commit de référence: `f07b7ea0deb63bbf0e569b4dc0e51f82ecf12aa2`  
Expo SDK: `57.0.24` (react-native `0.86.3`)  
Profil EAS: `preview` (Distribution interne)  
URL API cible: `https://bookfin-api-production.up.railway.app` (Railway Production avec migration 0012)  
Version de l'application: `0.1.0`  
Numéro de build iOS: `3` (`bundleIdentifier`: `com.bookfin.alpha`)  
Version code Android: `3` (`package`: `com.bookfin.alpha`)  

---

## 1. VÉRIFICATIONS PRÉ-BUILD

- **Backend Railway** : Migration 0012 déployée, /health 200, routes start/resume/library opérationnelles.
- **Git** : Working tree propre sur master (`f07b7ea`).
- **Configuration** : `appVersionSource: local`, `android.versionCode: 3`, `ios.buildNumber: 3`.

---

## 2. RÉSULTATS DES BUILDS EAS (BUILD 3)

### Android Build 3
- **Build ID** : `bb40bb1d-ab3b-44ed-a0e6-1088a07b22eb`
- **Plateforme** : Android
- **Statut** : **PASS** (`FINISHED`)
- **Profil** : `preview` (`distribution: internal`)
- **Type d'artefact** : APK directement installable (`buildType: apk`)
- **Version code** : `3` (version `0.1.0`)
- **Application Archive URL** : [https://expo.dev/artifacts/eas/FQmx2KDO3G7ee1ESldR9TndlEwx4OE1gkKukl1lQhjY.apk](https://expo.dev/artifacts/eas/FQmx2KDO3G7ee1ESldR9TndlEwx4OE1gkKukl1lQhjY.apk)
- **Fichier local** : `mobile/builds/bookfin-v0.1.0-build3.apk` (71 018 176 octets, SHA-256 `41C06D2277A07597878917213118D35222AF74023477918014B88531C029A824`)

### iOS
- Non lancé dans cette mission (conformément aux consignes).

---

## 3. VALIDATION ÉMULATEUR ANDROID (ENVIRONNEMENT DE REPRODUCTION DU BUG)

Testé sur `emulator-5554` (AVD `Pixel_8a`, API 35 — environnement exact où le bug WebView noir sur Build 2 avait été reproduit) :

- **Lancement à froid Light Mode** : **PASS** (Poliphili chargée, texte immédiatement visible, 0 écran blanc/noir)
- **Lancement à froid Dark Mode** : **PASS** (Fond sombre, texte ivoire net, 0 écran noir)
- **Scroll vertical & gestes** : **PASS**
- **Like / Révélation titre & auteur** : **PASS**
- **Commencer depuis le début (page 1)** : **PASS**
- **Swipe séquentiel (page 2)** : **PASS** (progression incrémentée sans like)
- **Retour à la découverte** : **PASS**
- **Bibliothèque (Profil)** : **PASS** (reprise directe à la page 2)
- **Réglages & langues UI/Corpus** : **PASS**

---

## 4. VALIDATION SAMSUNG RÉEL

- En attente de déploiement et test sur l'appareil physique Samsung concerné.
- L'APK peut être installé directement via :
  ```bash
  adb install -r mobile/builds/bookfin-v0.1.0-build3.apk
  ```
  ou téléchargé directement sur le téléphone via le lien EAS :
  [Télécharger Bookfin v0.1.0 Build 3 APK](https://expo.dev/artifacts/eas/FQmx2KDO3G7ee1ESldR9TndlEwx4OE1gkKukl1lQhjY.apk)

---

READY FOR DEVICE TEST DISTRIBUTION: YES (Build 3 prêt pour installation Samsung)

