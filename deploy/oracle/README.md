# Bookfin — Déploiement Oracle Cloud Infrastructure (OCI) Always Free

Ce dossier contient l'ensemble de la configuration nécessaire pour déployer et opérer l'infrastructure **Bookfin** sur une instance **Oracle Cloud Always Free ARM64** sans aucun coût d'hébergement.

---

## 1. Architecture Cible

```text
       Internet
          │
          │ HTTPS (443) / HTTP (80)
          ▼
   ┌──────────────┐
   │    Caddy     │  (Reverse proxy, terminaison TLS automatique Let's Encrypt / ZeroSSL)
   └──────┬───────┘
          │ HTTP (interne : 3000)
          ▼
   ┌──────────────┐
   │ Bookfin API  │  (Binaire Rust Axum compilé nativement linux/arm64)
   └──────┬───────┘
          │ TCP (interne : 5432, réseau Docker isolé)
          ▼
   ┌──────────────┐
   │  PostgreSQL  │  (PostgreSQL 18 Alpine ARM64, volume persistant)
   └──────────────┘
```

### Principes de sécurité
1. **Zéro exposition base de données** : Le port PostgreSQL 5432 n'est **jamais** publié sur l'hôte ni exposé à Internet. Il est confiné dans un réseau Docker interne (`internal_net`).
2. **Reverse-proxy Caddy étanche** : Seuls les ports 80 (HTTP) et 443 (HTTPS TCP/UDP) sont exposés sur la machine hôte.
3. **SSH restreint** : Authentification SSH par clé privée uniquement (`id_ed25519`). Aucun mot de passe root.
4. **Zéro secret commité** : Les identifiants sont définis exclusivement dans un fichier `.env` non suivi par Git.

---

## 2. Dimensionnement & Éligibilité OCI Always Free

Le programme **Oracle Cloud Always Free** offre généreusement jusqu'à :
- **4 OCPU** et **24 Go de RAM** sur les formes Ampere A1 ARM64 (`VM.Standard.A1.Flex`) ;
- **200 Go** d'espace disque Boot Volume cumulé ;
- **1 IPv4 publique** gratuite.

### Dimensionnement recommandé pour Bookfin :
- **Shape** : `VM.Standard.A1.Flex`
- **Architecture** : Ampere A1 (`aarch64` / `arm64`)
- **OS** : Ubuntu 24.04 LTS ou Ubuntu 22.04 LTS (Canonical Ubuntu, aarch64)
- **OCPU** : `1 OCPU`
- **RAM** : `6 Go` (largement suffisant pour Axum + Postgres + Caddy, laissant 3 OCPU et 18 Go de réserve Free Tier)
- **Boot Volume** : `50 Go` à `100 Go` (sélectionner "Always Free Eligible")
- **Coût estimé affiché par OCI** : **0,00 € / mois (Always Free Eligible)**

> ⚠️ **Règle absolue** : Si la console OCI indique un montant supérieur à 0,00 € ou mentionne un manque de capacité sans badge "Always Free", **ne validez pas**.

---

## 3. Configuration Réseau OCI (Security List / Firewall)

Dans la console OCI (`Networking` > `Virtual Cloud Networks` > `VCN` > `Security Lists` > `Default Security List`) :

### Règles d'entrée (Ingress Rules) à configurer :
| Protocole | Port Source | Port Destination | Description | Exposition |
|-----------|-------------|------------------|-------------|------------|
| TCP       | Any         | `22`             | SSH         | Restreint (ou votre IP si IP fixe) |
| TCP       | Any         | `80`             | HTTP Caddy  | Public (0.0.0.0/0) |
| TCP       | Any         | `443`            | HTTPS Caddy | Public (0.0.0.0/0) |
| UDP       | Any         | `443`            | HTTP/3 QUIC | Public (0.0.0.0/0) |

> ⛔ **NE JAMAIS OUVRIR LE PORT 5432 DANS LA SECURITY LIST.**

### Firewall Ubuntu (sur la VM) :
Par défaut sur Ubuntu OCI, `iptables` filtre le trafic entrant. Exécutez :
```bash
# Autoriser HTTP et HTTPS
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 80 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 443 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p udp --dport 443 -j ACCEPT
sudo netfilter-persistent save
```

---

## 4. Préparation de la Machine Virtuelle Ubuntu

Depuis votre poste de travail Windows (PowerShell) :

### Connexion SSH initiale :
```powershell
ssh -i ~/.ssh/id_ed25519 ubuntu@<IP_PUBLIQUE_ORACLE>
```

### Installation de Docker et Docker Compose :
```bash
# 1. Mise à jour système
sudo apt-get update && sudo apt-get upgrade -y

# 2. Installation des prérequis Docker
sudo apt-get install -y ca-certificates curl gnupg lsb-release

# 3. Clé GPG officielle Docker
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# 4. Dépôt Docker
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 5. Installation Docker Engine et Compose
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# 6. Autoriser l'utilisateur ubuntu à exécuter docker
sudo usermod -aG docker ubuntu
newgrp docker
```

---

## 5. Déploiement de Bookfin

### Cloner le dépôt et initialiser l'environnement :
```bash
git clone https://github.com/votre-compte/bookfin.git
cd bookfin/deploy/oracle

# Copier le fichier d'exemple et générer un mot de passe fort
cp .env.example .env
chmod 600 .env
nano .env
```

Dans `.env` :
- Fixez un `POSTGRES_PASSWORD` long et aléatoire (ex: `openssl rand -hex 32`).
- Laissez `DOMAIN=:80` au départ (pour tester l'IP sans domaine).
- Dès que votre nom de domaine pointe vers l'IP Oracle, changez pour `DOMAIN=api.votre-domaine.com`.

### Lancer la stack :
```bash
docker compose up -d --build
```

---

## 6. Observabilité & Commandes Utiles

### État des conteneurs :
```bash
docker compose ps
```

### Logs en direct :
```bash
# Logs de l'API Axum
docker compose logs -f api

# Logs Caddy (accès et certificats)
docker compose logs -f caddy

# Logs PostgreSQL
docker compose logs -f postgres
```

### Test de santé (/health) :
```bash
# Depuis la VM en local
curl -i http://localhost:80/health

# Depuis Internet (après configuration du firewall)
curl -i http://<IP_PUBLIQUE_ORACLE>/health
```

### Monitoring ressources :
```bash
df -h          # Espace disque
free -h        # Mémoire RAM disponible
docker stats   # Consommation CPU / RAM en temps réel par conteneur
```

---

## 7. Sauvegardes & Restauration

### Sauvegarde manuelle :
```bash
./backup.sh
```
Génère un fichier horodaté `backups/bookfin_bookfin_YYYYMMDD_HHMMSS.dump` avec contrôle SHA-256.

### Automatisation quotidienne via Cron :
```bash
crontab -e
```
Ajouter la ligne suivante pour exécuter un backup chaque nuit à 03:00 UTC :
```cron
0 3 * * * /home/ubuntu/bookfin/deploy/oracle/backup.sh >> /home/ubuntu/backup.log 2>&1
```

### Restauration :
```bash
./restore.sh backups/bookfin_bookfin_YYYYMMDD_HHMMSS.dump
```
Le script demande une confirmation explicite avant d'écraser la base et affiche les compteurs de vérification post-restauration (`_sqlx_migrations`, `works`, `pages`, etc.).
