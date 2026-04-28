# Tasks: p3-c004

## 1. Diagnose current race window

- [ ] 1.1 Read `rsky-pds/src/sequencer/mod.rs` to map where seq numbers are assigned vs where rows are committed
- [ ] 1.2 Reproduce the race with a focused test: spawn N concurrent applyWrites against the same DID and assert seq order

## 2. Serialize per-DID sequencing

- [ ] 2.1 Wrap `sequenceCommit` (rsky equivalent) inside the same Diesel transaction that writes the actor-store rows for the commit
- [ ] 2.2 Acquire a Postgres advisory lock keyed on the DID at the start of the transaction so concurrent writes against the same DID serialize
- [ ] 2.3 Verify cross-DID writes still run in parallel (no global lock)

## 3. Recovery hardening

- [ ] 3.1 Identify any sequencer recovery / replay code path in `rsky-pds/src/sequencer/` (or note that recovery is implicit and document accordingly)
- [ ] 3.2 In any path that parses stored commit ops, treat invalid `<nsid>/<rkey>` paths as warn-and-skip, not fatal
- [ ] 3.3 Log skipped paths with the offending `did + seq + path` for ops visibility

## 4. Tests

- [ ] 4.1 Concurrent-write regression test asserts strictly increasing seq across N=50 writes
- [ ] 4.2 Recovery test injects a malformed path and asserts recovery completes
- [ ] 4.3 `cargo test --release -p rsky-pds` passes
