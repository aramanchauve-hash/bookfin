# BOOKFIN — RUNBOOK DE MIGRATION PRODUCTION (RAILWAY → ORACLE CLOUD)

Ce document décrit la procédure opérationnelle pas à pas pour exécuter la bascule de Bookfin depuis Railway vers Oracle Cloud Infrastructure (OCI) Always Free, avec garantie **zéro perte de données**, **zéro interruption imprévue** et **capacité de rollback immédiate**.

> ⚠️ **STATUT : DOCUMENT DE PRÉPARATION (PHASE 1)**  
> **NE PAS EXÉCUTER CETTE PROCÉDURE TANT QUE LA MACHINE VIRTUELLE ORACLE N'EST PAS VALIDÉE.**  
> Railway reste la source de vérité et la production active jusqu'à l'exécution contrôlée de la Phase 2.

---

## Vue d'Ensemble du Processus de Migration

```text
Étape 1 : Sauvegarde à chaud Railway (pg_dump -Fc)
   │
   ▼
Étape 2 : Vérification d'intégrité (SHA-256 & pg_restore --list)
   │
   ▼
Étape 3 : Transfert sécurisé vers la VM Oracle (SCP / SFTP)
   │
   ▼
Étape 4 : Restauration dans PostgreSQL Oracle (deploy/oracle/restore.sh)
   │
   ▼
Étape 5 : Démarrage Bookfin API sur Oracle & validation _sqlx_migrations
   │
   ▼
Étape 6 : Batterie de tests fonctionnels et d'isolation (sans impacter la prod)
   │
   ▼
Étape 7 : Configuration DNS (api.<domaine>) et TLS Caddy
   │
   ▼
Étape 8 : Mise à jour de l'application mobile (eas.json) et nouveau build
   │
   ▼
Étape 9 : Période d'observation bivalente (Railway en veille de secours)
   │
   ▼
Étape 10 : Extinction progressive et archivage Railway
```

---

## 1. Prérequis & Pré-vol

Avant d'entamer la migration finale :
1. **VM Oracle active** : Shape `VM.Standard.A1.Flex`, 1 OCPU, 6 Go RAM, Ubuntu 24.04 ARM64.
2. **Docker & Compose installés** sur la VM Oracle.
3. **Ports 80 et 443 ouverts** sur le VCN Oracle et `iptables` Ubuntu (port 5432 STRICTEMENT fermé).
4. **PostgreSQL Oracle démarré** via `docker compose up -d postgres`.
5. **PostgreSQL client local (Windows)** : `pg_dump.exe` et `pg_restore.exe` (v18.4) disponibles sous `C:\Program Files\PostgreSQL\18\bin\`.
6. **Railway CLI connecté** : `railway status` confirme l'accès au projet de production.

---

## 2. Sauvegarde Complète et Fraîche de Railway

Depuis votre poste de travail Windows (PowerShell) :

### Étape 2.1 : Établir le tunnel sécurisé vers Railway Postgres
Ouvrez un terminal dédié :
```powershell
$env:BOOKFIN_RAILWAY_TUNNEL_CONFIRMATION = "postgres-production-ssh-tunnel"
railway connect Postgres --environment production --ssh --tunnel-only --port 55432
```
*Laissez ce terminal ouvert pendant la durée du dump.*

### Étape 2.2 : Récupérer le dump au format Custom (-Fc)
Dans un second terminal PowerShell :
```powershell
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$dumpFile = "bookfin_railway_prod_${timestamp}.dump"
$dumpDir = "C:\Users\arama\Dev\Bookfin\backups"
New-Item -ItemType Directory -Force -Path $dumpDir | Out-Null
$dumpPath = Join-Path $dumpDir $dumpFile

Write-Host "Démarrage du dump Railway vers $dumpPath..."
& "C:\Program Files\PostgreSQL\18\bin\pg_dump.exe" `
    -h localhost -p 55432 -U postgres -d railway `
    -Fc -b -v -f $dumpPath

