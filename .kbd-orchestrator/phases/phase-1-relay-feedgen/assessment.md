# Assessment: phase-1-relay-feedgen
> Generated: 2026-04-27 | Tool: claude-code | Protocol: kbd-assess

## Phase Goal

Deploy the remaining four ATProto services — rsky-relay, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber — to the `atproto` namespace on GKE, wired together so that:
- rsky-relay crawls the Bluesky firehose and exposes `relay.know-me.tools`
- rsky-feedgen provides algorithmic feeds at `feed.know-me.tools`
- rsky-labeler and rsky-jetstream-subscriber run as background workers feeding rsky-feedgen

## Phase-0 Foundation State (COMPLETE)

All phase-0 work is in place:
- ✅ `k8s/namespace.yaml` — `atproto` namespace
- ✅ `k8s/postgresql/` — pgvector:pg17 StatefulSet, `atproto-ssd-immediate` StorageClass
- ✅ `k8s/rsky-pds/` — full k8s stack (StatefulSet, Service, Gateway, Certificate, HTTPRoutes)
- ✅ `k8s/web-client/` — Deployment (2 replicas), Gateway for `social.know-me.tools`
- ✅ `k8s/argocd/application.yaml` — ArgoCD GitOps wired to `k8s/` path
- ✅ `.github/workflows/deploy.yaml` — matrix build for 5 services, envsubst secret injection
- ✅ All 5 Dockerfiles use correct local `COPY . .` multi-stage pattern

## Gap Analysis by Service

---

### 1. rsky-relay

**Status**: Blocked — source code patch required before k8s manifests will work.

**Critical Gap — Hardcoded Bind Address**

`rsky-relay/src/server/server.rs:157` binds to `127.0.0.1:9000` unconditionally:
```rust
let listener = TcpListener::bind(format!("127.0.0.1:{PORT}"))?;
```

In a container, this means the service is unreachable from outside the pod. The Kubernetes Service selector will route traffic to the pod but the TCP listener will refuse the connection (loopback only). This must be patched to `0.0.0.0:{PORT}` or made configurable via `RELAY_ADDRESS` env var before the k8s deployment will work.

**Critical Gap — File Paths are Relative to CWD**

All database files use relative paths from the process CWD:
- `relay.db` — SQLite, opened in `src/validator/manager.rs` and `src/crawler/manager.rs`
- `plc_directory.db` — SQLite, opened in `src/validator/resolver.rs`
- `db/` — fjall LSM database, opened in `src/types.rs` as `fjall::Config::new("db")`

There is no env var or CLI flag to configure these paths. The Dockerfile WORKDIR must be set to the PVC mount point so that relative paths resolve to persistent storage. If WORKDIR is `/app`, then the PVC must mount at `/app`.

**Gap — No Dockerfile**

`rsky-relay/Dockerfile` does not exist. Must be created following the same pattern as rsky-pds.

**Gap — No k8s Manifests**

`k8s/rsky-relay/` directory does not exist.

**Service Characteristics:**
- Port: 9000 (hardcoded in `src/config.rs:18`)
- HTTP server: Yes — serves firehose WebSocket + REST admin endpoints
- Health endpoint: `/_health` returns 200 with body "ok"
- Storage: SQLite (`relay.db`, `plc_directory.db`) + fjall (`db/`) — all relative paths, need PVC
- PVC sizing: fjall configured for 320 GiB disk size (the `DISK_SIZE` constant) — must provision >= 50Gi SSD to start
- External outbound: crawls `relay1.us-west.bsky.network` for initial host discovery; then crawls all PDS instances it discovers
- `RELAY_ADMIN_PASSWORD` — optional but strongly recommended for production

**TLS at relay level**: rsky-relay can accept TLS directly via `--certs` and `--private-key` CLI flags, but in k8s with TLS-termination at Envoy Gateway, these are not needed. The service runs plain HTTP inside the cluster.

---

### 2. rsky-feedgen

**Status**: Ready for k8s manifests — no source patch needed.

**Gap — No k8s Manifests**

`k8s/rsky-feedgen/` directory does not exist.

**Service Characteristics:**
- Port: 3000 (Rocket default, exposed in existing Dockerfile)
- HTTP server: Yes — serves AT Protocol lexicon feed endpoints
- Health endpoint: None identified — use `/xrpc/app.bsky.feed.describeFeedGenerator` or `/well_known` as readiness check (returns 200 with valid JSON if wired correctly)
- Storage: PostgreSQL only — uses `DATABASE_URL` from shared postgresql StatefulSet
- Read replicas: `READ_REPLICA_URL_1`, `READ_REPLICA_URL_2` — both optional; can point to same primary in phase 1
- Domain: `feed.know-me.tools`

