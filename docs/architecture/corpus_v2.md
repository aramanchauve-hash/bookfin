# Corpus V2 — contrat éditorial

Le corpus actuellement servi est Corpus V1 : fichiers TXT nettoyés et pages
SQL historiques. Il ne doit pas être réimporté ni voir ses identifiants changer.

Corpus V2 sépare source originale immuable (EPUB, HTML/XHTML ou Wikisource
structuré), normalisation, Bookfin JSON, puis pages Bookfin. Le JSON canonique est UTF-8,
déterministe et indépendant du fournisseur. Ses blocs sont paragraph, heading,
scene_break, blockquote et verse; ses spans peuvent porter italic, bold et
plus tard small_caps. Aucune règle occidentale de césure ou de justification
n'est encodée dans le document.

Avant l'acquisition d'une œuvre `ACCEPTED`, la curation cherche d'abord un EPUB
fiable. Lorsqu'un EPUB et un HTML/XHTML existent, elle compare paragraphes,
italiques, titres, séparateurs, vers, notes et structure des chapitres, puis
retient la source qui les préserve le mieux. EPUB est la préférence, pas un
verdict automatique : un EPUB mal structuré ou issu d'une source plus pauvre
ne l'emporte pas. EPUB n'est jamais rendu directement par l'application.

Le module scripts/corpus_v2.py fournit un adaptateur HTML conservateur, un
fallback TXT explicitement lossy, une pagination qui ne coupe ni bloc ni span,
et un hash SHA-256 canonique. TXT n'est admissible qu'en dernier recours
documenté, et jamais depuis le corpus alpha si une source riche existe. Chaque
archive source retenue est conservée intacte avec son SHA-256. Par exemple,
une balise em devient un span avec italic true. Ce n'est pas une conversion
rétroactive du corpus V1.

Migration : retenir une source riche par œuvre dans un dossier versionné
séparé, produire JSON et hashes en prévisualisation, comparer V2 à V1, puis
créer une nouvelle édition/version pour une œuvre pilote. Les anciennes pages
et impressions restent inchangées; aucun écrasement.
