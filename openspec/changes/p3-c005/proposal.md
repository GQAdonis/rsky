# p3-c005: rsky-lexicon refresh against upstream lexicons HEAD

## Why

`rsky-lexicon` was generated from a pre-0.4.107 snapshot of `bluesky-social/atproto/lexicons/`. Upstream is at 0.4.220 (commit `877e629`). The lexicon JSON has changed across multiple `lex-data` bumps: sync v1.1 fields, ozone surface expansion, chat lexicon updates, sync `prev` fields, removal of deprecated fields (PR #2506), labeler declaration fields (PR #3521 / #3579), `listReposByCollection` (PR #3524), `tid`/`record-key` formats (PR #2378). The Rust lexicon types must catch up before the dependent rsky-pds and rsky-repo work can rely on them.

## What Changes

- Snapshot upstream lexicons at commit `877e629` into a vendored copy under `rsky-lexicon/lexicons/`.
- Re-run rsky-lexicon's codegen against the snapshot.
- Reconcile the regenerated types with consumers (`rsky-pds`, `rsky-repo`, `rsky-firehose`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`) — fix any compile breaks introduced by lexicon changes.
- Document the refresh procedure (snapshot, codegen, reconcile, test) in `rsky-lexicon/README.md` so future bumps are routine.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "PDS MUST track upstream lexicons within one minor version" requirement.

## Impact

- Largest blast radius of any phase-3 change: every consumer of `rsky-lexicon` may need a small fix.
- Recommended agent: cursor (parallel multi-file refactor) or claude-code with a careful per-crate sweep.
- Run `cargo check --workspace` early and often.
- This change is a prerequisite for p3-c006 (sync v1.1 in rsky-repo).
