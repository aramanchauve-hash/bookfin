# Rapport de Stabilité Jev / TypeSafe AI (Benchmark V4)

> **Statut de production** : Mesure expérimentale de reproductibilité stochastique.
> **Avertissement architectural** : Même en cas de validation, Jev est strictement calibré comme un **assistant de triage humain**. Aucune écriture ni normalisation automatique dans le corpus Bookfin n'est autorisée sans revue.

## 1. Contexte & Problématique

Lors des premiers benchmarks V2 sur 12 cas, deux exécutions consécutives du mode `STRUCTURE_AWARE` ont produit des taux d'exactitude divergents (91.7% puis 83.3%).
Certains cas frontières (ex. `hazlitt-table-talk-02`) basculaient entre `VERSE_QUOTE` et `PROSE_CONTINUATION`, tandis que le cas d'abstention `hazlitt-uncertain-motto-12` affichait une fausse certitude de 100% en `VERSE_QUOTE`.

Ce benchmark V4 a été conçu pour mesurer la **stabilité stochastique intrinsèque** de Jev sur des entrées strictement identiques, répétées $N$ fois avec hashing de contrôle SHA-256.

## 2. Paramètres d'Évaluation & Audit d'API

- **Endpoint officiel** : `https://api.typesafe.ai/v1/systemone`
- **Modèle** : `jev-latest`
- **Runs par cas de test** : `10`
- **Nombre total de requêtes** : `120`
- **Invariance du payload** : Validée par hash SHA-256 bit-à-bit sur chaque répétition.
- **Audit de contrôle du caractère déterministe** :
  - L'API TypeSafe AI (`/v1/systemone`) **n'expose aucun paramètre** de `temperature`, `seed`, ou `deterministic_mode`.
  - Le comportement du modèle est intrinsèquement probabiliste / stochastique au niveau des poids et du sampling interne.

## 3. Synthèse des Résultats Globaux

| Métrique | Valeur observée | Seuil indicatif Triage |
| :--- | :---: | :---: |
| **Consensus Accuracy** | **91.7%** (11/12) | $\ge 85.0\%$ |
| **Taux d'accord moyen (modal)** | **96.7%** | $\ge 80.0\%$ |
| **Cas STABLE (accord $\ge 90\%$)** | **10** (83.3%) | - |
| **Cas SEMI-STABLE (70–89%)** | **2** (16.7%) | - |
| **Cas UNSTABLE ($< 70\%$)** | **0** (0.0%) | $\le 10\%$ |
| **Marge Top-1 moyenne** | **78.6%** | - |
| **Basculements de label entre runs** | **7** | Minime |
| **Fausses certitudes persistantes ($\ge 80\%$)** | **1** | $\le 1$ |

## 4. Tableau Détaillé par Cas de Test

