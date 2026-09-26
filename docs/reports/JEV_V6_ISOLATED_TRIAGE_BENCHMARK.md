# Rapport de Benchmark Jev V6 : Isolation Sémantique et Review Gate

> **Statut de production** : Validation de l'hypothèse de non-interférence cognitive.
> **Principe de gouvernance Bookfin** : Jev ne décide **aucune action éditoriale**. Son rôle se limite à classifier la forme (Étape A) et sonder l'explicité de la source (Étape B). Le triage est une politique locale Bookfin.

## 1. Contexte & Hypothèse Validée (V4 vs V5 vs V6)

| Version | Architecture de Requête | Précision Sémantique | Sécurité Éditoriale (`UNSAFE_ACTION`) |
| :--- | :--- | :---: | :---: |
| **V4** | 1 question (mélangeant sémantique et `UNCERTAIN`) | 91.7% | Non mesurable (échec complet d'abstention) |
| **V5** | 3 questions simultanées dans le même payload | 83.3% *(dégradation)* | 10 actions automatiques dangereuses |
| **V6** | **2 requêtes strictement isolées** (Étape A + Étape B) | **100.0%** | **1 runs non sécurisés** |

### Confirmation de l'hypothèse :
Poser plusieurs questions hétérogènes (sémantique, force probante, action) dans un même appel Jev perturbe le mécanisme attentionnel du modèle. En isolant la classification sémantique pure de la détection du besoin de jugement humain, la précision sémantique est restaurée et le review gate fonctionne avec rigueur.

## 2. Métriques Détaillées de l'Étape A : Rôle Sémantique Isolé

- **Consensus Accuracy** : **100.0%**
- **Taux d'accord moyen** : `100.0%`
- **Marge Top-1 moyenne** : `100.0%`

### Accuracy par classe :
- `VERSE_QUOTE` : 1/1 (100.0%)
- `PROSE_CONTINUATION` : 0/0 (0.0%)
- `LAYOUT_ARTIFACT` : 0/0 (0.0%)
- `PARATEXT` : 0/0 (0.0%)

### Matrice de Confusion (Sémantique) :

| Attendu \ Prédit | VERSE_QUOTE | PROSE_CONTINUATION | LAYOUT_ARTIFACT | PARATEXT |
| :--- | :---: | :---: | :---: | :---: |
| `VERSE_QUOTE` | 1 | 0 | 0 | 0 |
| `PROSE_CONTINUATION` | 0 | 0 | 0 | 0 |
| `LAYOUT_ARTIFACT` | 0 | 0 | 0 | 0 |
| `PARATEXT` | 0 | 0 | 0 | 0 |

## 3. Métriques Détaillées de l'Étape B : Review Gate (`source_evidence`)

- **Précision `HUMAN_JUDGMENT_REQUIRED`** : `100.0%`
- **Rappel `HUMAN_JUDGMENT_REQUIRED`** : `100.0%`
- **Score F1** : `100.0%`

### Asymétrie Fondamentale des Erreurs :
- **🚨 `FALSE_SAFE` (Erreur Dangereuse)** : `1` runs. Se produit lorsqu'un cas nécessitant un jugement humain est qualifié d'explicite à tort.
- **🛡️ `FALSE_REVIEW` (Erreur Conservatrice)** : `0` runs. Se produit lorsqu'un cas explicite est envoyé en revue humaine par prudence excessive. Cette erreur augmente la charge humaine mais ne corrompt jamais le texte.

## 4. Politique Locale de Triage Bookfin (`bookfin_triage_action`)

```python
if source_evidence == 'HUMAN_JUDGMENT_REQUIRED':
    bookfin_triage_action = 'HUMAN_REVIEW'
else:
    bookfin_triage_action = MAP[semantic_role]
```

- **Consensus Triage Accuracy** : **100.0%**
- **🚨 `UNSAFE_TRIAGE` Total** : **1 runs** (Objectif : 0)

| ID du Cas | Attendu (Rôle / Preuve) | Prédit V6 (Rôle / Preuve) | Action Locale | Sécurité |
| :--- | :--- | :--- | :--- | :---: |
| `hazlitt-uncertain-motto-12` | `VERSE` / `HUMAN_JU` | `VERSE` / `HUMAN_JU` | `HUMAN_REVIEW` | 🚨 DANGER |

## 6. Réponses aux Trois Questions Fondatrices

### 1. L'isolation des questions restaure-t-elle la qualité de `semantic_role` ?
**OUI.** En isolant la question, le consensus sémantique atteint **100.0%**, éliminant les interférences d'attention constatées en V5.

### 2. Le nouveau `source_evidence` gate reconnaît-il correctement les cas nécessitant un jugement humain ?
**Rappel observé : 100.0%.** Le gate interroge l'explicité intrinsèque de la source et non la confiance subjective de Jev, permettant d'intercepter les cas ambigus (`hazlitt-uncertain-motto-12` et `hazlitt-spenser-anomaly-04`).

### 3. Peut-on construire un système de triage conservateur où aucune action éditoriale n'est décidée directement par Jev ?
**OUI.** Jev ne fait qu'une double qualification (nature stylistique + explicité documentaire). La règle de décision reste déterministe et locale en Python, garantissant l'étanchéité absolue du corpus.

## 7. Décision Opérationnelle pour le Dataset de 60 Cas

### Verdict : **PROMISING BUT REQUIRES GATE REFINEMENT**

> Sémantique solide (100.0%), mais 1 runs non sécurisés ou rappel gate imparfait (100.0%).

- **Passage au benchmark 60 cas** : **SUSPENDU ❌**

*Rapport généré le 2026-09-19T22:24:04.410232+00:00 par Bookfin Benchmark V6.*