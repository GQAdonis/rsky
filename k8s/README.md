# ATProto Stack — Kubernetes Manifests

Deployed to GKE via ArgoCD GitOps. CI builds images, commits SHA tags back to this directory, and ArgoCD auto-syncs.

## Namespace

`atproto`

## Services

| Service | Kind | Domain |
|---------|------|--------|
| rsky-pds | StatefulSet | pds.know-me.tools |
| rsky-relay | StatefulSet | relay.know-me.tools |
| rsky-feedgen | Deployment | feed.know-me.tools |
| rsky-labeler | Deployment | internal worker (no public domain) |
| rsky-jetstream-subscriber | Deployment | internal worker (no public domain) |
| postgresql | StatefulSet | internal (ClusterIP) |
| web-client (Ouranos) | Deployment | social.know-me.tools |

## Directory Structure

```
k8s/
├── namespace.yaml
├── argocd/
│   └── application.yaml                 # ArgoCD Application (apply manually once)
├── postgresql/
│   ├── storageclass.yaml                # atproto-ssd-immediate (volumeBindingMode: Immediate)
│   ├── initdb-configmap.yaml            # Creates rsky + rsky_feedgen databases on first start
│   ├── secret.yaml                      # envsubst template
│   ├── statefulset.yaml
│   └── service.yaml
├── rsky-pds/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template
│   ├── statefulset.yaml
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-relay/
│   ├── secret.yaml                      # envsubst template
│   ├── statefulset.yaml                 # WORKDIR /data — PVC mounts here
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-feedgen/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-labeler/
│   ├── configmap.yaml                   # ENABLE_* = false (standby mode)
│   ├── secret.yaml                      # envsubst template
│   └── deployment.yaml
├── rsky-jetstream-subscriber/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template
│   └── deployment.yaml
└── web-client/
    ├── configmap.yaml
    ├── deployment.yaml
    ├── service.yaml
    ├── certificate.yaml
    ├── gateway.yaml
    ├── httproute-https.yaml
    └── httproute-redirect.yaml
```

## Required GitHub Secrets

Set these in the repository settings before the first deploy:

### Infrastructure

| Secret | Description |
|--------|-------------|
| `GKE_SA_KEY` | GCP service account JSON (same as conduit project) |
| `GKE_PROJECT_ID` | GCP project ID |
| `GHCR_PAT` | GitHub PAT with `packages:write` + `contents:write` |

### PostgreSQL

| Secret | Description |
|--------|-------------|
| `POSTGRES_USER` | PostgreSQL username (e.g. `rsky`) |
| `POSTGRES_PASSWORD` | Strong generated password |

### rsky-pds

| Secret | Description |
|--------|-------------|
| `PDS_ADMIN_PASS` | rsky-pds admin password |
| `PDS_JWT_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex for JWT signing |
| `PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex for PLC rotation |
| `PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex for repo signing |
| `RESEND_API_KEY` | Resend API key for transactional email |
| `GCS_HMAC_ACCESS_KEY` | GCS HMAC access key (S3-compatible blob storage) |
| `GCS_HMAC_SECRET_KEY` | GCS HMAC secret key |
| `GCS_BUCKET_NAME` | GCS bucket name for PDS blobs |

### rsky-relay

| Secret | Description |
|--------|-------------|
| `RELAY_ADMIN_PASSWORD` | Admin password for relay management endpoints |

### rsky-feedgen + rsky-jetstream-subscriber

| Secret | Description |
|--------|-------------|
| `RSKY_API_KEY` | Shared API key authenticating jetstream→feedgen queue writes |

### rsky-labeler (standby — populate with placeholders initially)

| Secret | Description |
|--------|-------------|
| `MOD_SERVICE_DID` | Labeler service account DID (e.g. `did:plc:xxxxx`) |
| `MOD_SERVICE_EMAIL` | Labeler service account email |
| `MOD_SERVICE_PASSWORD` | Labeler service account password |

## Generating PDS Keys

```bash
# Each key is a secp256k1 private key — generate 3 independent ones
openssl ecparam -name secp256k1 -genkey -noout | \
  openssl ec -text -noout 2>/dev/null | \
  grep priv -A 3 | tail -3 | tr -d ' :\n'
```

## First Deploy

1. Apply the ArgoCD Application manually once:
   ```bash
   kubectl apply -f k8s/argocd/application.yaml
   ```
2. Set all GitHub secrets listed above. For labeler secrets, use placeholder values (e.g. `placeholder`).
3. Push to `main` — CI builds images and commits SHA tags; ArgoCD syncs.
4. Set Cloudflare DNS A records (DNS only / grey cloud initially for cert issuance):
   ```
   pds.know-me.tools     →  kubectl get gateway pds-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
   relay.know-me.tools   →  kubectl get gateway relay-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
   feed.know-me.tools    →  kubectl get gateway feedgen-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
   social.know-me.tools  →  kubectl get gateway web-client-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
   ```

## GCS HMAC Keys

Create via GCP Console → Cloud Storage → Settings → Interoperability → Create key for service account.

The `AWS_*` env var names are intentional — rsky-pds uses an S3-compatible Rust SDK that reads standard AWS env vars. The endpoint is set to `https://storage.googleapis.com`.

## Activating the Labeler

rsky-labeler is deployed in standby mode (all actions disabled). To activate:

1. Create a Bluesky account to serve as the labeler service account.
2. Register a labeler service record (`app.bsky.labeler.service`) on your PDS for that account.
3. Get the account's DID from: `https://pds.know-me.tools/xrpc/com.atproto.identity.resolveHandle?handle=<your-handle>`
4. Set the GitHub secrets: `MOD_SERVICE_DID`, `MOD_SERVICE_EMAIL`, `MOD_SERVICE_PASSWORD` with real values.
5. Update `k8s/rsky-labeler/configmap.yaml`: set `ENABLE_CREATE_REPORT`, `ENABLE_CREATE_LABEL`, `ENABLE_CREATE_TAG` to `"true"`.
6. Commit and push — ArgoCD deploys the updated ConfigMap and restarts the pod.

## rsky-relay Storage Notes

The relay StatefulSet uses WORKDIR `/data` in the container, which is also the PVC mount path. All rsky-relay storage files use relative paths from CWD:
- `relay.db` — SQLite for host bans and validator state
- `plc_directory.db` — SQLite for DID resolution cache
- `db/` — fjall LSM database for firehose event storage (up to 320 GiB)

The 100Gi PVC is sized to handle initial operation. Expand via:
```bash
kubectl patch pvc rsky-relay-data-rsky-relay-0 -n atproto \
  -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'
```
