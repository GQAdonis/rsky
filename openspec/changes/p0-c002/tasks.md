# Tasks: p0-c002

- [ ] Create `k8s/namespace.yaml` (atproto namespace)
- [ ] Create `k8s/postgresql/storageclass.yaml` (atproto-ssd-immediate, pd-ssd, volumeBindingMode: Immediate)
- [ ] Create `k8s/postgresql/secret.yaml` (envsubst template: ${POSTGRES_USER}, ${POSTGRES_PASSWORD})
- [ ] Create `k8s/postgresql/pvc.yaml` (20Gi, atproto-ssd-immediate, Retain)
- [ ] Create `k8s/postgresql/statefulset.yaml` (pgvector/pgvector:pg17, PGDATA set, liveness/readiness probes)
- [ ] Create `k8s/postgresql/service.yaml` (ClusterIP, port 5432, named postgres)
- [ ] Verify StatefulSet spec passes `kubectl dry-run` syntax check
