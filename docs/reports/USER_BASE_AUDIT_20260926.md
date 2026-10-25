# Audit Technique de la Base Utilisateurs Bookfin (Source Dump : 26 Septembre 2026)

**Date d'audit** : 25 octobre 2026  
**Source de données** : Dump certifié de production Railway du 26 septembre 2026 (`bookfin-pre-migration-0012-20260926-141822.dump`) restauré dans Neon Free PostgreSQL 18.6  
**Total utilisateurs audités** : **33**  
**Politique de confidentialité & sécurité** : Aucun UUID complet, aucune adresse IP, aucun hash sensible, aucune donnée privée exposée.  
**Règle d'intégrité** : **Aucun compte n'a été supprimé**.

---

## 1. Contexte & Limites de l'Audit

1. **Périmètre temporel** : Les données reflètent strictement l'état figé dans la sauvegarde de production Railway au **26 septembre 2026**. Les créations et activités postérieures à cette date sur Railway ne figurent pas dans cette archive.
2. **Objectif** : Distinguer techniquement et factuellement les comptes générés par des suites de tests automatisées (stress-tests, scripts d'intégration, benchmarks) des comptes associés à de vrais appareils de testeurs alpha (iOS / Android).
3. **Absence de suppression** : Tous les enregistrements restent en base. Cet audit est strictement analytique.

---

## 2. Schéma Relationnel Inspecté

La table `users` est minimale :
- `id` : `uuid` (Primary Key)
- `created_at` : `timestamptz` (défaut `now()`)

Tables associées référençant `users(id)` via Foreign Keys en cascade :
- `alpha_claimed_invites` (`user_id`, `code_id`, `claimed_at`, `app_version`, `build_version`, `device_summary`)
- `page_impressions` (`user_id`, `page_id`, `served_at`, `app_version`, `build_version`, `reading_context`, `navigation_action`, etc.)
- `reactions` (`user_id`, `page_id`, `reaction_type`, `created_at`, `reading_time_ms`, etc.)
- `user_language_preferences` (`user_id`, `language_tag`, `priority`, `created_at`)
- `user_work_progress` (introduit par la migration 0012 ; vide dans le dump pré-0012)
- `reading_events` (introduit par la migration 0012 ; vide dans le dump pré-0012)

---

## 3. Synthèse Chiffrée Globale

| Indicateur | Valeur | Note / Détail |
|---|:---:|---|
| **Total utilisateurs** | **33** | 100 % présents dans Neon |
| **Utilisateurs avec $\ge$ 1 impression** | **29** | 4 utilisateurs n'ont aucune impression de page |
| **Utilisateurs avec $\ge$ 1 réaction (Like/Dislike)** | **8** | 25 utilisateurs n'ont jamais réagi |
| **Utilisateurs avec $\ge$ 1 préférence de langue** | **31** | 2 utilisateurs n'ont pas défini de langues |
| **Utilisateurs liés à un code d'invitation Alpha** | **2** | `reader-01` (Android) et `reader-02` (iOS) |
| **Utilisateurs avec 0 activité** | **1** | User `2310ef9e` (créé en début de script) |

---

## 4. Analyse Temporelle & Origine des Comptes

La répartition chronologique par date de création fait apparaître 3 phases nettes :

```text
2026-09-12 :  2 utilisateurs (Ouverture Alpha fermée, validation invitations)
2026-09-19 : 28 utilisateurs (Validation mobile & 4 runs du script de stress test 30 pages)
2026-09-21 :  2 utilisateurs (Build 2 alpha, sessions mobiles)
2026-09-24 :  1 utilisateur  (Build 2 alpha, session mobile)
─────────────────────────────
Total      : 33 utilisateurs
```

### La vague du 19 septembre 2026 (Preuve formelle du script de test)
Entre 15:11:35 et 15:13:32 (soit **1 minute et 57 secondes**), **24 comptes** ont été créés à la chaîne :
- Tous portent les en-têtes explicites `X-App-Version: 1.0.0` et `X-Build-Version: 1`.
- Ces en-têtes correspondent exactement au script `mobile/scripts/validate_mobile_prod.mjs` (lignes 18-19).
- La structure reproduit 4 itérations complètes du script :
  1. Onboarding anonyme + langues `['fr', 'en', 'es']` (utilisateurs à 0 impression, 3 langues : `2b85aebd`, `5e013f0c`, `ef72482c`).
  2. Test des feeds FR, EN, ES (3 utilisateurs successifs à 1 impression, 1 langue).
  3. Test des réactions et du délai anti-bot 422 (1 utilisateur à 1-2 impressions, 1 réaction).
  4. **Stress-test 30 pages consécutives** : 4 utilisateurs (`54c5d46e`, `a93ca9d3`, `593e5ec2`, `d6ba6c3d`) ayant chacun **strictement 30 impressions** générées en 5 à 8 secondes.

---

## 5. Segmentation & Classification Technique

| Catégorie | Nombre | Critères objectifs |
|---|:---:|---|
| **CONFIRMED_TEST** | **24** | Créés lors du batch automatisé du 19/09 15:11-15:13, version `1.0.0`, build `1`, cadence sub-seconde, 4 comptes avec exactement 30 impressions de stress-test. |
| **POSSIBLE_TEST** | **4** | Créés le 19/09 à 11:00 (`d065a092`) et 15:06-15:07 (`94af3cda`, `7ce23004`, `ee7443ba`), sans version header, activité de moins de 30 secondes, précédant immédiatement le script automatisé. |
| **LIKELY_GENUINE** | **3** | Activité multi-jours, versions applicatives réelles `0.1.0-alpha` (build 1 et 2), interactions humaines espacées :<br>• `78a317d5` (Android 35, invité Alpha `reader-01`, actif sur 12 jours, 7 imp, 2 réactions)<br>• `f7be57e8` (iOS 26.6.1, invité Alpha `reader-02`, actif sur 2 jours, 10 imp, 7 réactions)<br>• `9f67eeee` (Build 2, actif sur 5 jours du 21 au 26/09, 9 imp, 6 réactions, 3 langues). |
| **UNKNOWN** | **2** | Enregistrés sous build 2 `0.1.0-alpha` mais avec très courte session (16s et 18min, 2 impressions chacun) : `21cb32d3` et `35c1801b`. |
| **Total** | **33** | |

---

## 6. Tableau Exhaustif des 33 Utilisateurs

*Par souci de confidentialité, seuls les 8 premiers caractères de l'UUID sont affichés.*

| User ID (8 car.) | Création (UTC) | Dernière activité | Imp. | Réactions | Langues | Invité Alpha | En-tête version | Classification |
|---|---|---|:---:|:---:|:---:|:---:|---|---|
| `78a317d5` | 2026-09-12 20:09:32 | 2026-09-24 10:48:18 | 7 | 2 | 2 | **OUI** (reader-01) | `0.1.0-alpha` / b1, b2 | **LIKELY_GENUINE** |
| `f7be57e8` | 2026-09-12 21:17:03 | 2026-09-13 13:13:41 | 10 | 7 | 0 | **OUI** (reader-02) | `0.1.0-alpha` / b1 | **LIKELY_GENUINE** |
| `d065a092` | 2026-09-19 11:00:20 | 2026-09-19 11:00:57 | 3 | 0 | 1 | NON | *aucun* | **POSSIBLE_TEST** |
| `94af3cda` | 2026-09-19 15:06:56 | 2026-09-19 15:07:20 | 2 | 1 | 2 | NON | *aucun* | **POSSIBLE_TEST** |
| `7ce23004` | 2026-09-19 15:07:23 | 2026-09-19 15:07:29 | 1 | 0 | 1 | NON | *aucun* | **POSSIBLE_TEST** |
| `ee7443ba` | 2026-09-19 15:07:31 | 2026-09-19 15:07:59 | 1 | 1 | 1 | NON | *aucun* | **POSSIBLE_TEST** |
| `2310ef9e` | 2026-09-19 15:11:35 | 2026-09-19 15:11:35 | 0 | 0 | 0 | NON | *aucun* | **CONFIRMED_TEST** |
| `711e3120` | 2026-09-19 15:11:35 | 2026-09-19 15:11:35 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `51b0dc34` | 2026-09-19 15:11:36 | 2026-09-19 15:11:36 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `1612b675` | 2026-09-19 15:11:36 | 2026-09-19 15:11:37 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `d8f422fe` | 2026-09-19 15:11:37 | 2026-09-19 15:11:42 | 1 | 1 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `54c5d46e` | 2026-09-19 15:11:42 | 2026-09-19 15:11:47 | **30** | 0 | 3 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `2b85aebd` | 2026-09-19 15:12:10 | 2026-09-19 15:12:10 | 0 | 0 | 3 | NON | *aucun* | **CONFIRMED_TEST** |
| `f69d5f2e` | 2026-09-19 15:12:11 | 2026-09-19 15:12:11 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `952a67d1` | 2026-09-19 15:12:11 | 2026-09-19 15:12:11 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `82fca899` | 2026-09-19 15:12:12 | 2026-09-19 15:12:12 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `316c1d68` | 2026-09-19 15:12:12 | 2026-09-19 15:12:18 | 2 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `a93ca9d3` | 2026-09-19 15:12:18 | 2026-09-19 15:12:23 | **30** | 0 | 3 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `5e013f0c` | 2026-09-19 15:12:54 | 2026-09-19 15:12:54 | 0 | 0 | 3 | NON | *aucun* | **CONFIRMED_TEST** |
| `302039f0` | 2026-09-19 15:12:54 | 2026-09-19 15:12:55 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `5c5dfb48` | 2026-09-19 15:12:55 | 2026-09-19 15:12:55 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `a1b61cd6` | 2026-09-19 15:12:55 | 2026-09-19 15:12:56 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `de1079c6` | 2026-09-19 15:12:56 | 2026-09-19 15:13:01 | 2 | 1 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `593e5ec2` | 2026-09-19 15:13:01 | 2026-09-19 15:13:07 | **30** | 0 | 3 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `ef72482c` | 2026-09-19 15:13:17 | 2026-09-19 15:13:17 | 0 | 0 | 3 | NON | *aucun* | **CONFIRMED_TEST** |
| `d1ee32f4` | 2026-09-19 15:13:17 | 2026-09-19 15:13:18 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `30d9dac3` | 2026-09-19 15:13:18 | 2026-09-19 15:13:18 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `b7ee8c3b` | 2026-09-19 15:13:18 | 2026-09-19 15:13:19 | 1 | 0 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `1a15ac21` | 2026-09-19 15:13:19 | 2026-09-19 15:13:24 | 2 | 1 | 1 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `d6ba6c3d` | 2026-09-19 15:13:24 | 2026-09-19 15:13:32 | **30** | 0 | 3 | NON | `1.0.0` / b1 | **CONFIRMED_TEST** |
| `9f67eeee` | 2026-09-21 16:43:10 | 2026-09-26 08:48:54 | 9 | 6 | 3 | NON | `0.1.0-alpha` / b2 | **LIKELY_GENUINE** |
| `21cb32d3` | 2026-09-21 22:13:29 | 2026-09-21 22:13:45 | 2 | 0 | 1 | NON | `0.1.0-alpha` / b2 | **UNKNOWN** |
| `35c1801b` | 2026-09-24 10:31:41 | 2026-09-24 10:49:23 | 2 | 0 | 1 | NON | `0.1.0-alpha` / b2 | **UNKNOWN** |

---

## 7. Recommandation pour un éventuel Nettoyage Futur

**AUCUN COMPTE N'EST SUPPRIMÉ À CE STADE.**

Si un nettoyage futur de la base de production devait être approuvé par l'équipe produit :
1. **Comptes éligibles à la purge** (24 comptes) : Uniquement les comptes marqués `CONFIRMED_TEST` créés par `validate_mobile_prod.mjs` le 19/09 entre 15:11 et 15:13 (en-tête `1.0.0`).
   - *Impact* : Réduction de 120 impressions de test et 4 réactions de test purement artificielles.
   - *Risque* : **Zéro** pour les vrais utilisateurs.
2. **Comptes à préserver impérativement** :
   - `78a317d5` (Testeur Alpha Android)
   - `f7be57e8` (Testeur Alpha iOS)
   - `9f67eeee` (Testeur Alpha actif)
   - `21cb32d3` et `35c1801b` (Utilisateurs réels ou développeur en condition réelle)
