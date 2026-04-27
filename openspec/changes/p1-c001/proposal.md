# p1-c001: Fix rsky-relay — Bind Address Patch + Dockerfile

**Phase**: phase-1-relay-feedgen
**Priority**: 1 (blocks all other relay work)
**Assigned to**: claude-code

## Problem

`rsky-relay/src/server/server.rs:157` hardcodes the bind address as `127.0.0.1`:

```rust
let listener = TcpListener::bind(format!("127.0.0.1:{PORT}"))?;
```

In a Kubernetes pod, this means the TCP listener only accepts connections from within the same pod (loopback). The Kubernetes Service will route traffic to the pod IP, but the listener will refuse it. The service is unreachable.

Additionally, `rsky-relay/Dockerfile` does not exist — it cannot be built until the source is patched.

## Fix

### Source Patch

**File**: `rsky-relay/src/server/server.rs:157`

Change:
```rust
let listener = TcpListener::bind(format!("127.0.0.1:{PORT}"))?;
```

To:
```rust
let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))?;
```

This is a 1-line change. No architectural changes, no new dependencies.

### Dockerfile

Create `rsky-relay/Dockerfile` following the same workspace caching pattern as all other service Dockerfiles:
- `rust:1.86-slim` builder stage
- No libpq dependency (rsky-relay uses SQLite, not PostgreSQL)
- `libssl-dev`/`libssl3` (for TLS to external services)
- WORKDIR in runtime stage: `/data` — this is critical so that SQLite files (`relay.db`, `plc_directory.db`) and fjall (`db/`) resolve to PVC-backed paths
- Binary: `rsky-relay`
- Port: 9000
- No `EXPOSE` directive needed (internal cluster only for health checks; Envoy Gateway handles external)

## WORKDIR Strategy

rsky-relay opens all storage files with relative paths from CWD:
- `relay.db`
- `plc_directory.db`
- `db/` (fjall)

By setting the runtime container WORKDIR to `/data` and mounting the PVC at `/data`, all file I/O goes to persistent storage automatically — no source changes needed for paths.

## Files to Create/Modify

```
rsky-relay/src/server/server.rs    — 1-line patch
rsky-relay/Dockerfile              — new file
```