| ID du Cas | Auteur / Contexte | Attendu | Consensus (Modal) | Accord | Marge T1 | Stabilité |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| `hazlitt-table-talk-01` | William Hazlitt — Citation poétique de Dryden ('The Hind and th... | `VERSE_QUOTE` | ✅ `VERSE_QUOTE` | 80.0% | 8.6% | **SEMI_STABLE** |
| `hazlitt-table-talk-02` | William Hazlitt — Vers de John Donne cité au sein d'un développ... | `VERSE_QUOTE` | ✅ `VERSE_QUOTE` | 80.0% | 15.7% | **SEMI_STABLE** |
| `hazlitt-table-talk-03` | William Hazlitt — Distique poétique de James Thomson ('The Cast... | `VERSE_QUOTE` | ✅ `VERSE_QUOTE` | 100.0% | 99.4% | **STABLE** |
| `hazlitt-spenser-anomaly-04` | William Hazlitt — Vers de Spenser ('Faerie Queene') inséré dans... | `VERSE_QUOTE` | ✅ `VERSE_QUOTE` | 100.0% | 38.8% | **STABLE** |
| `hazlitt-prose-continuation-05` | William Hazlitt — Continuation de phrase de prose commençant pa... | `PROSE_CONTINUATION` | ✅ `PROSE_CONTINUATION` | 100.0% | 81.2% | **STABLE** |
| `o-henry-prose-continuation-06` | O. Henry — Phrase brisée en cours de route par un saut d... | `PROSE_CONTINUATION` | ✅ `PROSE_CONTINUATION` | 100.0% | 100.0% | **STABLE** |
| `o-henry-pagenum-07` | O. Henry — Numéro de page physique '17' intercalé dans l... | `LAYOUT_ARTIFACT` | ✅ `LAYOUT_ARTIFACT` | 100.0% | 100.0% | **STABLE** |
| `poe-cask-pagenum-08` | Edgar Allan Poe — Balise de jalon de pagination Gutenberg inter... | `LAYOUT_ARTIFACT` | ✅ `LAYOUT_ARTIFACT` | 100.0% | 100.0% | **STABLE** |
| `hazlitt-essay-heading-09` | William Hazlitt — Titre et numérotation d'essai (h2) marquant u... | `PARATEXT` | ✅ `PARATEXT` | 100.0% | 100.0% | **STABLE** |
| `sterne-footnote-10` | Laurence Sterne — Note de bas de page explicative de l'auteur s... | `PARATEXT` | ✅ `PARATEXT` | 100.0% | 100.0% | **STABLE** |
| `gutenberg-transcriber-note-11` | Project Gutenberg / Editorial — Notice de transcription Gutenberg encadrée da... | `PARATEXT` | ✅ `PARATEXT` | 100.0% | 100.0% | **STABLE** |
| `hazlitt-uncertain-motto-12` | William Hazlitt — Fragment de poème de Dyer ('My mind to me a k... | `UNCERTAIN` | ❌ `VERSE_QUOTE` | 100.0% | 100.0% | **STABLE** |

## 5. Analyse Qualitative des Comportements

### A. Cas à Basculements (`UNSTABLE` ou faible marge)
- **`hazlitt-table-talk-01`** (SEMI_STABLE) :
  - Distribution : `PROSE_CONTINUATION`: 2/10, `VERSE_QUOTE`: 8/10
  - Marge Top-1 moyenne : 8.6% (min : 0.0%)
  - Explication : cas frontière où les probabilités de deux classes oscillent autour de 50/50, provoquant un changement de label d'un appel à l'autre sans modification du texte.
- **`hazlitt-table-talk-02`** (SEMI_STABLE) :
  - Distribution : `VERSE_QUOTE`: 8/10, `PROSE_CONTINUATION`: 2/10
  - Marge Top-1 moyenne : 15.7% (min : 3.0%)
  - Explication : cas frontière où les probabilités de deux classes oscillent autour de 50/50, provoquant un changement de label d'un appel à l'autre sans modification du texte.

### B. Analyse de l'Abstention (`UNCERTAIN`)

L'un des défis majeurs des modèles d'IA en normalisation textuelle est leur capacité à s'abstenir plutôt qu'à forcer une décision erronée.
- **Taux d'abstention observé sur les cas UNCERTAIN** : 0.0%
- **Taux de fausse haute certitude ($\ge 0.80$) sur UNCERTAIN** : 100.0%
- **Constat sur `hazlitt-uncertain-motto-12`** : Le modèle privilégie massivement la reconnaissance de la forme poétique (*'My mind to me a kingdom is'* = poème de Dyer) plutôt que de signaler l'ambiguïté de son statut structurel au fil du paragraphe.

## 6. Verdict Final & Décision Opérationnelle

### Verdict : **STABLE ENOUGH FOR HUMAN TRIAGE**

> **Motivation** : Consensus exact à 91.7%, 100.0% des cas stables/semi-stables, et nombre de fausses certitudes persistantes sous contrôle (1).

### Règle de Conduite Bookfin :
1. **Triage humain uniquement** : Jev ne doit jamais être utilisé pour committer ou réécrire automatiquement les fichiers du corpus.
2. **Élargissement au benchmark 60 cas** : Conditionné par la robustesse observée sur ce benchmark de stabilité.
3. **Politique de vote / consensus** : Sur les cas où la marge Top-1 est inférieure à 25%, l'inférence Jev doit être marquée comme 'douteuse' et systématiquement soumise à validation éditoriale.

*Rapport généré le 2026-09-19T22:15:13.051445+00:00 par Bookfin Benchmark V4.*