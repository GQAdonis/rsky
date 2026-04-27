# p0-c003: rsky-pds Kubernetes Manifests

**Phase**: phase-0-foundation  
**Priority**: 3 (depends on p0-c001 Dockerfile fix, p0-c002 PostgreSQL)  
**Assigned to**: claude-code  

## Decision Summary

- **Kind**: StatefulSet (stable storage identity for MST/repo data)
- **Image**: `ghcr.io/know-me-tools/rsky-pds:IMAGE_TAG` (SHA substituted by CI)
- **Port**: 3000 (Rocket default; set via `ROCKET_PORT=3000`, `ROCKET_ADDRESS=0.0.0.0`)
- **Blob storage**: GCS via HMAC keys (S3-compatible API endpoint: `https://storage.googleapis.com`)
- **PVC**: 10Gi SSD for Rocket/repo local state, `atproto-ssd-immediate` storageclass
- **Domain**: `pds.know-me.tools`
- **TLS**: cert-manager ClusterIssuer `letsencrypt`
- **Gateway**: `gatewayClassName: eg` (Envoy Gateway, same as conduit)

## Files to Create

```
k8s/rsky-pds/
├── statefulset.yaml
├── service.yaml
├── pvc.yaml
├── configmap.yaml       # Non-secret env vars (PDS_HOSTNAME, PDS_SERVICE_DID, etc.)
├── secret.yaml          # envsubst template for keys, DB URL, GCS HMAC
├── gateway.yaml         # pds.know-me.tools, HTTPS + HTTP listeners
├── certificate.yaml     # cert-manager, ClusterIssuer: letsencrypt
├── httproute-https.yaml # All traffic → rsky-pds:3000
└── httproute-redirect.yaml  # HTTP → HTTPS 301
```

## Key Environment Variables

### ConfigMap (non-secret)
```
PDS_HOSTNAME=pds.know-me.tools
PDS_SERVICE_DID=did:web:pds.know-me.tools
PDS_SERVICE_HANDLE_DOMAINS=.know-me.tools
PDS_EMAIL_FROM_ADDRESS=noreply@know-me.tools
PDS_EMAIL_FROM_NAME=know-me.tools
PDS_CRAWLERS=https://bsky.network
ROCKET_PORT=3000
ROCKET_ADDRESS=0.0.0.0
```

### Secret (envsubst placeholders)
```
PDS_ADMIN_PASS=${PDS_ADMIN_PASS}
PDS_JWT_KEY_K256_PRIVATE_KEY_HEX=${PDS_JWT_KEY_K256_PRIVATE_KEY_HEX}
PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX=${PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX}
PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX=${PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX}
PDS_MAILGUN_API_KEY=${PDS_MAILGUN_API_KEY}
PDS_MAILGUN_DOMAIN=${PDS_MAILGUN_DOMAIN}
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky
AWS_ACCESS_KEY_ID=${GCS_HMAC_ACCESS_KEY}
AWS_SECRET_ACCESS_KEY=${GCS_HMAC_SECRET_KEY}
AWS_ENDPOINT_URL=https://storage.googleapis.com
AWS_DEFAULT_REGION=auto
AWS_BUCKET=${GCS_BUCKET_NAME}
```

## GCS Blob Storage

rsky-pds uses an S3-compatible client. GCS HMAC keys expose an S3-compatible endpoint at `storage.googleapis.com`. The env vars use `AWS_*` names — this is intentional (Rust S3 SDK convention).

## AT Protocol Well-Known

rsky-pds serves `/.well-known/atproto-did` and `/.well-known/did.json` natively — no sidecar nginx needed (unlike conduit's matrix well-known pattern).

## Liveness / Readiness Probes

```yaml
livenessProbe:
  httpGet:
    path: /xrpc/_health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 30
readinessProbe:
  httpGet:
    path: /xrpc/_health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 10
```

## DNS (Cloudflare)

After deploy, add A record in Cloudflare:
```
pds.know-me.tools  →  IP of pds-gateway LoadBalancer
```
Proxy status: **DNS only** (grey cloud) — Let's Encrypt ACME HTTP-01 challenge requires direct IP resolution; Cloudflare proxy would interfere with cert issuance.
