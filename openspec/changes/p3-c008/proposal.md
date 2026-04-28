# p3-c008: Federation conformance harness (rsky-pds vs @atproto/pds@0.4.220)

## Why

The Postgres-only divergence means there is no upstream "Postgres-only PDS" to copy. Every parity claim has to be verified end-to-end. A side-by-side conformance harness — running rsky-pds and `@atproto/pds@0.4.220` against identical inputs and diffing their outputs — is the only way to detect protocol regressions before they ship.

This is the highest-leverage investment in the phase. Without it, every other change is theoretical.

## What Changes

- Add a new test target `rsky-pds-conformance` (top-level dir or test bench) that:
  1. Boots a fresh rsky-pds (Postgres + S3) and a fresh upstream `@atproto/pds@0.4.220` (SQLite + DiskBlobStore) in containers.
  2. Drives a canonical workload through both via `goat` or a hand-rolled XRPC driver: createAccount, createSession, applyWrites, createRecord, putRecord, deleteRecord, uploadBlob (image and video), refreshSession, requestEmailConfirmation, getRepo, getBlob, listRecords, importRepo round-trip.
  3. Captures both firehoses (`subscribeRepos`).
  4. Diffs (a) firehose frames on lexicon-defined fields, (b) `getRepo` CAR bytes per DID, (c) per-record `getRecord` payloads.
  5. Reports pass/fail.
- Add a CI workflow that runs the harness on every push to `main` and on PRs that touch `rsky-pds`, `rsky-repo`, `rsky-lexicon`, or `rsky-firehose`.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "Federation conformance harness MUST exist and MUST pass" requirement.

## Impact

- New top-level `conformance/` directory with the harness driver and fixtures.
- New GitHub Actions workflow `.github/workflows/conformance.yaml`.
- Pulls upstream `@atproto/pds@0.4.220` Docker image as a CI fixture.
- This is the gate item: p3-c009 → p3-c012 (OAuth) cannot be claimed correct without it.
