# p3-c006: rsky-repo sync v1.1 — prev CIDs, covering proofs, #sync event

## Why

Upstream `@atproto/repo` reached 0.7.0 → 0.9.x with sync v1.1 fully landed: each commit op carries a `prev` CID, every commit is accompanied by covering proofs, and the firehose includes `#sync` events on account creation and identity transitions while `#handle` and `#tombstone` are deprecated. Strict relays will reject firehose frames that lack these. The actual implementation work belongs in `rsky-repo` (the MST + commit machinery) and `rsky-pds` only consumes those primitives. This change updates `rsky-repo` and the rsky-pds firehose emitter to match upstream's wire format.

## What Changes

- In `rsky-repo`: add `prev` CID tracking per commit op and emit covering proofs alongside the diff blocks.
- In `rsky-pds/src/sequencer/events.rs`: extend `CommitEvt` to carry `prev` CIDs and the covering-proof CAR slice; add `SyncEvt` emission on account creation and on activate/deactivate identity transitions; mark `#handle` and `#tombstone` as deprecated (keep emitting them for back-compat where upstream still does).
- In `rsky-pds/src/apis/com/atproto/sync/subscribe_repos.rs`: serialize the new event shapes into the WebSocket frames upstream-compatibly.
- Confirm `rsky-lexicon` (after p3-c005) exposes the updated `SubscribeRepos*` types — wire them through.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "PDS MUST emit sync v1.1 firehose events" requirement.

## Impact

- Touches `rsky-repo` core MST/commit code — high blast radius, requires careful CAR-byte equivalence testing.
- Updates `rsky-pds/src/sequencer/events.rs` and `subscribe_repos.rs`.
- Depends on p3-c005 (lexicon refresh) for the new event types.
- Verified end-to-end by p3-c008 (federation conformance harness).