**Key env vars needed in ConfigMap:**
```
FEEDGEN_SERVICE_DID=did:web:feed.know-me.tools
FEEDGEN_HOSTNAME=feed.know-me.tools
SHOW_SPONSORED_POST=0
TRENDING_PERCENTILE=0.9
ROCKET_PORT=3000
ROCKET_ADDRESS=0.0.0.0
```

**Key env vars needed in Secret (envsubst):**
```
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
READ_REPLICA_URL_1=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
READ_REPLICA_URL_2=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
RSKY_API_KEY=${RSKY_API_KEY}
```

**Note**: rsky-feedgen needs its own PostgreSQL database (`rsky_feedgen`), separate from rsky-pds (`rsky`). A `k8s/postgresql/init-scripts/` ConfigMap with a `CREATE DATABASE rsky_feedgen;` SQL script can handle this automatically via the `POSTGRES_INITDB_ARGS` pattern or a post-start init container.

---

### 3. rsky-labeler

**Status**: Ready for k8s manifests — no source patch needed.

**Gap — No k8s Manifests**

`k8s/rsky-labeler/` directory does not exist.

**Service Characteristics:**
- Port: None — this is a **background worker only**, not an HTTP server
- No Gateway, Certificate, or HTTPRoute needed
- Kind: `Deployment` (stateless, replicas: 1)
- Storage: None (stateless WebSocket subscriber)
- External dependencies:
  - Connects outbound to firehose WebSocket (`FEEDGEN_SUBSCRIPTION_PATH`, default `wss://bsky.network`)
  - Reports to moderation service (`BSKY_AGENT_URL`, default `https://bsky.social`)
- No readiness/liveness HTTP probe — use `exec` probe: `["pgrep", "-x", "rsky-labeler"]`

**Key env vars needed in ConfigMap:**
```
FEEDGEN_SUBSCRIPTION_PATH=wss://bsky.network
FEEDGEN_SUBSCRIPTION_ENDPOINT=com.atproto.sync.subscribeRepos
BSKY_AGENT_URL=https://bsky.social
MOD_SERVICE_LABEL=antiblack-harassment
MOD_SERVICE_LABEL_REASON=Explicit slur filter
MOD_SERVICE_REASON=com.atproto.moderation.defs#reasonRude
ENABLE_CREATE_REPORT=true
ENABLE_CREATE_LABEL=true
ENABLE_CREATE_TAG=true
```

**Key env vars needed in Secret (envsubst):**
```
MOD_SERVICE_DID=${MOD_SERVICE_DID}
MOD_SERVICE_EMAIL=${MOD_SERVICE_EMAIL}
MOD_SERVICE_PASSWORD=${MOD_SERVICE_PASSWORD}
```

---

### 4. rsky-jetstream-subscriber

**Status**: Ready for k8s manifests — no source patch needed.

**Gap — No k8s Manifests**

`k8s/rsky-jetstream-subscriber/` directory does not exist.

**Service Characteristics:**
- Port: None — this is a **background worker only**
- No Gateway, Certificate, or HTTPRoute needed
- Kind: `Deployment` (stateless, replicas: 1)
- Storage: None (stateless WebSocket subscriber)
- External dependencies:
  - Connects outbound to Jetstream (`JETSTREAM_SERVER_ENDPOINT`, default `wss://jetstream1.us-west.bsky.network`)
  - Sends HTTP PUT to feedgen queue endpoints (`FEEDGEN_QUEUE_ENDPOINT`, must point to rsky-feedgen)
- No HTTP probe — use `exec` probe: `["pgrep", "-x", "rsky-jetstream-subsc"]`

**Key env vars needed in ConfigMap:**
```
JETSTREAM_SERVER_ENDPOINT=wss://jetstream1.us-west.bsky.network
FEEDGEN_QUEUE_ENDPOINT=http://rsky-feedgen:3000
FILTER_PARAM=
```

**Key env vars needed in Secret (envsubst):**
```
RSKY_API_KEY=${RSKY_API_KEY}
```

---

## Missing k8s Infrastructure

### PostgreSQL init script for multiple databases

