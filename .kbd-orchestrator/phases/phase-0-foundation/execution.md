# Execution Contract: phase-0-foundation

**Backend**: openspec  
**Primary agent**: claude-code  
**Started**: 2026-04-27  

## Dispatch Order

1. p0-c001 — Fix Dockerfiles (claude-code, sequential)
2. p0-c002 — PostgreSQL StatefulSet (claude-code, sequential)
3. p0-c003 + p0-c004 — PDS manifests + Web client (parallel)
4. p0-c005 — Deploy workflow + ArgoCD (after 3+4 complete)

## QA Gate

Each change with ≥3 files: validate against constraints.md before archiving.
Skip QA for changes with <3 files (none in this phase qualify for skip).

## Progress Tracking

Update `progress.json` after each change completes.
Commit after each change: `git add . && git commit -m "kbd: p0-cXXX complete"`
