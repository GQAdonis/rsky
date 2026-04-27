# p1-c003: rsky-relay Kubernetes Manifests

**Phase**: phase-1-relay-feedgen
**Priority**: 3 (depends on p1-c001; parallel with p1-c002)
**Assigned to**: claude-code

## Decision Summary

- **Kind**: StatefulSet (requires stable storage identity for SQLite + fjall databases)
- **Image**: `ghcr.io/know-me-tools/rsky-relay:IMAGE_TAG`
- **Port**: 9000
- **PVC**: 100Gi pd-ssd for `/data` — WORKDIR in container matches mountPath so all relative file I/O goes to persistent storage
- **StorageClass**: `atproto-ssd-immediate` (reuse from PostgreSQL)
- **Domain**: `relay.know-me.tools`
- **TLS**: cert-manager ClusterIssuer `letsencrypt`
- **Gateway**: `gatewayClassName: eg` (Envoy Gateway, same as all other services)

## Files to Create

```
k8s/rsky-relay/
├── configmap.yaml
├── secret.yaml          # envsubst template for RELAY_ADMIN_PASSWORD
├── statefulset.yaml     # WORKDIR matches PVC mountPath: /data
├── service.yaml
├── certificate.yaml
├── gateway.yaml
├── httproute-https.yaml
└── httproute-redirect.yaml
```

## Key Configuration

### ConfigMap (non-secret)
```
RELAY_ADMIN_PASSWORD is in secret; no public config needed beyond standard env
```
(rsky-relay has minimal env var surface — all config is in `src/config.rs` as constants)

### Secret (envsubst template)
```
RELAY_ADMIN_PASSWORD=${RELAY_ADMIN_PASSWORD}
```

### StatefulSet Critical Details

- `WORKDIR` in Dockerfile runtime stage is `/data`
- PVC mounts at `/data` — all relative paths (`relay.db`, `plc_directory.db`, `db/`) resolve here
- Health probe: `httpGet path: /_health port: 9000`
- Resources: cpu `500m`/`4`, memory `1Gi`/`4Gi` (fjall is memory-hungry)

### VolumeClaimTemplate
```yaml
- metadata:
    name: rsky-relay-data
  spec:
    accessModes: ["ReadWriteOnce"]
    storageClassName: atproto-ssd-immediate
    resources:
      requests:
        storage: 100Gi
```

## DNS (Cloudflare)

After deploy, add A record:
```
relay.know-me.tools  →  IP of relay-gateway LoadBalancer
```
Proxy status: **DNS only** (grey cloud) for cert issuance.
