# Rapport & Runbook — Migration Bookfin (Railway → Neon + Render Free)

Ce rapport documente la stratégie, l'architecture et les opérations de migration de **Bookfin** depuis l'infrastructure Railway vers le couple **Neon Free (PostgreSQL)** et **Render Free (Web Service Docker)**.

---

## 1. Architecture Cible

```text
Application mobile Bookfin (Expo / React Native)
               │
               │ HTTPS (port 443)
               ▼
   Render Free Web Service (bookfin-api)
   - Conteneur Docker Rust 1.98.1 / Axum
   - Écoute sur 0.0.0.0:$PORT
   - Healthcheck natif /health
   - Filesystem éphémère (aucun état local)
               │
               │ PostgreSQL TLS (rustls / port 5432)
               ▼
   Neon Free Serverless PostgreSQL
   - Instance cloud isolée (Région Europe / eu-central-1)
   - Base de données persistante (quota gratuit : 0,5 Go de données)
   - Prise en charge TLS obligatoire (sslmode=require)
   - Mode connection pooling PgBouncer pour Render
```

---

## 2. Quotas & Limites des Plans Gratuits

| Fournisseur | Ressource | Quota Gratuit | Usage Estimé Bookfin | Statut |
|---|---|---|---|---|
| **Neon** | Stockage DB | 0,5 Go (512 Mo) | ~25 à 40 Mo | ✅ Conforme |
| **Neon** | Compute | 100 CU-heures / mois | Veille automatique | ✅ Conforme |
| **Render** | Instance Type | Free Web Service (0.1 CPU, 512 Mo RAM) | Axum binaire léger (< 50 Mo RAM) | ✅ Conforme |
| **Render** | Temps d'exécution | 750 heures gratuites / mois | 1 Web Service en veille | ✅ Conforme |
| **Render** | Bande passante | 100 Go / mois sortant | < 5 Go / mois | ✅ Conforme |

---

## 3. Gestion Spécifique du Cold Start Render

- **Comportement** : Render Free met le conteneur en veille après 15 minutes sans requête entrante.
- **Délai de réveil** : Le démarrage d'un conteneur Rust en veille prend entre **25 et 50 secondes**. Les requêtes ultérieures répondent en moins de 50 ms.
- **Règle acceptée** : Aucun cron de keep-alive artificiel n'est mis en place. L'application mobile doit tolérer un timeout initial de 45 à 60 secondes au lancement pour le premier appel utilisateur.

---

## 4. Procédure Opérationnelle de Migration

### Étape 1 : Sauvegarde de la Base Source (Railway)
```bash
# Vérifier l'accès et déclencher le dump binaire custom
pg_dump "$RAILWAY_DATABASE_URL" -Fc -b -v -f "bookfin-railway-pre-neon-$(date +%Y%m%d-%H%M%S).dump"

# Calcul du hash et validation structurelle
sha256sum "$DUMP_FILE" > "${DUMP_FILE}.sha256"
pg_restore --list "$DUMP_FILE"
```

### Étape 2 : Restauration vers Neon
```bash
# Injection vers l'URL directe Neon
pg_restore --no-owner --no-privileges -d "$NEON_DIRECT_DATABASE_URL" -v "$DUMP_FILE"

# Contrôle SQLx
psql "$NEON_DIRECT_DATABASE_URL" -c "SELECT version, success FROM _sqlx_migrations ORDER BY version;"
```

### Étape 3 : Déploiement Render
1. Connecter le dépôt Git à Render.
2. Définir le Web Service avec `Dockerfile` et plan *Free*.
3. Renseigner la variable secrète `DATABASE_URL` avec la chaîne **pooled** Neon.
4. Lancer le build et surveiller `/health`.

---

## 5. Stratégie de Cutover & Synchronisation Finale

Pour éviter toute perte d'écritures intervenues pendant la phase de préparation :
1. Comparer `MAX(created_at)` sur `reading_events` et `page_impressions` sur Railway.
2. Réaliser le dump final frais juste avant la mise à jour mobile.
3. Restaurer le dump final sur Neon.
4. Mettre à jour `EXPO_PUBLIC_API_BASE_URL` dans `mobile/eas.json` :
   `https://<service-render>.onrender.com`
5. Distribuer le build mobile mis à jour.

---

## 6. Procédure de Rollback

En cas d'anomalie bloquante sur Render ou Neon :
1. Railway reste **100 % intact et non supprimé**.
2. Remettre l'URL `https://bookfin-api-production.up.railway.app` dans `mobile/eas.json`.
3. Rééditer le build mobile en cas d'urgence.
4. Les sauvegardes de sécurité locales restent archivées sous `~/Bookfin-Railway-Backups/`.
