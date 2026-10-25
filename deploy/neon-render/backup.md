# Procédure de Sauvegarde et Restauration Neon PostgreSQL

Ce document décrit les opérations manuelles de sauvegarde et de restauration de la base de données Neon PostgreSQL pour Bookfin.

---

## 1. Sauvegarde Complète (pg_dump format Custom)

Depuis une machine disposant des outils PostgreSQL (v18.x) :

### Commande :
```bash
# Remplacer par la chaîne de connexion directe (non-pooler) de Neon
export NEON_DIRECT_URL="postgres://<user>:<password>@ep-bookfin.<region>.aws.neon.tech/neondb?sslmode=require"

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
DUMP_FILE="bookfin-neon-${TIMESTAMP}.dump"

pg_dump "$NEON_DIRECT_URL" -Fc -b -v -f "$DUMP_FILE"
```

### Vérification de l'intégrité :
```bash
# Calcul de l'empreinte SHA-256
sha256sum "$DUMP_FILE" > "${DUMP_FILE}.sha256"

# Validation de la Table des Matières (TOC)
pg_restore --list "$DUMP_FILE" | head -n 30
```

---

## 2. Restauration vers Neon

> ⚠️ **Attention** : À exécuter uniquement sur une base Neon vide ou préparée.

```bash
export NEON_DIRECT_URL="postgres://<user>:<password>@ep-bookfin.<region>.aws.neon.tech/neondb?sslmode=require"

# Utiliser --no-owner et --no-privileges pour s'adapter au rôle Neon
pg_restore --no-owner --no-privileges -d "$NEON_DIRECT_URL" -v "$DUMP_FILE"
```

### Validation post-restauration :
```bash
psql "$NEON_DIRECT_URL" -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"
psql "$NEON_DIRECT_URL" -c "SELECT count(*) FROM works; SELECT count(*) FROM pages; SELECT count(*) FROM users;"
```
