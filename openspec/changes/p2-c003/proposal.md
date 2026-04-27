# p2-c003: Seed GitHub Actions secrets

**Phase**: phase-2-commit-and-deploy  
**Priority**: 3 (depends on p2-c002; human task)  
**Assigned to**: human  
**Type**: operational runbook

## Overview

All 17 GitHub Actions secrets must be set before CI can complete the inject-secrets job.
Set under: repository Settings → Secrets and variables → Actions → New repository secret.

## Secret Checklist

### Infrastructure

| Secret | Value |
|--------|-------|
| `GKE_SA_KEY` | GCP service account JSON (same SA used for conduit project) |
| `GKE_PROJECT_ID` | GCP project ID |
| `GHCR_PAT` | GitHub PAT with `packages:write` + `contents:write` |

### PostgreSQL

| Secret | Value |
|--------|-------|
| `POSTGRES_USER` | PostgreSQL username (e.g. `rsky`) |
| `POSTGRES_PASSWORD` | Strong generated password |

### rsky-pds

| Secret | Value |
|--------|-------|
| `PDS_ADMIN_PASS` | PDS admin password |
| `PDS_JWT_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex |
| `PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex |
| `PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX` | secp256k1 private key hex |
| `PDS_MAILGUN_API_KEY` | Mailgun API key |
| `PDS_MAILGUN_DOMAIN` | Mailgun sending domain |
| `GCS_HMAC_ACCESS_KEY` | GCS HMAC access key (S3-compatible) |
| `GCS_HMAC_SECRET_KEY` | GCS HMAC secret key |
| `GCS_BUCKET_NAME` | GCS bucket name for PDS blobs |

### rsky-relay

| Secret | Value |
|--------|-------|
| `RELAY_ADMIN_PASSWORD` | Relay admin API password |

### rsky-feedgen + rsky-jetstream-subscriber

| Secret | Value |
|--------|-------|
| `RSKY_API_KEY` | Shared API key for jetstream→feedgen auth |

### rsky-labeler (use placeholder values)

| Secret | Value |
|--------|-------|
| `MOD_SERVICE_DID` | `placeholder` |
| `MOD_SERVICE_EMAIL` | `placeholder@placeholder.invalid` |
| `MOD_SERVICE_PASSWORD` | `placeholder` |

## Key Generation

Generate 3 independent secp256k1 private key hex values:
```bash
openssl ecparam -name secp256k1 -genkey -noout | \
  openssl ec -text -noout 2>/dev/null | \
  grep priv -A 3 | tail -3 | tr -d ' :\n'
```

## GCS Setup

1. GCP Console → Cloud Storage → Create bucket (or reuse existing)
2. GCP Console → Cloud Storage → Settings → Interoperability → Create HMAC key for service account
3. Note Access Key and Secret — these become `GCS_HMAC_ACCESS_KEY` and `GCS_HMAC_SECRET_KEY`
