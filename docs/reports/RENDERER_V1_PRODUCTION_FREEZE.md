# BOOKFIN — RENDERER V1 PRODUCTION FREEZE

Date: 2026-09-19
Statut: **RENDERER_V1 = FROZEN**
Application: Bookfin Mobile (React Native / Expo SDK 57 / TypeScript)
Environnements cibles: iOS (WKWebView via `react-native-webview`), Android (Chromium WebView via `react-native-webview`)

---

## 1. DÉCLARATION FORMELLE DE GEL

Le moteur de rendu typographique et littéraire de Bookfin (**Renderer V1**) est formellement **GELÉ** pour la production.
Toute modification future de ces spécifications devra faire l'objet d'une décision d'architecture (ADR) explicite et documentée.

---

## 2. CONFIGURATION DE PRODUCTION FIGÉE

| Paramètre | Spécification de Production | Règle / Invariant |
|---|---|---|
| **Architecture du Renderer** | **Persistent WebView** (`ReadingWebView`) | Une instance unique native de WebView reste montée en permanence. Changement de page via injection JS `setPage()` sans remount (`mountCount = 1`). |
| **Police principale** | **Poliphili Roman** (embarquée sous licence) | Famille `"Bookfin Poliphili"`, injectée via Data URI Truetype dans le shell HTML initial. Fallback ordonné: `"Times New Roman", Georgia, serif`. |
| **Synthèse de fonte** | `font-synthesis: none; -webkit-font-synthesis: none;` | Aucune fausse graisse ni faux italique générés artificiellement par le moteur de rendu. |
| **Gestion de l'Italique** | Fallback gracieux sur serif natif pour `<em>` | Poliphili Roman étant chargée sans déclinaison italique dédiée, l'italique bascule sobrement sur le stack serif (Times/Georgia) sans déformation géométrique. |
| **Preset typographique** | **BOOK BALANCED** (`.bf-balanced-style`) | Prose justifiée (`text-align: justify`), espacement inter-paragraphes très faible (`margin: 0 0 0.12em`), retrait de 1ère ligne (`text-indent: 0.9em`). |
| **Césure (Hyphenation)** | **OFF par défaut** en production | `hyphens: none; -webkit-hyphens: none;`. Soft hyphens injectés = 0 en lecture standard. |
| **Patterns de césure DEV** | TeX / Liang (hypher) FR / EN / ES | Disponibles sous flag expérimental / DEV avec seuils stricts (ex: FR 4/4 ou 3/3). |
| **Blocs sémantiques V2** | Rendu HTML sémantique direct | `heading` (`<h2>`/`<h3>`), `paragraph` (`<p>`), `blockquote` (`<blockquote>`), `verse` (`<div class="bf-verse">`), `scene_break` (`<div class="bf-scene-break">`). |
| **Théâtre & Didascalies** | `bf-speaker-cue` non justifié | Les noms de personnages/répliques (`isSpeakerCue`) sont alignés à gauche (`text-align: start`), en gras sobre, sans alinéa, et neutralisent l'alinéa du premier paragraphe de dialogue suivant. |
| **Poésie / Vers** | `bf-verse` non justifié | Alignement fer à gauche (`text-align: start`), préservation des retours à la ligne (`<br>`), césure désactivée. |
| **Renderer RN Legacy** | **DEV-only** | Conservé strictement pour diagnostic ou tests locaux comparatifs. Neutralisé en production standard. Aucun repli silencieux en cas d'erreur. |

---

## 3. CONTRAT BRIDGE WEBVIEW ⇄ REACT NATIVE

Le bridge de communication bidirectionnel respecte strictement les signatures suivantes :

### Messages WebView → React Native
1. `SCROLL_STATE` : `{ type: 'SCROLL_STATE', scrollTop: number, scrollHeight: number, viewportHeight: number, atBottom: boolean }`
2. `SWIPE_LEFT` : `{ type: 'SWIPE_LEFT', atBottom: boolean }`
3. `SWIPE_RIGHT` : `{ type: 'SWIPE_RIGHT' }`
4. `FONT_STATUS` : `{ type: 'FONT_STATUS', family: string, requestedMode: 'poliphili' | 'system', romanLoaded: boolean, italicLoaded: boolean, status: 'loaded' | 'fallback' | 'not_installed' | 'italic_missing' }`
5. `READY` : `{ type: 'READY', pageId: string }`

### Injection React Native → WebView
- `window.BookfinReader.setPage({ pageId, language, html, fontMode, scrollOffset, theme })`

---

## 4. INVARIANTS TECHNIQUES SANCTUARISÉS

1. **Persistance de l'instance WebView** : `ReadingWebView` n'a pas de `key` dépendant de `page_id`. Le passage de page ne détruit ni ne recrée le process web natif.
2. **Priorité au scroll vertical** : Les listeners gestuels du bridge WebView utilisent `{ passive: true }`. Le scroll vertical natif n'est jamais volé par le swipe horizontal (ratio requis `|dx| > |dy| * 2.5` et `|dx| > 50px`).
3. **Sécurité et étanchéité** : Tout le contenu textuel passe par `escapeHtml()` avant injection. Aucune balise `<script>` n'est admise dans les blocs V2.
4. **Intégrité canonique** : Le rendu HTML ne mute jamais l'objet JSON source de la page (`FeedPageDto`).

