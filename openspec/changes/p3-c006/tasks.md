# Tasks: p3-c006

## 1. rsky-repo core changes

- [ ] 1.1 Add `prev` CID field to commit-op data model in `rsky-repo`
- [ ] 1.2 Emit covering proofs on every commit (CAR slice that proves the MST diff)
- [ ] 1.3 Add unit tests asserting `prev` is populated correctly across multi-write sequences

## 2. rsky-pds sequencer event shapes

- [ ] 2.1 Extend `CommitEvt` in `rsky-pds/src/sequencer/events.rs` with `prev` CIDs per op and the covering-proof bytes
- [ ] 2.2 Add `SyncEvt` emission on account create and on activate/deactivate transitions
- [ ] 2.3 Keep `#handle` and `#tombstone` emission for back-compat (matching upstream's deprecation behavior)

## 3. WebSocket frame serialization

- [ ] 3.1 Update `subscribe_repos.rs` to serialize the new event shapes using the rsky-lexicon types
- [ ] 3.2 Verify frame byte layout matches upstream by hand-diffing one frame

## 4. Tests

- [ ] 4.1 Apply a canonical write sequence and capture rsky's firehose frames
- [ ] 4.2 Apply the same sequence to upstream `@atproto/pds@0.4.220` and capture its frames
- [ ] 4.3 Diff lexicon-defined fields — must match
- [ ] 4.4 Note: full conformance check is owned by p3-c008; this task is a smoke check

## 5. Verify

- [ ] 5.1 `cargo test --release -p rsky-repo` passes
- [ ] 5.2 `cargo test --release -p rsky-pds` passes
