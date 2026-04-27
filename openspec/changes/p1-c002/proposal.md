# p1-c002: PostgreSQL initdb ConfigMap for rsky_feedgen Database

**Phase**: phase-1-relay-feedgen
**Priority**: 2 (parallel with p1-c003; required before p1-c004)
**Assigned to**: claude-code

## Problem

The existing `k8s/postgresql/statefulset.yaml` creates only the database named by `POSTGRES_DB` (which will be `rsky` for rsky-pds). rsky-feedgen requires a separate `rsky_feedgen` database with its own schema.

PostgreSQL's `pgvector/pgvector:pg17` image automatically runs all `.sql` scripts found in `/docker-entrypoint-initdb.d/` on first startup. A Kubernetes ConfigMap mounted at that path is the standard way to provision multiple databases.

## Solution

Create a ConfigMap containing an init SQL script, and mount it into the PostgreSQL StatefulSet.

### initdb ConfigMap (`k8s/postgresql/initdb-configmap.yaml`)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: postgresql-initdb
  namespace: atproto
data:
  init.sql: |
    -- Create databases for each ATProto service
    SELECT 'CREATE DATABASE rsky'
      WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'rsky')\gexec

    SELECT 'CREATE DATABASE rsky_feedgen'
      WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'rsky_feedgen')\gexec

    -- Enable pgvector on both databases
    \c rsky
    CREATE EXTENSION IF NOT EXISTS vector;

    \c rsky_feedgen
    CREATE EXTENSION IF NOT EXISTS vector;
```

### StatefulSet update (`k8s/postgresql/statefulset.yaml`)

Add volume and volumeMount to mount the ConfigMap at `/docker-entrypoint-initdb.d/`:

```yaml
volumes:
  - name: initdb-scripts
    configMap:
      name: postgresql-initdb

containers:
  - name: postgresql
    volumeMounts:
      - name: initdb-scripts
        mountPath: /docker-entrypoint-initdb.d
```

**Important**: initdb scripts only run on first pod startup (when the data directory is empty). If the PostgreSQL PVC already exists with data, the scripts will NOT re-run. This is correct behavior — migrations handle schema changes after initial setup.

## Files to Create/Modify

```
k8s/postgresql/initdb-configmap.yaml    — new file
k8s/postgresql/statefulset.yaml         — add initdb volume + mount
```
