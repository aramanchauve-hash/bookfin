#!/usr/bin/env bash
# ==============================================================================
# BOOKFIN — SCRIPT DE RESTAURATION POSTGRESQL (ORACLE CLOUD)
# Format attendu : pg_dump custom (-Fc)
# Usage : ./restore.sh <chemin_vers_fichier.dump>
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "$#" -ne 1 ]; then
    echo "Usage : $0 <fichier_dump.dump>"
    exit 1
fi

DUMP_FILE="$1"

if [ ! -f "$DUMP_FILE" ]; then
    echo "ERREUR : Fichier introuvable : $DUMP_FILE" >&2
    exit 1
fi

# Charger la configuration
if [ -f "$SCRIPT_DIR/.env" ]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/.env"
fi

POSTGRES_USER="${POSTGRES_USER:-bookfin}"
POSTGRES_DB="${POSTGRES_DB:-bookfin}"
CONTAINER_NAME="bookfin-postgres"

echo "=== [$(date --iso-8601=seconds)] Préparation de la restauration Bookfin ==="
echo "Fichier de dump : $DUMP_FILE"
FILE_SIZE="$(stat -c%s "$DUMP_FILE" 2>/dev/null || stat -f%z "$DUMP_FILE")"
echo "Taille          : $FILE_SIZE octets (~$(( FILE_SIZE / 1024 / 1024 )) Mo)"

# 1. Vérification SHA-256 si le fichier .sha256 d'accompagnement existe
SHA_FILE="${DUMP_FILE}.sha256"
if [ -f "$SHA_FILE" ]; then
    echo "Contrôle de la somme SHA-256..."
    sha256sum -c "$SHA_FILE"
    echo "Contrôle SHA-256 : OK"
else
    echo "Note : Aucun fichier .sha256 trouvé, vérification du hash ignorée."
fi

# 2. Vérification que le conteneur cible tourne
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "ERREUR : Le conteneur $CONTAINER_NAME n'est pas actif." >&2
    exit 1
fi

# 3. Validation de la TOC du dump
echo "Inspection de la table des matières du dump (TOC)..."
docker exec -i "$CONTAINER_NAME" pg_restore --list < "$DUMP_FILE" | head -n 25
echo "... (TOC valide)"

# 4. Confirmation humaine de sécurité
echo ""
echo "ATTENTION : Vous allez restaurer ce dump dans la base '$POSTGRES_DB' ($CONTAINER_NAME)."
echo "Cette action écrasera les données actuelles de la base."
read -r -p "Confirmez-vous la restauration ? (tapez 'RESTORE_BOOKFIN_CONFIRMED') : " CONFIRMATION

if [ "$CONFIRMATION" != "RESTORE_BOOKFIN_CONFIRMED" ]; then
    echo "Restauration annulée par l'utilisateur."
    exit 1
fi

echo "Arrêt temporaire du conteneur API Bookfin pour libérer les connexions actives..."
docker compose stop api || true

echo "Exécution de pg_restore..."
# --clean --if-exists : supprime les tables existantes avant de les recréer
# --no-owner --no-privileges : prévient les conflits de rôles entre Railway et l'environnement local
docker exec -i -e PGPASSWORD="${POSTGRES_PASSWORD:-}" "$CONTAINER_NAME" \
    pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists --no-owner --no-privileges < "$DUMP_FILE" || true

echo "Vérification post-restauration de la table _sqlx_migrations..."
docker exec -e PGPASSWORD="${POSTGRES_PASSWORD:-}" "$CONTAINER_NAME" \
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT version, description, success, installed_on FROM _sqlx_migrations ORDER BY version;"

echo "Vérification des volumes de données dans les tables principales..."
docker exec -e PGPASSWORD="${POSTGRES_PASSWORD:-}" "$CONTAINER_NAME" \
    psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "
        SELECT 'works' AS table_name, count(*) AS count FROM works
        UNION ALL
        SELECT 'editions', count(*) FROM editions
        UNION ALL
        SELECT 'pages', count(*) FROM pages
        UNION ALL
        SELECT 'users', count(*) FROM users
        UNION ALL
        SELECT 'page_impressions', count(*) FROM page_impressions
        UNION ALL
        SELECT 'reactions', count(*) FROM reactions
        UNION ALL
        SELECT 'user_work_progress', count(*) FROM user_work_progress;
    "

echo "Redémarrage de l'API Bookfin..."
docker compose up -d api

echo "=== [$(date --iso-8601=seconds)] Restauration terminée. ==="
