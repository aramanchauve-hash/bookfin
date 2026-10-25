#!/usr/bin/env bash
# ==============================================================================
# BOOKFIN — SCRIPT DE SAUVEGARDE POSTGRESQL (ORACLE CLOUD)
# Format : pg_dump custom (-Fc)
# Fonctionnalités :
#   - Exécution via le conteneur Docker bookfin-postgres
#   - Horodatage précis (YYYYMMDD_HHMMSS)
#   - Calcul automatique de la somme SHA-256
#   - Validation d'intégrité TOC via pg_restore --list
#   - Rotation automatique des sauvegardes anciennes (par défaut 7 jours)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_DIR="${BACKUP_DIR:-$SCRIPT_DIR/backups}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"

# Charger l'environnement si présent
if [ -f "$SCRIPT_DIR/.env" ]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/.env"
fi

POSTGRES_USER="${POSTGRES_USER:-bookfin}"
POSTGRES_DB="${POSTGRES_DB:-bookfin}"
CONTAINER_NAME="bookfin-postgres"

mkdir -p "$BACKUP_DIR"

TIMESTAMP="$(date +"%Y%m%d_%H%M%S")"
BACKUP_NAME="bookfin_${POSTGRES_DB}_${TIMESTAMP}.dump"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"
SHA_PATH="${BACKUP_PATH}.sha256"

echo "=== [$(date --iso-8601=seconds)] Démarrage de la sauvegarde Bookfin ==="
echo "Conteneur : $CONTAINER_NAME"
echo "Base      : $POSTGRES_DB"
echo "Cible     : $BACKUP_PATH"

# Vérifier que le conteneur postgres est en marche
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "ERREUR : Le conteneur $CONTAINER_NAME n'est pas actif." >&2
    exit 1
fi

# 1. Extraction du dump au format binaire compressé custom (-Fc)
docker exec -e PGPASSWORD="${POSTGRES_PASSWORD:-}" "$CONTAINER_NAME" \
    pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc -b -v > "$BACKUP_PATH"

# Vérifier que le fichier n'est pas vide
FILE_SIZE="$(stat -c%s "$BACKUP_PATH" 2>/dev/null || stat -f%z "$BACKUP_PATH")"
if [ "$FILE_SIZE" -le 1024 ]; then
    echo "ERREUR : La taille de la sauvegarde ($FILE_SIZE octets) semble anormale." >&2
    exit 1
fi
echo "Taille du dump : $FILE_SIZE octets (~$(( FILE_SIZE / 1024 / 1024 )) Mo)"

# 2. Calcul du hash SHA-256
sha256sum "$BACKUP_PATH" > "$SHA_PATH"
SHA_VAL="$(cut -d' ' -f1 "$SHA_PATH")"
echo "SHA-256 : $SHA_VAL"

# 3. Validation de l'intégrité par pg_restore --list
echo "Validation de la structure du dump via pg_restore --list..."
docker exec -i "$CONTAINER_NAME" pg_restore --list < "$BACKUP_PATH" > /dev/null
echo "Validation réussie : table des matières du dump lisible et valide."

# 4. Rotation des anciennes sauvegardes
echo "Nettoyage des sauvegardes datant de plus de $RETENTION_DAYS jours..."
find "$BACKUP_DIR" -maxdepth 1 -type f \( -name "bookfin_*.dump" -o -name "bookfin_*.sha256" \) -mtime +"$RETENTION_DAYS" -exec rm -v {} \; || true

echo "=== [$(date --iso-8601=seconds)] Sauvegarde terminée avec succès. ==="