Currently the PostgreSQL StatefulSet creates only the database specified by `POSTGRES_DB`. rsky-feedgen needs a separate `rsky_feedgen` database. Options:
1. Add an `initdb` ConfigMap with a SQL script mounted as `/docker-entrypoint-initdb.d/init.sql`
2. Or use a single `rsky` database for both (simpler but less clean)

**Recommended**: initdb ConfigMap — aligns with production practice.

### GitHub Actions: missing services in matrix

The existing `.github/workflows/deploy.yaml` matrix includes `rsky-pds`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`, `web-client`. It does NOT include `rsky-relay` because `rsky-relay/Dockerfile` did not exist in phase 0. Once the Dockerfile is created, add `rsky-relay` to the matrix.

The `inject-secrets` job also needs to be extended to handle the new secrets (labeler credentials, RSKY_API_KEY, relay admin password).

---

## Open Questions

**Q1**: rsky-relay bind address patch — patch `127.0.0.1` → `0.0.0.0` directly in source, or add a `RELAY_ADDRESS` env var override? Given the project avoids large refactors (CONTRIBUTING.md), a minimal 1-line patch is preferred. Recommendation: patch to `0.0.0.0` directly (it's a container deployment; loopback-only makes no sense in k8s).

**Q2**: rsky-relay PVC size — fjall is configured for 320 GiB max disk. For phase 1, provision 50Gi pd-ssd to start (expandable via `allowVolumeExpansion: true`). Is that acceptable or should we start larger?

**Q3**: Multiple PostgreSQL databases — add init script to create `rsky_feedgen` database, or use a single `rsky` database for everything? Recommendation: separate DB per service (`rsky` for pds, `rsky_feedgen` for feedgen) via initdb script.

**Q4**: Labeler credentials — `MOD_SERVICE_DID`, `MOD_SERVICE_EMAIL`, `MOD_SERVICE_PASSWORD` must point to a valid ATProto account with labeler permissions. For phase 1, does the team have a Bluesky account to use for this, or should rsky-labeler be deployed in a disabled/no-op mode initially?

---

## Gaps Summary Table

| Gap | Service | Severity | Action Required |
|-----|---------|----------|-----------------|
| Hardcoded `127.0.0.1` bind address | rsky-relay | **BLOCKING** | Patch `src/server/server.rs:157` |
| No `rsky-relay/Dockerfile` | rsky-relay | **BLOCKING** | Create Dockerfile (same pattern as rsky-pds minus libpq) |
| No `k8s/rsky-relay/` manifests | rsky-relay | **BLOCKING** | Create full k8s stack |
| No `k8s/rsky-feedgen/` manifests | rsky-feedgen | **BLOCKING** | Create k8s stack (Deployment, no PVC) |
| No `k8s/rsky-labeler/` manifests | rsky-labeler | **BLOCKING** | Create k8s stack (Deployment, no port, no Gateway) |
| No `k8s/rsky-jetstream-subscriber/` manifests | rsky-jetstream | **BLOCKING** | Create k8s stack (Deployment, no port, no Gateway) |
| PostgreSQL multi-db init | postgresql | **HIGH** | Add initdb ConfigMap for `rsky_feedgen` database |
| CI matrix missing rsky-relay | deploy workflow | **HIGH** | Add `rsky-relay` to matrix after Dockerfile created |
| CI inject-secrets missing new secrets | deploy workflow | **HIGH** | Add labeler secrets + RSKY_API_KEY injection |
| fjall/SQLite paths are CWD-relative | rsky-relay | **HIGH** | Set Dockerfile WORKDIR = PVC mountPath `/data` |

## Definition of Done

Phase 1 is complete when:
- [ ] rsky-relay binds to `0.0.0.0:9000` in source
- [ ] rsky-relay has a working Dockerfile
- [ ] All four services have complete k8s manifests in their respective `k8s/<service>/` directories
- [ ] rsky-relay and rsky-feedgen have Gateway + Certificate + HTTPRoutes for their public domains
- [ ] rsky-labeler and rsky-jetstream-subscriber run as headless Deployments
- [ ] PostgreSQL init script creates both `rsky` and `rsky_feedgen` databases
- [ ] GitHub Actions CI matrix and inject-secrets job cover all 6 services
- [ ] `k8s/README.md` updated with new secrets table entries
- [ ] All `secret.yaml` files use `envsubst` placeholders (C-001)
- [ ] All Gateway resources have matching Certificate resources (C-007)