# Vérifier la taille
$size = (Get-Item $dumpPath).Length
Write-Host "Dump terminé. Taille : $size octets (~$([math]::Round($size / 1MB, 2)) Mo)"
```

### Étape 2.3 : Calcul de la somme SHA-256
```powershell
$hash = (Get-FileHash -Path $dumpPath -Algorithm SHA256).Hash
$hash | Out-File -Encoding ascii "$dumpPath.sha256"
Write-Host "SHA-256 : $hash"
```

### Étape 2.4 : Validation de la Table des Matières (TOC)
```powershell
& "C:\Program Files\PostgreSQL\18\bin\pg_restore.exe" --list $dumpPath | Select-Object -First 30
```
*Le dump doit comporter toutes les tables (`works`, `editions`, `pages`, `users`, `page_impressions`, `reactions`, `user_work_progress`, `reading_events`, `_sqlx_migrations`).*

---

## 3. Transfert vers la VM Oracle

Transférez le fichier de dump et son hash vers la VM Oracle :
```powershell
scp -i ~/.ssh/id_ed25519 $dumpPath "$dumpPath.sha256" ubuntu@<IP_ORACLE>:/home/ubuntu/
```

---

## 4. Restauration dans PostgreSQL Oracle

Connectez-vous en SSH à la VM Oracle :
```powershell
ssh -i ~/.ssh/id_ed25519 ubuntu@<IP_ORACLE>
```

Exécutez la restauration contrôlée :
```bash
cd /home/ubuntu/bookfin/deploy/oracle
./restore.sh "/home/ubuntu/$dumpFile"
```

Le script :
- Valide l'intégrité SHA-256 ;
- Affiche la TOC du dump ;
- Demande la confirmation `RESTORE_BOOKFIN_CONFIRMED` ;
- Restaure le schéma et les données (`--clean --if-exists --no-owner --no-privileges`) ;
- Affiche l'historique `_sqlx_migrations` (12 migrations attendues, dont la 0012) ;
- Affiche les totaux de lignes des tables critiques.

---

## 5. Démarrage et Validation de l'API Bookfin sur Oracle

Sur la VM Oracle :
```bash
cd /home/ubuntu/bookfin/deploy/oracle
docker compose up -d api
docker compose logs -f api
```

### Vérifications au démarrage :
1. Les logs affichent : `Connexion à PostgreSQL configurée.`
2. Les logs affichent : `Exécution des migrations SQLx...`
3. Les logs affichent : `Migrations terminées avec succès.` (aucun écart de checksum sur les migrations 0001 à 0012).
4. Le listener démarre sur `0.0.0.0:3000`.
5. Le conteneur passe en statut `healthy` (`docker compose ps`).

---

## 6. Protocole de Tests Fonctionnels Complets

Exécutez ce protocole contre l'instance Oracle **avant** toute modification DNS ou mobile, à l'aide d'identités temporaires jetables.

```bash
# Variable de base (test local sur la VM ou via IP publique)
export API_URL="http://localhost:80"
```

### Checklist Fonctionnelle :

- [ ] **1. Healthcheck** :
  ```bash
  curl -s -i "$API_URL/health"
  ```
  *Attendu* : HTTP 200 OK, corps `OK`.

- [ ] **2. Création d'identités anonymes** (2 utilisateurs) :
  ```bash
  U1_RESP=$(curl -s -X POST "$API_URL/api/v1/identities/anonymous")
  U2_RESP=$(curl -s -X POST "$API_URL/api/v1/identities/anonymous")
  U1_ID=$(echo "$U1_RESP" | jq -r '.user.id')
  U2_ID=$(echo "$U2_RESP" | jq -r '.user.id')
  echo "User 1: $U1_ID | User 2: $U2_ID"
  ```
  *Attendu* : HTTP 201 Created avec UUID valides.

- [ ] **3. Préférences de langue** :
  ```bash
  curl -s -i -X PUT "$API_URL/api/v1/users/$U1_ID/languages" \
    -H "Content-Type: application/json" \
    -d '{"languages": ["en", "fr"]}'
  ```
  *Attendu* : HTTP 200 OK.

- [ ] **4. Feed de découverte (Discovery Feed)** :
  ```bash
  PAGE1_RESP=$(curl -s "$API_URL/api/v1/feed/next?user_id=$U1_ID")
  P1_ID=$(echo "$PAGE1_RESP" | jq -r '.page.id')
  W1_ID=$(echo "$PAGE1_RESP" | jq -r '.work.id')
  echo "Page ID: $P1_ID | Work ID: $W1_ID"
  ```
  *Attendu* : Page servie, `reading_context = 'discovery'`, progression utilisateur intacte.

- [ ] **5. Réaction** :
  ```bash
  curl -s -i -X POST "$API_URL/api/v1/pages/$P1_ID/reactions" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$U1_ID\", \"reaction_type\": \"like\"}"
  ```
  *Attendu* : HTTP 200/201.

- [ ] **6. Démarrage de lecture suivie (Start Work)** :
  ```bash
  curl -s -i -X POST "$API_URL/api/v1/works/$W1_ID/start" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$U1_ID\"}"
  ```
  *Attendu* : HTTP 200 OK, première page de l'œuvre servie, `reading_context = 'sequential'`, `user_work_progress` créé à la page 1.

- [ ] **7. Détection de doublon (Duplicate Start)** :
  ```bash
  curl -s -i -X POST "$API_URL/api/v1/works/$W1_ID/start" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$U1_ID\"}"
  ```
  *Attendu* : **HTTP 409 Conflict** (`work_already_started`).

- [ ] **8. Continuation séquentielle (Continue Sequential)** :
  ```bash
  curl -s -i -X POST "$API_URL/api/v1/pages/$P1_ID/continue" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$U1_ID\"}"
  ```
  *Attendu* : Page suivante retournée, `current_page_sequence_number` incrémenté.

- [ ] **9. Reprise de lecture (Resume Work)** :
  ```bash
  curl -s -i -X POST "$API_URL/api/v1/works/$W1_ID/resume" \
    -H "Content-Type: application/json" \
    -d "{\"user_id\": \"$U1_ID\"}"
  ```
  *Attendu* : Reprise exacte à la page courante sauvegardée.

- [ ] **10. Bibliothèque personnelle (Library)** :
  ```bash
  curl -s "$API_URL/api/v1/me/library?user_id=$U1_ID" | jq .
  ```
  *Attendu* : L'œuvre commencée apparaît avec la bonne progression.

- [ ] **11. Invariant d'isolation entre découverte et séquentiel** :
  Un appel aléatoire à `/api/v1/feed/next` ou `/continue` depuis une autre page de découverte ne doit en aucun cas altérer la progression dans `user_work_progress` de l'œuvre suivie.

- [ ] **12. Isolation stricte entre utilisateurs** :
  ```bash
  curl -s "$API_URL/api/v1/me/library?user_id=$U2_ID" | jq .
  ```
  *Attendu* : La bibliothèque de User 2 est strictement vide (`[]`). Une tentative de resume par User 2 retourne HTTP 404.

- [ ] **13. Nettoyage des identités de test** :
  ```bash
  docker exec -i -e PGPASSWORD="$POSTGRES_PASSWORD" bookfin-postgres \
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" \
    -c "DELETE FROM users WHERE id IN ('$U1_ID', '$U2_ID');"
  ```

---

## 7. Configuration DNS & Bascule HTTPS

1. Rendez-vous sur votre registraire DNS.
2. Créez l'enregistrement A :
   `api.<votre-domaine>` → `<IP_PUBLIQUE_ORACLE>` (TTL court, ex: 300s).
3. Mettez à jour `/home/ubuntu/bookfin/deploy/oracle/.env` sur la VM :
   ```env
   DOMAIN=api.<votre-domaine>
   ACME_EMAIL=votre-email@domaine.com
   ```
4. Redémarrez Caddy :
   ```bash
   docker compose up -d caddy
   docker compose logs -f caddy
   ```
   *Caddy émet et configure automatiquement le certificat TLS Let's Encrypt / ZeroSSL.*
5. Testez depuis votre navigateur ou machine locale :
   ```powershell
   curl -i https://api.<votre-domaine>/health
   ```
   *Attendu : HTTP/2 200 OK avec certificat valide.*

---

## 8. Bascule de l'Application Mobile

1. Dans `mobile/eas.json`, mettez à jour `EXPO_PUBLIC_API_BASE_URL` :
   ```json
   "env": {
     "EXPO_PUBLIC_API_BASE_URL": "https://api.<votre-domaine>"
   }
   ```
2. Générez le nouveau build preview / production avec EAS :
   ```bash
   eas build --platform android --profile preview
   ```
3. Installez l'APK sur l'appareil de test et vérifiez l'ensemble des parcours utilisateurs en conditions réelles.

---

## 9. Procédure de Rollback (En cas d'anomalie imprévue)

Pendant toute la phase de migration et la période d'observation post-migration :
- **Railway reste intact et allumé**.
- En cas de problème bloquant sur Oracle :
  1. Remettre l'URL `https://bookfin-api-production.up.railway.app` dans `mobile/eas.json`.
  2. Si le DNS avait été modifié, rediriger `api.<domaine>` vers Railway ou restaurer l'ancienne configuration.
  3. Aucune perte de données car le dump initial Railway reste archivé et intact.

---

## 10. Clôture & Décommissionnement Railway (Post-observation)

Après 7 à 14 jours d'observation stable sur Oracle sans aucune régression :
1. Effectuer une dernière sauvegarde de sécurité de la base Railway.
2. Suspendre puis supprimer le projet Railway.
