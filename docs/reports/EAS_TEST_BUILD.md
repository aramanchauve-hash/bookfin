# BOOKFIN — RAPPORT DE BUILD EAS DE TEST

Date: 2026-09-19  
Projet: **Bookfin Mobile**  
Compte EAS: `rmnchv` (`aramanchauve@gmail.com`) / Project ID: `aa201c92-31a3-4f32-b9df-1c7b7d20a320`  
Commit de référence: `c8215c0bf18b2c6d515de67b9dfc67bf0a2ea12b`  
Expo SDK: `57.0.24` (react-native `0.86.3`)  
Profil EAS: `preview` (Distribution interne)  
URL API cible: `https://bookfin-api-production.up.railway.app`  
Version de l'application: `0.1.0-alpha`  
Numéro de build iOS: `2` (`bundleIdentifier`: `com.bookfin.alpha`)  
Version code Android: `2` (`package`: `com.bookfin.alpha`)  

---

## 1. VÉRIFICATIONS PRÉ-BUILD

Avant le déclenchement des builds cloud EAS, l'ensemble des validations locales obligatoires a été exécuté avec succès :

- **TypeScript (`npm run typecheck`)** : PASS (0 erreur)
- **Tests unitaires & intégration (`npm test`)** : PASS (14/14 suites, 97/97 tests réussis)
- **Expo Doctor (`npx expo-doctor`)** : PASS (21/21 vérifications conformes)
- **Statut Git** : Arbre propre, commit dédié enregistré (`c8215c0`)

---

## 2. RÉSULTATS DES BUILDS EAS

### Android
- **Build ID** : `0e0ad500-0f67-4e04-aa79-06f93a89b65f`
- **Plateforme** : Android
- **Statut** : **PASS** (`finished`)
- **Profil** : `preview` (`distribution: internal`)
- **Type d'artefact** : APK directement installable (`buildType: apk`)
- **Version code** : `2`
- **Application Archive URL** : [https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk](https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk)
- **Page EAS & QR Code** : [https://expo.dev/accounts/rmnchv/projects/bookfin/builds/0e0ad500-0f67-4e04-aa79-06f93a89b65f](https://expo.dev/accounts/rmnchv/projects/bookfin/builds/0e0ad500-0f67-4e04-aa79-06f93a89b65f)

### iOS
- **Statut** : **NEEDS USER ACTION**
- **Profil** : `preview` (`distribution: internal`)
- **Raison exacte** :
  Le profil d'exportation interne Ad Hoc / Distribution interne iOS requiert l'association préalable de credentials Apple (Certificat de distribution et Profil de provisionnement Ad Hoc incluant les UDID des appareils de test enregistrés sur le compte Apple Developer du développeur).
  En mode `--non-interactive`, EAS CLI n'a pas trouvé de certificat/profil existant sur les serveurs EAS pour `com.bookfin.alpha`.
  L'utilisateur doit exécuter une fois de manière interactive dans son terminal :
  ```bash
  cd mobile
  eas build --platform ios --profile preview
  ```
  et s'authentifier auprès de son compte Apple Developer pour générer ou lier automatiquement ses certificats.
  *Note configuration :* La clé `ios.infoPlist.ITSAppUsesNonExemptEncryption: false` a déjà été configurée dans `app.json` pour éliminer toute interruption relative à la conformité du chiffrement.

---

## 3. INSTRUCTIONS D'INSTALLATION SUR APPAREIL DE TEST

### Android (Installation immédiate via APK)
1. Télécharger directement l'APK sur votre smartphone Android depuis le lien sécurisé EAS :  
   👉 [Télécharger Bookfin v0.1.0-alpha (Build 2) APK](https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk)  
   *(ou scanner le QR code disponible sur [la page de build EAS](https://expo.dev/accounts/rmnchv/projects/bookfin/builds/0e0ad500-0f67-4e04-aa79-06f93a89b65f))*.
2. Ouvrir le fichier `.apk` téléchargé sur votre téléphone.
3. Si Android affiche une alerte de sécurité, autoriser l'installation d'applications inconnues pour votre navigateur ou explorateur de fichiers.
4. Appuyer sur **Installer** puis ouvrir **Bookfin**.

### iOS (Dès synchronisation interactive des credentials Apple)
1. Enregistrer au préalable l'UDID de l'iPhone de test via `eas device:create` ou directement sur le portail Apple Developer.
2. Lancer `eas build --platform ios --profile preview` depuis votre terminal interactif.
3. Scanner le QR code généré par Expo pour installer le profil et l'application via Safari mobile.

---

## 4. CHECKLIST DU SMOKE TEST SUR APPAREIL RÉEL

1. [ ] Lancement de l'application
2. [ ] Affichage de l'écran d'onboarding au premier démarrage
3. [ ] Choix des langues (FR / EN / ES) et validation
4. [ ] Arrivée directe sur le premier extrait littéraire
5. [ ] Typographie Poliphili Roman active, visible et nette
6. [ ] Défilement vertical fluide
7. [ ] Pouce levé (Like) -> révélation sobre de l'auteur
8. [ ] Pouce baissé (Dislike) -> révélation sobre de l'auteur
9. [ ] Swipe gauche après avoir atteint le bas de page -> page suivante (Like + Continue)
10. [ ] Swipe droit -> retour immédiat sur la page précédente en mémoire locale
11. [ ] Changement de langue via le bouton discret "Langues"
12. [ ] Fermeture forcée et relance de l'application -> reprise directe de la lecture avec préférences conservées
13. [ ] Absence complète des contrôles de débogage et des bandeaux DEV

---

READY FOR DEVICE TEST DISTRIBUTION: YES
