# p2-c007: Smoke test all endpoints

**Phase**: phase-2-commit-and-deploy  
**Priority**: 7 (depends on p2-c006; human task)  
**Assigned to**: human  
**Type**: operational runbook

## Overview

Verify all 4 public endpoints are reachable, TLS is valid, and services are healthy.

## Health Checks

```bash
# rsky-pds
curl -s https://pds.know-me.tools/xrpc/_health | jq .
# Expected: {"version":"<semver>"}

# rsky-relay
curl -s https://relay.know-me.tools/_health
# Expected: 200 OK (body may be empty or {"status":"ok"})

# rsky-feedgen
curl -s https://feed.know-me.tools/xrpc/_health | jq .
# Expected: {"version":"<semver>"}

# web client (Ouranos)
curl -sI https://social.know-me.tools | head -5
# Expected: HTTP/2 200
```

## Pod Status

```bash
kubectl get pods -n atproto
```

Expected states:
- `rsky-pds-0` — Running
- `rsky-relay-0` — Running
- `rsky-feedgen-*` — Running (1 replica)
- `rsky-labeler-*` — Running (standby — no active work)
- `rsky-jetstream-subscriber-*` — Running
- `web-client-*` — Running (2 replicas)
- `postgresql-0` — Running

## PDS XRPC Check

```bash
# Resolve a handle (will work once DNS + TLS are up)
curl "https://pds.know-me.tools/xrpc/com.atproto.identity.resolveHandle?handle=pds.know-me.tools"
```

## If Services Are Not Ready

```bash
# View pod logs
kubectl logs -n atproto deployment/rsky-feedgen --tail=50
kubectl logs -n atproto statefulset/rsky-relay --tail=50
kubectl logs -n atproto statefulset/rsky-pds --tail=50

# Check events
kubectl get events -n atproto --sort-by='.lastTimestamp' | tail -20
```
