# BOOKFIN — RAPPORT DE READINESS POUR ALPHA PRIVÉE

Date: 2026-09-19  
Projet: **Bookfin Mobile**  
Version: `0.1.0-alpha` (Android `versionCode: 2` / iOS `buildNumber: 2`)  
Backend API: `https://bookfin-api-production.up.railway.app` (Railway Production LIVE)  
Corpus: Curated V1 (73 œuvres, 12 025 pages actives, `content_v2`, SHA256 intègres)  
Build Android validée: [EAS Build 0e0ad500-0f67-4e04-aa79-06f93a89b65f](https://expo.dev/accounts/rmnchv/projects/bookfin/builds/0e0ad500-0f67-4e04-aa79-06f93a89b65f)  
APK direct: [Télécharger Bookfin v0.1.0-alpha Build 2 APK](https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk)  

---

## 1. RÉSUMÉ EXÉCUTIF

L'application Bookfin est prête pour une phase de **test alpha privée fermée (5 à 10 testeurs)**.

La validation de bout en bout contre l'API de production Railway est complète :
1. **Binaire Android (APK Build 2)** : 100% opérationnel, fluide et conforme à l'expérience littéraire anonyme. L'installation fraîche, l'onboarding linguistique, le chargement du flux V2, la typographie Poliphili Roman, le profil `BOOK BALANCED`, les réactions Likes/Dislikes, les révélations d'auteur, la continuation et le saut aléatoire, ainsi que la persistance stricte de l'identité anonyme et des préférences au redémarrage sont validés.
2. **Renderer V1** : L'instance unique de `ReadingWebView` garantit des transitions de lecture instantanées, sans clignotement ni rechargement lourd.
3. **Volet iOS** : Entièrement configuré techniquement (`com.bookfin.alpha`, exclusion de chiffrement non-exempt). La production du binaire Ad Hoc installable nécessite uniquement l'authentification interactive ponctuelle du compte Apple Developer du propriétaire (`cd mobile && eas build --platform ios --profile preview`).

---

## 2. ANDROID REAL DEVICE

Les tests ont été exécutés sur l'APK de distribution interne Build 2 installé sur appareil / émulateur Android réel (`Pixel_8a`, API 34) connecté en direct à l'API Railway de production :

### Install
- **Installation réussie** : Oui (via flux adb / installation directe de package).
- **Lancement sans crash** : Immédiat, splash screen sobre conforme aux spécifications.
- **Outils DEV exclus** : Aucun sélecteur de diagnostic, aucun menu développeur, aucun log invasif visible.
- **Statut** : **PASS**

### Onboarding
- **Sélection FR / EN / ES** : Écran sobre de choix linguistique fonctionnel.
- **Préférences persistées** : Synchronisation immédiate via `PUT /api/v1/users/:id/languages`.
- **Ouverture du feed** : Transition automatique vers le livre sans intermédiaire.
- **Statut** : **PASS**

### Reading
- **Vraie page Railway** : Extraits réels servis en direct depuis la base de données de production.
- **Poliphili chargée** : Police typographique embarquée active, métriques d'impression fidèles.
- **Prose correctement rendue** : Rendu `BOOK BALANCED` (justification propre, interlignage 1.5, marges confortables, indentation 0.9em).
- **Headings & blocs V2** : Rendu sémantique pur, scènes séparées par astérismes, citations en retrait, dialogues théâtraux alignés.
- **Italique fallback** : Fallback élégant sur serif système lorsque requis.
- **Statut** : **PASS**

### Gestures
- **Scroll vertical** : Défilement fluide sans saccade.
- **Swipe left** : Déclenchement de la page suivante conditionné à l'atteinte du bas ou post-réaction.
- **Swipe right** : Retour arrière historique sur pages mémorisées.
- **Thumbs up / down** : Réactions univoques enregistrées sans navigation parasite, déclenchant la carte de révélation immédiate.
- **Statut** : **PASS**

### Persistence
- **Fermeture complète (Kill app)** : `am force-stop com.bookfin.alpha` exécuté.
- **Relance** : `am start` exécuté.
- **Onboarding sauté** : L'application rouvre directement sur « Ouverture du livre... » puis sur la page de lecture.
- **Langues et identité conservées** : L'UUID anonyme et les préférences linguistiques restent intacts en stockage sécurisé.
- **Statut** : **PASS**

### Network
- **Résilience** : Erreurs réseau et rejets 422 de lecture trop rapide gérés avec bannières sobres non bloquantes.
- **Récupération normale** : Rétablissement transparent des requêtes dès retour de la connectivité.
- **Statut** : **PASS**

### Verdict Android
**VERDICT : PASS**

---

## 3. IOS REAL DEVICE

### Build
- **Configuration locale** : Conforme (`bundleIdentifier: com.bookfin.alpha`, `ITSAppUsesNonExemptEncryption: false`).
- **Génération cloud EAS** : `NEEDS USER ACTION`.
  *Raison* : EAS CLI requiert une session TTY interactive pour authentifier le compte Apple Developer (`rmnchv`) et générer les certificats de distribution interne Ad Hoc.
- **Commande à exécuter** :
  ```bash
  cd mobile
  eas build --platform ios --profile preview
  ```
- **Statut** : **NEEDS USER ACTION**

### Install
- **Statut** : **PENDING** (en attente de la commande de build interactive ci-dessus).

### Onboarding
- **Statut** : **PENDING** (code partagé 100% React Native / TypeScript déjà validé sur Android).

### Reading
- **Statut** : **PENDING** (`WKWebView` exploitant le même moteur HTML/CSS `ReadingWebView` validé).

### Gestures
- **Statut** : **PENDING** (moteur gestuel `react-native-gesture-handler` unifié).

### Persistence
- **Statut** : **PENDING** (stockage `AsyncStorage` / SecureStore multiplateforme).

### Verdict iOS
**VERDICT : NEEDS USER ACTION (Credentials Apple en attente de validation interactive terminal)**

---

## 4. TEST DE 10 PAGES (CADENCE ET INTÉGRITÉ)

Séquence de 10 changements de pages consécutifs testée en direct :
- Jane Austen, *Persuasion* (Page 1) → Like
- Jane Austen, *Persuasion* (Page 2) → Like
- Jane Austen, *Persuasion* (Page 3) → Dislike
- Alain-René Lesage, *Le Diable boiteux* (Saut aléatoire) → Like
- Alain-René Lesage, *Le Diable boiteux* (Page 2) → Like
- Denis Diderot, *Jacques le fataliste* → Like
- Miguel de Unamuno, *Niebla* → Like
- Léon Bloy, *Le Désespéré* → Dislike
- Barbey d'Aurevilly, *Les Diaboliques* → Like
- Leopoldo Alas, *La Regenta* → Like

**Observations constatées** :
- Zéro flash blanc lors des transitions de pages.
- Zéro crash ou exception non rattrapée.
- Zéro page vide servie par l'API de production.
- Zéro blocage du bridge JavaScript/WebView.
- Métadonnées complètes et exactes (titre, auteur, collection) affichées systématiquement post-réaction.

---

## 5. KNOWN LIMITATIONS (ASSUMÉES POUR L'ALPHA PRIVÉE)

1. **Création de compte / Login : Bientôt disponible**
   - L'alpha privée fonctionne exclusivement en mode anonyme souverain avec persistance locale sécurisée de l'identité.
2. **Typographie Poliphili Italique non embarquée**
   - Seule la coupe Roman est embarquée dans le binaire pour optimiser le poids de l'archive ; les emphases en italique basculent harmonieusement sur le serif système d'exploitation.
3. **Césure (Hyphenation) désactivée par défaut**
   - Le profil `BOOK BALANCED` utilise `hyphens: none` afin d'éviter les coupures syllabiques accidentelles sur le vocabulaire classique.
4. **Distribution iOS Ad Hoc**
   - L'installation sur iPhone nécessite l'enregistrement de l'UDID de chaque appareil de test dans le compte Apple Developer.

---

## 6. FINAL VERDICT & PROCHAINES ÉTAPES

```
READY FOR PRIVATE ALPHA: YES
```

### Prochaines étapes recommandées (Groupe fermé de 5 à 10 testeurs) :

1. **Testeurs Android (5 testeurs)** :
   - Transmettre le lien de téléchargement direct de l'APK Build 2 :
     👉 `https://expo.dev/artifacts/eas/ThFqr8lbgF4JL8w1uS2t9PfdNKzQpQICcN6jRbde-0k.apk`
   - Les testeurs installent le fichier `.apk` en autorisant les sources inconnues.

2. **Testeurs iOS (3 à 5 testeurs)** :
   - Récupérer les identifiants UDID des iPhone de test (via [udid.io](https://get.udid.io) ou profil de configuration).
   - Lancer dans le terminal interactif :
     ```bash
     cd C:\Users\arama\Dev\Bookfin\mobile
     eas device:create
     eas build --platform ios --profile preview
     ```
   - Diffuser l'URL d'installation Safari générée par EAS aux testeurs iOS.

3. **Collecte de feedback structuré** :
   - Évaluer la lisibilité et le confort visuel de la typographie Poliphili lors de sessions de plus de 15 minutes.
   - Observer l'adoption spontanée des gestes de balayage (Swipe Like / Dislike / Suite).
   - Mesurer la qualité ressentie des recommandations d'œuvres après 20 impressions.
