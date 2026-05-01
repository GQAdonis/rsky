# Execution — phase-3-pds-feature-parity

**Started**: 2026-05-01  
**Tool**: opencode  
**Backend**: `openspec` (all 12 changes are OpenSpec-backed under `openspec/changes/p3-c00N/`)

---

## Backend Selection

| Criterion | Decision |
|-----------|----------|
| OpenSpec available? | YES — `openspec/specs/pds-server/spec.md` exists with 14 requirements |
| Spec-backed traceability required? | YES — Postgres-only divergence means every parity claim must be traceable |
| Network access needed? | NO for G2/G3/G4. YES for p3-c005 full codegen (delegated to human) |
| Selected backend | `openspec` for G1–G4 residuals |

---

## Remaining Work (from 2026-04-30 assessment)

| Gap | Change | Priority | Status |
|-----|--------|----------|--------|
| G1 — Lexicon uncommitted; no UPSTREAM_VERSION.md; no refresh procedure docs | p3-c005 | HIGH | EXECUTING |
| G2 — auth_verifier.rs OAuth token validation stub (@TODO line 797) | p3-c012 | HIGH | EXECUTING |
| G3 — pipethrough.rs scope gate missing | p3-c012 | HIGH | EXECUTING |
| G4 — requestCrawl has no debounce | untracked | MEDIUM | EXECUTING |

---

## QA Gate

Per kbd-execute protocol, artifact-refiner QA is **skipped** for G1 (documentation-only
files + one-file lexicon commit) and G4 (fewer than 3 files). QA applies to G2+G3
(multi-file auth changes touching security-critical paths).

---

## Dispatch log

- 2026-05-01: opencode begins execution of G1 → G4
- p3-c005: partial lexicon work already present in working tree; completing with
  UPSTREAM_VERSION.md, README refresh procedure, cargo check verification, commit
- p3-c012: completing auth_verifier DPoP/OAuth discriminator + scope wiring +
  pipethrough scope gate
- G4 (untracked): requestCrawl debounce added to crawlers.rs
