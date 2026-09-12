# Application Mobile Bookfin (React Native / Expo / TypeScript)

Ce dossier contient le client mobile officiel de **Bookfin**, conçu pour une expérience de lecture littéraire immédiate, anonyme et épurée.

---

## 1. Prérequis & Installation

- **Node.js** : version 20+ (testé et certifié sous Node v24.19.0 / npm 11.17.0).
- **Backend Bookfin** : le serveur Axum doit être démarré sur le port `3000` (`cargo run`).

```bash
cd mobile
npm install
```

---

## 2. Configuration de l'Environnement

Créez un fichier `.env` à la racine de `mobile/` en vous basant sur `.env.example` :

```env
# Pour Android Emulator :
EXPO_PUBLIC_API_BASE_URL=http://10.0.2.2:3000

# Pour iOS Simulator ou Web :
# EXPO_PUBLIC_API_BASE_URL=http://localhost:3000

# Pour un appareil physique (Expo Go sur le même réseau Wi-Fi) :
# EXPO_PUBLIC_API_BASE_URL=http://192.168.1.XX:3000
```

> [!IMPORTANT]
> **Pourquoi `10.0.2.2` sur Android Emulator ?**
> Sur l'émulateur Android officiel, `localhost` ou `127.0.0.1` désigne la machine virtuelle Android elle-même. Pour contacter le serveur Bookfin qui tourne sur votre PC hôte, l'émulateur fournit l'alias réseau dédié `10.0.2.2`.

---

## 3. Lancement de l'Application

### Démarrage d'Expo (Serveur Metro)
```bash
npm start
```

### Lancement sur Android Emulator
Assurez-vous qu'un AVD Android (ex: `Pixel_8a`) est configuré ou déjà ouvert, puis :
```bash
npm run android
```

### Lancement sur un Téléphone Physique (Expo Go)
1. Installez l'application **Expo Go** (disponible sur Google Play Store ou Apple App Store).
2. Vérifiez que votre téléphone et votre PC sont connectés au **même réseau Wi-Fi**.
3. Définissez `EXPO_PUBLIC_API_BASE_URL=http://<IP_LOCALE_DU_PC>:3000` dans votre `.env`.
4. Lancez `npm start` et scannez le QR code affiché dans le terminal avec l'appareil photo (iOS) ou via l'application Expo Go (Android).

### Lancement Web (Aperçu Rapide)
```bash
npm run web
```

---

## 4. Commandes de Validation & Tests

### Vérification du typage TypeScript
```bash
npm run typecheck
```

### Exécution de la suite de tests automatisés (Jest)
```bash
npm test
```

La suite de tests valide :
- L'affichage immédiat sans écran d'accueil superflu.
- Le respect strict de l'anonymat pré-réaction (aucun titre ni auteur dans le feed).
- La soumission de préférence Like et Dislike.
- La révélation sobre post-réaction confirmée.
- La navigation *« Lire la page suivante »* (`CONTINUE_BOOK`) et *« Une autre page au hasard »* (`RANDOM_PAGE`).
- La détection et le message d'arrêt élégant lors de la fin d'une édition (`EndOfEdition`).
- La mesure locale des signaux de lecture (scroll, relecture, détection de débordement).
- La mise en pause stricte du chronomètre de lecture lors du passage en arrière-plan (`AppState`).
- L'idempotence des requêtes et la protection anti-double tap.

---

## 5. Architecture & Invariants

- **Écran unique convergent** : Aucun dashboard, catalogue ou profil au lancement.
- **Découplage strict** : Préférence (`Like`/`Dislike`) et Navigation (`Continue`/`Random`) sont deux axes totalement distincts.
- **Résilience réseau** : En cas de perte de connectivité, le bouton *Réessayer* réémet la réaction avec le même `event_id`, garantissant la non-duplication côté serveur PostgreSQL.

