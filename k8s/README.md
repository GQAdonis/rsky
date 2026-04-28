# ATProto Stack — Kubernetes Manifests

Deployed to GKE via GitHub Actions (`deploy.yml`). On push to `main`, images
are built in parallel, then the deploy job applies manifests and rolls out
workloads using `kubectl` directly — no ArgoCD.

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
├── postgresql/
│   ├── storageclass.yaml                # atproto-ssd-immediate (volumeBindingMode: Immediate)
│   ├── initdb-configmap.yaml            # Creates rsky + rsky_feedgen databases on first start
│   ├── secret.yaml                      # envsubst template (filled by CI)
│   ├── statefulset.yaml
│   └── service.yaml
├── rsky-pds/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template (filled by CI)
│   ├── statefulset.yaml                 # IMAGE_TAG replaced by CI sed
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-relay/
│   ├── secret.yaml                      # envsubst template (filled by CI)
│   ├── statefulset.yaml                 # WORKDIR /data — PVC mounts here
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-feedgen/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template (filled by CI)
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── certificate.yaml
│   ├── gateway.yaml
│   ├── httproute-https.yaml
│   └── httproute-redirect.yaml
├── rsky-labeler/
│   ├── configmap.yaml                   # ENABLE_* = false (standby mode)
│   ├── secret.yaml                      # envsubst template (filled by CI)
│   └── deployment.yaml
├── rsky-jetstream-subscriber/
│   ├── configmap.yaml
│   ├── secret.yaml                      # envsubst template (filled by CI)
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

All secrets are already seeded. Reference only:

### Infrastructure

| Secret | Description |
|--------|-------------|
| `GKE_SA_KEY` | GCP service account JSON |
| `GKE_PROJECT_ID` | GCP project ID |
| `GHCR_PAT` | GitHub PAT with `packages:write` |

### PostgreSQL

| Secret | Description |
|--------|-------------|
| `POSTGRES_USER` | PostgreSQL username |
| `POSTGRES_PASSWORD` | PostgreSQL password |
| `POSTGRES_DB` | Default database name (`rsky`) |

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

### rsky-labeler (standby — placeholders until activated)

| Secret | Description |
|--------|-------------|
| `MOD_SERVICE_DID` | Labeler service account DID |
| `MOD_SERVICE_EMAIL` | Labeler service account email |
| `MOD_SERVICE_PASSWORD` | Labeler service account password |

### web-client

| Secret | Description |
|--------|-------------|
| `NEXTAUTH_SECRET` | NextAuth.js session secret |

## Deploying

Push to `main` — GitHub Actions (`deploy.yml`) builds changed images and deploys to GKE.

Manual full redeploy (all services):

```bash
gh workflow run deploy.yml
```

Targeted single-service redeploy:

```bash
gh workflow run deploy.yml -f services=rsky-pds
```

## After First Deploy — DNS

Get Gateway IPs and set Cloudflare A records (DNS-only / grey cloud initially for cert issuance):

```bash
kubectl get gateway -n atproto \
  -o custom-columns='NAME:.metadata.name,IP:.status.addresses[0].value'
```

| DNS record | Gateway name |
|-----------|--------------|
| `pds.know-me.tools` | `pds-gateway` |
| `relay.know-me.tools` | `relay-gateway` |
| `feed.know-me.tools` | `feedgen-gateway` |
| `social.know-me.tools` | `web-client-gateway` |

## GCS HMAC Keys

Create via GCP Console → Cloud Storage → Settings → Interoperability → Create key for service account.

The `AWS_*` env var names are intentional — rsky-pds uses an S3-compatible Rust SDK that reads standard AWS env vars. The endpoint is set to `https://storage.googleapis.com`.

## Generating PDS Keys

```bash
openssl ecparam -name secp256k1 -genkey -noout | \
  openssl ec -text -noout 2>/dev/null | \
  grep priv -A 3 | tail -3 | tr -d ' :\n'
```

Generate three independent keys: JWT signing, PLC rotation, repo signing.

## Activating the Labeler

rsky-labeler is deployed in standby mode (all actions disabled). To activate:

1. Create a Bluesky account to serve as the labeler service account.
2. Register a labeler service record (`app.bsky.labeler.service`) on your PDS.
3. Get the account's DID:
   ```
   https://pds.know-me.tools/xrpc/com.atproto.identity.resolveHandle?handle=<your-handle>
   ```
4. Set GitHub secrets: `MOD_SERVICE_DID`, `MOD_SERVICE_EMAIL`, `MOD_SERVICE_PASSWORD`.
5. Update `k8s/rsky-labeler/configmap.yaml`: set `ENABLE_CREATE_REPORT`, `ENABLE_CREATE_LABEL`, `ENABLE_CREATE_TAG` to `"true"`.
6. Push to `main` — GitHub Actions deploys the updated ConfigMap and restarts the pod.

## rsky-relay Storage Notes

The relay StatefulSet uses WORKDIR `/data` in the container, which is also the PVC mount path:
- `relay.db` — SQLite for host bans and validator state
- `plc_directory.db` — SQLite for DID resolution cache
- `db/` — fjall LSM database for firehose event storage

Expand PVC when needed:

```bash
kubectl patch pvc rsky-relay-data-rsky-relay-0 -n atproto \
  -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'
```
