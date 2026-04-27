# Execution: phase-1-relay-feedgen
> Generated: 2026-04-27 | Backend: openspec | Agent: claude-code

## Backend Selection

OpenSpec — `openspec/` directory exists, `change_backend: openspec` in project.json.

## Dispatch Order

```
p1-c001  (serial — blocks all relay work)
    ↓
p1-c002 ║ p1-c003  (parallel)
    ↓
p1-c004
    ↓
p1-c005
    ↓
p1-c006
```

## QA Gate Policy

- p1-c001: QA required (source patch + new Dockerfile, >3 files)
- p1-c002: QA required (modifies existing statefulset.yaml + new file)
- p1-c003: QA required (7 new files)
- p1-c004: QA required (8 new files)
- p1-c005: QA required (6 new files)
- p1-c006: QA required (modifies deploy.yaml + README)

## Constraint Checks Per Change

For each change, verify before marking DONE:
- C-001: No committed secrets (all secret.yaml use ${VAR} placeholders)
- C-003: Every service with a secret has a matching envsubst template
- C-004: No ghcr.io/blacksky-algorithms references
- C-007: Every Gateway has a matching Certificate resource
- W-001: rsky-relay uses StatefulSet
- W-002: rsky-feedgen/labeler/jetstream use Deployment
- W-003: Each service gets its own Gateway (relay and feedgen only — workers get none)
