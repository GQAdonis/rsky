# Tasks: p1-c005

## rsky-labeler
- [ ] Create `k8s/rsky-labeler/configmap.yaml` — standby mode (all ENABLE_* = false)
- [ ] Create `k8s/rsky-labeler/secret.yaml` — envsubst template for MOD_SERVICE_DID, MOD_SERVICE_EMAIL, MOD_SERVICE_PASSWORD
- [ ] Create `k8s/rsky-labeler/deployment.yaml` — 1 replica, exec liveness probe `pgrep rsky-labeler`

## rsky-jetstream-subscriber
- [ ] Create `k8s/rsky-jetstream-subscriber/configmap.yaml` — FEEDGEN_QUEUE_ENDPOINT=http://rsky-feedgen:3000
- [ ] Create `k8s/rsky-jetstream-subscriber/secret.yaml` — envsubst template for RSKY_API_KEY
- [ ] Create `k8s/rsky-jetstream-subscriber/deployment.yaml` — 1 replica, exec liveness probe `pgrep rsky-jetstream-sub`
