# p0-c002: PostgreSQL StatefulSet with pgvector

**Phase**: phase-0-foundation  
**Priority**: 2 (rsky-pds depends on this)  
**Assigned to**: claude-code  

## Decision Summary

- **Kind**: StatefulSet (plain, no Helm chart)
- **Image**: `pgvector/pgvector:pg17` — PostgreSQL 17 + pgvector extension preinstalled
- **PVC volumeBindingMode**: `Immediate` — required for GKE; `WaitForFirstConsumer` causes scheduling deadlock with StatefulSets on zonal clusters
- **Storage**: 20Gi SSD persistent disk (GKE standard-rwo or premium-rwo)
- **Namespace**: `atproto`
- **Shared by**: rsky-pds (primary) and rsky-feedgen (secondary DB, separate database name)

## Why pgvector

- rsky-wintermute (AppView) and future agent integration use vector similarity search for embeddings
- Having pgvector available from day one avoids a migration later
- `pgvector/pgvector:pg17` is the canonical maintained image

## Files to Create

```
k8s/
├── namespace.yaml
└── postgresql/
    ├── storageclass.yaml     # immediate binding SSD storageclass
    ├── statefulset.yaml      # pgvector/pgvector:pg17
    ├── service.yaml          # ClusterIP, port 5432
    ├── pvc.yaml              # 20Gi, storageclass: atproto-ssd-immediate
    └── secret.yaml           # ${POSTGRES_PASSWORD}, ${POSTGRES_USER} placeholders
```

## StorageClass (Immediate Binding)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: atproto-ssd-immediate
provisioner: pd.csi.storage.gke.io
parameters:
  type: pd-ssd
volumeBindingMode: Immediate    # NOT WaitForFirstConsumer
reclaimPolicy: Retain
allowVolumeExpansion: true
```

## StatefulSet Key Config

```yaml
image: pgvector/pgvector:pg17
env:
  - name: POSTGRES_DB
    value: rsky
  - name: POSTGRES_USER
    valueFrom:
      secretKeyRef:
        name: postgresql-secrets
        key: POSTGRES_USER
  - name: POSTGRES_PASSWORD
    valueFrom:
      secretKeyRef:
        name: postgresql-secrets
        key: POSTGRES_PASSWORD
  - name: PGDATA
    value: /var/lib/postgresql/data/pgdata
volumeClaimTemplates:
  - metadata:
      name: postgres-data
    spec:
      storageClassName: atproto-ssd-immediate
      volumeMode: Filesystem
      accessModes: [ReadWriteOnce]
      resources:
        requests:
          storage: 20Gi
```

## Secret Template

`secret.yaml` uses `${VAR}` placeholders injected by CI via `envsubst`.
