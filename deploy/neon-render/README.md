# Bookfin — Déploiement Neon + Render Free

Ce dossier contient la configuration pour héberger l'API Bookfin et sa base de données sur une architecture 100 % gratuite et pérenne :

- **Render Free Web Service** : API Rust / Axum conteneurisée via Docker.
- **Neon Free PostgreSQL** : Base de données PostgreSQL managée, serverless, avec TLS obligatoire.

---

## 1. Architecture

```text
Application mobile Bookfin
        │
        │ HTTPS
        ▼
Render Free Web Service (bookfin-api)
        │
        │ PostgreSQL TLS (rustls)
        ▼
Neon Free PostgreSQL
```

---

## 2. Principes & Contraintes

1. **Aucun Render Postgres** : Render Postgres expire au bout de 30 jours sur le plan gratuit. La persistance est assurée exclusivement par Neon.
2. **Cold Start Render Free** : Le Web Service se met en veille après 15 minutes d'inactivité. Le premier appel de réveil prend 30 à 50 secondes. C'est un comportement normal et accepté pour l'alpha.
3. **Deux URLs Neon** :
   - **Pooled connection** (avec `-pooler`) : Pour l'exécution du Web Service sur Render (`DATABASE_URL`), afin d'optimiser le pool de connexions SQLx.
   - **Direct connection** (sans `-pooler`) : Pour les opérations de migration (`pg_restore`), dumps ou migrations lourdes.
4. **Zéro Secret dans Git** : La chaîne `DATABASE_URL` contenant les identifiants est renseignée uniquement dans les variables d'environnement secrètes de Render.

---

## 3. Mise en Service

1. **Créer le projet Neon** :
   - Région : Europe (`eu-central-1` Francfort recommandée).
   - PostgreSQL : v18 ou v17 (version stable).
   - Récupérer les deux chaînes de connexion (directe et pooled).
2. **Restaurer les données** :
   - Suivre les instructions dans `backup.md` pour injecter le dump initial vers l'URL directe Neon.
3. **Créer le Web Service Render** :
   - Choisir *Web Service* > Connecter le dépôt Git (`infra/neon-render-migration`).
   - Runtime : *Docker*.
   - Plan : *Free*.
   - Variable secrète : `DATABASE_URL` = chaîne pooled Neon.
   - Health check path : `/health`.
