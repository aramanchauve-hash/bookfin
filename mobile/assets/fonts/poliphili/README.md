# Poliphili Font Assets

Cet emplacement est réservé aux fichiers de fonte de lecture pour l'identité Bookfin :

- Poliphili-Regular.woff2 (ou .ttf / .otf) — Style Roman / Regular
- Poliphili-Italic.woff2 (ou .ttf / .otf) — Vraie Italique (Blado ou équivalent)

## Informations Typographiques & Propriété Intellectuelle

- Caractère : Poliphili (Monotype Series 332, Poliphilus & Blado Italic).
- Origine : Dessinée en 1923 sous la direction de Stanley Morison d'après les types d'Alde Manuce et Francesco Griffo (Venise, 1499, Hypnerotomachia Poliphili).
- Fonderie / Droits : Propriété de Monotype Imaging Inc. (ou Berthold pour Poliphilus BQ).
- Licence requise : Mobile App Embedding License (ou accord spécifique pour incorporation logicielle mobile).

## Instructions d'Intégration

Dès acquisition de la licence d'embarquement applicatif :
1. Déposer les fichiers de fontes sous licence dans ce dossier.
2. Mettre à jour mobile/src/lib/reader/bookfinFont.ts avec les Data URIs base64 correspondantes (ou loader d'asset local offline).
3. Le renderer WebView chargera automatiquement la police via @font-face sans modification du corpus ni des hashes.
