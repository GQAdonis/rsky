# p1-c006: CI Workflow Updates + README

**Phase**: phase-1-relay-feedgen
**Priority**: 6 (depends on all p1-c001..p1-c005)
**Assigned to**: claude-code

## Overview

Three integration updates to wire the new services into the deploy pipeline:

1. Add `rsky-relay` to the GitHub Actions build matrix
2. Extend `inject-secrets` job to cover all new secrets
3. Update `k8s/README.md` with new secrets table entries
4. Update `.github/scripts/update-image-tags.sh` to handle rsky-relay

---

## 1. GitHub Actions — Matrix Update

Add to `.github/workflows/deploy.yaml` matrix:

```yaml
- service: rsky-relay
  dockerfile: rsky-relay/Dockerfile
```

Matrix becomes 6 services: rsky-pds, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber, web-client, **rsky-relay**.

---

## 2. GitHub Actions — inject-secrets Job Extension

Add three new secret injection steps after the existing `rsky-pds` injection:

### Inject rsky-relay secret
```yaml
- name: Inject rsky-relay secret
  env:
    RELAY_ADMIN_PASSWORD: ${{ secrets.RELAY_ADMIN_PASSWORD }}
  run: |
    envsubst < k8s/rsky-relay/secret.yaml | kubectl apply -f -
```

### Inject rsky-feedgen secret
```yaml
- name: Inject rsky-feedgen secret
  env:
    POSTGRES_USER: ${{ secrets.POSTGRES_USER }}
    POSTGRES_PASSWORD: ${{ secrets.POSTGRES_PASSWORD }}
    RSKY_API_KEY: ${{ secrets.RSKY_API_KEY }}
  run: |
    envsubst < k8s/rsky-feedgen/secret.yaml | kubectl apply -f -
```

### Inject rsky-labeler secret
```yaml
- name: Inject rsky-labeler secret
  env:
    MOD_SERVICE_DID: ${{ secrets.MOD_SERVICE_DID }}
    MOD_SERVICE_EMAIL: ${{ secrets.MOD_SERVICE_EMAIL }}
    MOD_SERVICE_PASSWORD: ${{ secrets.MOD_SERVICE_PASSWORD }}
  run: |
    envsubst < k8s/rsky-labeler/secret.yaml | kubectl apply -f -
```

### Inject rsky-jetstream-subscriber secret
```yaml
- name: Inject rsky-jetstream-subscriber secret
  env:
    RSKY_API_KEY: ${{ secrets.RSKY_API_KEY }}
  run: |
    envsubst < k8s/rsky-jetstream-subscriber/secret.yaml | kubectl apply -f -
```

---

## 3. update-image-tags.sh

The script already handles any service under `k8s/` with `IMAGE_TAG` references. No changes needed as long as the rsky-relay manifests use the `IMAGE_TAG` placeholder pattern.

---

## 4. k8s/README.md — New Secrets Table Entries

Add to the Required GitHub Secrets table:

| Secret | Description |
|--------|-------------|
| `RELAY_ADMIN_PASSWORD` | rsky-relay admin API password |
| `RSKY_API_KEY` | Shared API key for jetstream→feedgen queue auth |
| `MOD_SERVICE_DID` | Labeler service account DID (placeholder until credentials available) |
| `MOD_SERVICE_EMAIL` | Labeler service account email (placeholder) |
| `MOD_SERVICE_PASSWORD` | Labeler service account password (placeholder) |

Add labeler activation instructions section (mirrors p1-c005 proposal notes).

---

## Files to Modify

```
.github/workflows/deploy.yaml        — add rsky-relay to matrix; add 4 secret injection steps
k8s/README.md                        — new secrets + labeler activation instructions
```
