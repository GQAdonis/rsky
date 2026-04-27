# p2-c002: Commit all phases 0+1 work

**Phase**: phase-2-commit-and-deploy  
**Priority**: 2 (depends on p2-c001)  
**Assigned to**: claude-code

## Overview

Commit all phases 0 and 1 work in three logical commits for clean history.
Nothing has been committed yet — 7 modified files and ~55 untracked paths.

## Commit 1: Relay source patch and Dockerfile

**Message**: `feat: patch rsky-relay bind address and add Dockerfile`

Files:
- `rsky-relay/src/server/server.rs` — bind `0.0.0.0` instead of `127.0.0.1`
- `rsky-relay/Dockerfile` — new multi-stage build with WORKDIR=/data

## Commit 2: Service Dockerfiles rewrite

**Message**: `feat: rewrite service Dockerfiles for containerized k8s deployment`

Files:
- `rsky-pds/Dockerfile`
- `rsky-feedgen/Dockerfile`
- `rsky-labeler/Dockerfile`
- `rsky-jetstream-subscriber/Dockerfile`
- `rsky-firehose/Dockerfile`
- `Dockerfile.web-client`

## Commit 3: GitOps infrastructure

**Message**: `feat: add k8s manifests, CI/CD, and KBD orchestration for atproto stack`

Files:
- `.gitignore`
- `CLAUDE.md`
- `k8s/` (all manifests — namespace, postgresql, pds, relay, feedgen, labeler, jetstream-subscriber, web-client, argocd)
- `.github/workflows/deploy.yaml`
- `.github/scripts/update-image-tags.sh`
- `.kbd-orchestrator/`
- `openspec/`

## Constraints

- C-001: No secrets committed — all secret.yaml files use `${VAR}` envsubst placeholders only
- C-004: No `ghcr.io/blacksky-algorithms` references in k8s manifests
