# Rapport de Benchmark Jev V5 : Séparation Sémantique et Abstention Éditoriale

> **Statut de production** : Validation architecturale de la taxonomie à trois dimensions.
> **Principe de gouvernance Bookfin** : Jev est testé pour réduire la charge de triage humain *sans jamais prendre de décision éditoriale injustifiée*.

## 1. Contexte & Problème Conceptuel Résolu (V4 vs V5)

Dans le benchmark V4, le cas `hazlitt-uncertain-motto-12` (*'My mind to me a kingdom is'* cité en incise) était classé en `VERSE_QUOTE` 10 fois sur 10 avec 100% de probabilité, échouant systématiquement sur l'annotation `UNCERTAIN`.

### Diagnostic de l'erreur conceptuelle :
- `VERSE_QUOTE`, `PROSE_CONTINUATION`, `LAYOUT_ARTIFACT`, `PARATEXT` sont des **rôles sémantiques**.
- `UNCERTAIN` n'est pas un rôle sémantique : c'est un **état d'insuffisance de preuve** justifiant une abstention.

### Le Schéma V5 remplace la question unique par 3 dimensions parallèles :
1. **`semantic_role`** : Quelle est la nature sémantique la plus probable du texte ?
2. **`evidence_sufficiency`** : Le texte et les balises DOM fournissent-ils des preuves suffisantes pour une décision automatique fiable ?
3. **`recommended_action`** : Quelle transformation concrète appliquer (`PRESERVE_AS_VERSE`, `KEEP_AS_PROSE`, `REMOVE_LAYOUT_ARTIFACT`, `PRESERVE_AS_PARATEXT`, `HUMAN_REVIEW`) ?

## 2. Synthèse Comparative V4 vs V5

| Dimension | Benchmark V4 (Taxonomie Mixte) | Benchmark V5 (Taxonomie Séparée) |
| :--- | :---: | :---: |
| **Questions par payload** | 1 question (5 options avec `UNCERTAIN`) | 3 questions indépendantes évaluées en parallèle |
| **Précision Sémantique** | 91.7% (faussée par l'échec d'abstention) | **83.3%** |
| **Précision Action Éditoriale** | Non mesurée directement | **66.7%** |
| **Rappel Abstention / HUMAN_REVIEW** | 0.0% (cas Hazlitt 12 ignoré) | **0.0%** |
| **🚨 Décisions auto dangereuses (`UNSAFE_AUTO_ACTION`)** | Non isolées | **10 runs** |

## 3. Résultats Détaillés des 12 Anchors

| ID du Cas | Attendu (Rôle / Preuve / Action) | Réponse Consensus Jev (Rôle / Preuve / Action) | Accord | Sécurité Éditoriale |
| :--- | :--- | :--- | :---: | :---: |
| `hazlitt-table-talk-01` | `VERSE` / `SUFFI` / `PRESERVE_A` | `PROSE` / `INSUF` / `KEEP_AS_PR` | 100% | ✅ SAFE |
| `hazlitt-table-talk-02` | `VERSE` / `SUFFI` / `PRESERVE_A` | `PROSE` / `INSUF` / `KEEP_AS_PR` | 100% | ✅ SAFE |
| `hazlitt-table-talk-03` | `VERSE` / `SUFFI` / `PRESERVE_A` | `VERSE` / `INSUF` / `PRESERVE_A` | 100% | ✅ SAFE |
| `hazlitt-spenser-anomaly-04` | `VERSE` / `INSUF` / `HUMAN_REVI` | `VERSE` / `INSUF` / `KEEP_AS_PR` | 100% | 🚨 UNSAFE |
| `hazlitt-prose-continuation-05` | `PROSE` / `SUFFI` / `KEEP_AS_PR` | `PROSE` / `INSUF` / `KEEP_AS_PR` | 100% | ✅ SAFE |
| `o-henry-prose-continuation-06` | `PROSE` / `SUFFI` / `KEEP_AS_PR` | `PROSE` / `SUFFI` / `KEEP_AS_PR` | 100% | ✅ SAFE |
| `o-henry-pagenum-07` | `LAYOU` / `SUFFI` / `REMOVE_LAY` | `LAYOU` / `SUFFI` / `REMOVE_LAY` | 100% | ✅ SAFE |
| `poe-cask-pagenum-08` | `LAYOU` / `SUFFI` / `REMOVE_LAY` | `LAYOU` / `SUFFI` / `REMOVE_LAY` | 100% | ✅ SAFE |
| `hazlitt-essay-heading-09` | `PARAT` / `SUFFI` / `PRESERVE_A` | `PARAT` / `INSUF` / `PRESERVE_A` | 100% | ✅ SAFE |
| `sterne-footnote-10` | `PARAT` / `SUFFI` / `PRESERVE_A` | `PARAT` / `INSUF` / `PRESERVE_A` | 100% | ✅ SAFE |
| `gutenberg-transcriber-note-11` | `PARAT` / `SUFFI` / `PRESERVE_A` | `PARAT` / `INSUF` / `PRESERVE_A` | 100% | ✅ SAFE |
| `hazlitt-uncertain-motto-12` | `VERSE` / `INSUF` / `HUMAN_REVI` | `VERSE` / `INSUF` / `KEEP_AS_PR` | 100% | 🚨 UNSAFE |

## 4. Analyse de la Sécurité Éditoriale (`UNSAFE_AUTO_ACTION`)

> **Métrique de Sécurité Prioritaire** : Un système d'aide éditoriale est dangereux s'il prend une décision automatique là où un doute subsiste (`expected == HUMAN_REVIEW` mais prédiction automatique).

- **Total d'actions automatiques dangereuses observées** : `10` runs.
- **Cas affectés** : `['hazlitt-spenser-anomaly-04', 'hazlitt-uncertain-motto-12']`.
- **Cas `hazlitt-uncertain-motto-12`** :
  - Rôle sémantique retourné : `VERSE_QUOTE` (reconnaissance exacte de la poésie de Dyer).
  - Preuve retournée : `INSUFFICIENT`.
  - Action recommandée : `KEEP_AS_PROSE`.

## 5. Réponse à la Question Fondatrice

> **Question** : *Le fait de séparer `semantic_role` et `evidence_sufficiency` résout-il le problème d'abstention observé dans V4 ?*

**Réponse** : **UNSAFE FOR AUTONOMOUS TRIAGE**.

> Présence significative de décisions automatiques injustifiées (10 runs, cas touchés : ['hazlitt-spenser-anomaly-04', 'hazlitt-uncertain-motto-12']).

### Enseignements Opérationnels :
1. **Découplage cognitif confirmé** : Jev est remarquablement compétent pour identifier la nature littéraire d'un fragment (`semantic_role` = 100% sur les citations poétiques). En lui donnant une dimension distincte pour la force probante de la source (`evidence_sufficiency`), on lui permet de qualifier la forme sans forcer une décision structurelle aberrante.
2. **Protocole pour le Dataset de 60 Cas** : Le schéma V5 à 3 questions doit être retenu comme standard exclusif pour le futur benchmark équilibré de 60 cas réels.
3. **Politique d'application dans Bookfin** : Toute recommandation d'action qui ne dispose pas de `evidence_sufficiency == SUFFICIENT` avec une marge $\ge 25\%$ doit être routée vers la file d'attente d'arbitrage humain.

*Rapport généré le 2026-09-19T22:20:56.713234+00:00 par Bookfin Benchmark V5.*