# p1-c005: rsky-labeler + rsky-jetstream-subscriber Kubernetes Manifests

**Phase**: phase-1-relay-feedgen
**Priority**: 5 (depends on p1-c004 — jetstream needs rsky-feedgen Service to exist)
**Assigned to**: claude-code

## Overview

Two headless background workers in one change — both are stateless WebSocket subscribers with no HTTP server, no Gateway, no Certificate.

---

## rsky-labeler

**Kind**: Deployment (stateless)
**Replicas**: 1
**Mode**: **Standby** — deployed but disabled until real labeler credentials are available

### Standby Mode Configuration

All action flags set to `false` in ConfigMap. The service will connect to the firehose and read events, but take no action (no labels, no reports, no tags). Placeholder values used for required secrets.

### ConfigMap
```
FEEDGEN_SUBSCRIPTION_PATH=wss://bsky.network
FEEDGEN_SUBSCRIPTION_ENDPOINT=com.atproto.sync.subscribeRepos
BSKY_AGENT_URL=https://bsky.social
MOD_SERVICE_LABEL=antiblack-harassment
MOD_SERVICE_LABEL_REASON=Explicit slur filter
MOD_SERVICE_REASON=com.atproto.moderation.defs#reasonRude
ENABLE_CREATE_REPORT=false
ENABLE_CREATE_LABEL=false
ENABLE_CREATE_TAG=false
```

### Secret (envsubst template — placeholder values in CI until credentials available)
```
MOD_SERVICE_DID=${MOD_SERVICE_DID}
MOD_SERVICE_EMAIL=${MOD_SERVICE_EMAIL}
MOD_SERVICE_PASSWORD=${MOD_SERVICE_PASSWORD}
```

To activate: set `ENABLE_CREATE_REPORT=true` / `ENABLE_CREATE_LABEL=true` / `ENABLE_CREATE_TAG=true` in ConfigMap, and provide real credentials in GitHub secrets.

### Health Probe
No HTTP server. Use `exec` probe:
```yaml
livenessProbe:
  exec:
    command: ["pgrep", "-x", "rsky-labeler"]
  initialDelaySeconds: 10
  periodSeconds: 30
```

### Files
```
k8s/rsky-labeler/
├── configmap.yaml
├── secret.yaml
└── deployment.yaml
```

---

## rsky-jetstream-subscriber

**Kind**: Deployment (stateless)
**Replicas**: 1
**Wiring**: Points at `rsky-feedgen:3000` as the queue endpoint

### ConfigMap
```
JETSTREAM_SERVER_ENDPOINT=wss://jetstream1.us-west.bsky.network
FEEDGEN_QUEUE_ENDPOINT=http://rsky-feedgen:3000
FILTER_PARAM=
```

### Secret (envsubst template)
```
RSKY_API_KEY=${RSKY_API_KEY}
```

Note: `RSKY_API_KEY` must match the `RSKY_API_KEY` set in rsky-feedgen's secret — this is the shared API key that authenticates jetstream→feedgen queue writes.

### Health Probe
No HTTP server. Use `exec` probe:
```yaml
livenessProbe:
  exec:
    command: ["pgrep", "-x", "rsky-jetstream-sub"]
  initialDelaySeconds: 10
  periodSeconds: 30
```

### Files
```
k8s/rsky-jetstream-subscriber/
├── configmap.yaml
├── secret.yaml
└── deployment.yaml
```

---

## Activating the Labeler (Post-Phase Instructions)

To fully activate rsky-labeler:

1. Create a Bluesky account to serve as the labeler service account
2. Register it as a labeler via `app.bsky.labeler.service` record on your PDS
3. Get the account's DID (e.g., `did:plc:xxxxx`)
4. Set GitHub secrets:
   - `MOD_SERVICE_DID` = account DID
   - `MOD_SERVICE_EMAIL` = account email
   - `MOD_SERVICE_PASSWORD` = account password
5. Update ConfigMap: `ENABLE_CREATE_REPORT=true`, `ENABLE_CREATE_LABEL=true`, `ENABLE_CREATE_TAG=true`
6. Trigger a new deploy
