# Android / Samsung WebView — diagnostic de l’APK preview

Date du test : 2026-09-24.

## Résultat

Le défaut est reproduit avec l’APK preview EAS distribué (build ID
`0e0ad500-0f67-4e04-aa79-06f93a89b65f`). Ce n’est pas un défaut de mode
sombre : le même lancement propre laisse la WebView vide en clair et en
sombre. En sombre, cette WebView vide prend simplement le fond noir attendu,
ce qui correspond au signalement Samsung.

La cause est une course au premier montage entre l’effet qui injecte la page
et le chargement du document WebView. L’injection était perdue lorsque
`window.BookfinReader` n’existait pas encore. Aucun crash JS, native, renderer
WebView ou OOM n’a été observé.

## Environnement

- AVD existant : `Pixel_8a` (`emulator-5554`), AOSP
  `sdk_gphone64_x86_64`.
- Android 15, API 35, 1080 × 2400.
- Android System WebView / Chromium : 124.0.6367.219.
- Expo Go : 57.0.9.
- Backend de développement :
  `https://bookfin-api-production.up.railway.app`.

L’APK téléchargé depuis l’URL EAS fournie fait 70 973 268 octets et son
SHA-256 est
`1BE7EEAB9011A67DBBAC1442520EDE715003B711032D62938CC2C5374405065F`.
Il est identique à `mobile/bookfin-preview-v2.apk`. Il a été réinstallé avec
`adb install -r`, sans désinstallation ni effacement de données.

## Matrice de reproduction

| Variante | Mode système | Lancement propre | Résultat |
| --- | --- | --- | --- |
| Expo Go, code actuel | clair | oui | texte lisible |
| Expo Go, code actuel | sombre | oui | texte lisible |
| APK preview EAS | clair | oui | WebView vide, fond clair |
| APK preview EAS | sombre | oui | WebView vide, fond noir ; symptôme Samsung reproduit |

Captures :

- `assets/preview-light.png` — APK EAS, clair, WebView vide.
- `assets/preview-dark.png` — APK EAS, sombre, WebView noire vide.
- `assets/expo-go-corrected-light.png` — correction, clair, texte visible.
- `assets/expo-go-corrected-dark.png` — correction, sombre, texte visible.

## Preuve de la cause

Le build distribué correspond au jalon `c8215c0` (« Freeze Renderer V1 and
prepare mobile test build for EAS distribution »). Le commit suivant
`4baa3cb` (« trigger WebView page injection on onLoadEnd to eliminate blank
initial mount ») ajoute une réinjection de la page dans
`mobile/src/components/ReadingWebView.tsx` au `onLoadEnd` de la WebView :

```tsx
onLoadEnd={injectCurrentPage}
```

Ainsi, même si l’effet React initial est exécuté avant que le shell HTML ait
défini `window.BookfinReader`, la page courante est réinjectée dès que le
document est prêt. Le commit est déjà présent dans l’arbre testé et explique
directement la différence APK preview / Expo Go.

Les observations logcat associées aux cas en échec montrent :

- `ReactNativeJS: Running "main"` ;
- création normale du processus sandboxed WebView ;
- absence de `FATAL EXCEPTION`, d’erreur `AndroidRuntime`, d’exception JS ou
  de crash du renderer.

Les messages Chromium concernant `variations_seed` et le cache HTTP sont des
messages d’initialisation non fatals ; ils apparaissent également lors des
lancements réussis et ne sont pas corrélés au contenu absent.

## Modification et validation

Aucune nouvelle modification de code n’a été faite durant ce diagnostic : la
correction ciblée existe déjà dans `4baa3cb`. Elle est validée par des
lancements Expo Go forcés en clair et en sombre avec un feed de production et
du texte visible dans la WebView.

Commandes de vérification :

- `npm run typecheck` — succès.
- `npm test -- --runInBand` — 17 suites, 107 tests réussis.
- `npx expo-doctor` — 20/21 contrôles réussis ; seul écart : `expo` 57.0.24
  alors que le SDK recommande `~57.0.25`. Cet écart de patch n’est pas lié au
  défaut de rendu reproduit.

## Suite nécessaire

Un nouveau build APK preview est requis : l’APK distribuée contient encore le
code antérieur à `4baa3cb`. Après génération, réinstaller ce nouvel APK et
rejouer les deux lancements propres clair/sombre avant diffusion à Samsung.
