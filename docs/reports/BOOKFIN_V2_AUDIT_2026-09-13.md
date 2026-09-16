# Audit Bookfin V2 — 13 septembre 2026

## Lecture mobile

Le swipe initial capturait PanResponder puis déplaçait le contenu sous le
doigt. Sur iOS, la chaîne de responders du ScrollView peut retenir le geste ou
laisser une transform après une terminaison. La modification en cours ne
revendique qu'un geste gauche franchement horizontal, ne suit plus le doigt,
remet la transform à l'identité avant navigation et verrouille les répétitions.
Le scroll vertical reste gagnant.

La conservation de l'offset vient de ReadingContent : son ScrollView est
conservé quand page_id change. Le correctif restant doit appeler scrollTo y=0
au changement de page_id, avant le nouveau rendu visible.

La colonne, police serif, taille 18,5 et line-height 30 sont corrects.
textAlign justify aide les écritures compatibles mais ne répare pas les
paragraphes, emphases ou vers perdus par TXT; il ne faut pas l'imposer aux
écritures zh ou ja.

## Corpus et provenance constatés

Les corpus présents ne contiennent que des TXT. Les scripts récupèrent et
nettoient du texte brut : Gutenberg pour le long format et HTML Wikisource
aplati vers TXT pour une partie du pilote. Les manifests conservent URL, droits
et SHA-256 mais aucun original HTML ou EPUB. Titres, italiques, vers, citations,
séparateurs et notes peuvent donc être perdus. Aucune source riche locale ne
peut être reconstruite sans une acquisition vérifiée; aucune n'est inventée.

Le contrat et le prototype V2 sont dans docs/architecture/corpus_v2.md et
scripts/corpus_v2.py. Aucun gros réimport n'est réalisé.

## Langues et identité

La table user_language_preferences existait mais le tirage ne la consultait
pas. Il filtre maintenant les pages actives par langues de l'identité, avec un
pont temporaire pour les anciennes identités alpha sans préférence. L'app
demande fr, en, es, ru, zh, ja, persiste le choix et le synchronise.

La porte alpha laisse place à une identité UUID anonyme créée côté serveur et
persistée localement. Impressions, réactions et langues y sont attachées.
Il n'existe pas de compte authentifié dans ce dépôt : compte et connexion sont
visibles mais non actionnables jusqu'au choix d'un fournisseur et d'un endpoint
transactionnel de fusion anonymous vers account. Cette fusion devra réassigner
préférences, impressions et réactions en une transaction.
