# p3-c004: Sequencer race fix + recovery hardening

## Why

Upstream `@atproto/pds` PR #3580 (0.4.104) fixed a bug where racing writes to the same repository could be sequenced out of order. Upstream PR #4408 (0.4.217) hardened the sequencer recovery script to skip stored commit ops with malformed `<nsid>/<rkey>` paths instead of aborting recovery. Both apply to `rsky-pds`; neither has been verified or applied in our codebase. Out-of-order firehose events are a federation-correctness problem; an aborting recovery script is an operational problem.

## What Changes

- Move `sequenceCommit` (or its rsky equivalent) inside the actor-store transaction so two concurrent writes to the same repo cannot interleave between commit-write and sequence-allocation. Today rsky's sequencer assigns sequence numbers from a row-level counter, which is racy under concurrent writes.
- Wrap the sequencer write in a per-DID async lock (or rely on Postgres advisory locks per DID) to serialize sequence assignment within a repo while keeping cross-repo writes parallel.
- Harden any sequencer recovery / replay path so that invalid `<nsid>/<rkey>` paths in stored commit ops are logged and skipped, not raised as fatal.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "Sequencer MUST sequence concurrent writes to the same repo in a deterministic order" requirement.

## Impact

- Edits to `rsky-pds/src/sequencer/{mod.rs, events.rs, outbox.rs}` and the actor-store write path under `rsky-pds/src/actor_store/repo/`.
- May require a Postgres advisory-lock function call per write transaction.
- Adds a regression test exercising concurrent writes against the same DID.
