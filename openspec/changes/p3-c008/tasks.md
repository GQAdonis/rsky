# Tasks: p3-c008

## 1. Harness scaffold

- [ ] 1.1 Create top-level `conformance/` directory
- [ ] 1.2 Add a `docker-compose.yml` that boots rsky-pds (with Postgres + minio for S3) and upstream `@atproto/pds:0.4.220` (with SQLite) on different ports
- [ ] 1.3 Add a workload-driver Rust binary `conformance/driver/` that hits both PDSes with identical XRPC calls

## 2. Workload definition

- [ ] 2.1 Define the canonical workload (createAccount → createSession → applyWrites → createRecord → putRecord → deleteRecord → uploadBlob image + video → refreshSession → getRepo → getBlob → listRecords → importRepo round-trip)
- [ ] 2.2 Capture deterministic inputs (fixed seeds, fixed timestamps where possible) so output diffs are meaningful

## 3. Capture + diff

- [ ] 3.1 Subscribe to both firehoses; capture frames to disk
- [ ] 3.2 After workload, fetch `getRepo` CAR for each DID from both PDSes
- [ ] 3.3 Implement frame diff that compares only lexicon-defined fields (ignore timestamps, signing-key-derived randomness)
- [ ] 3.4 Implement CAR-byte diff (commit CID + MST root + blocks)
- [ ] 3.5 Report PASS/FAIL with per-section detail

## 4. CI integration

- [ ] 4.1 Add `.github/workflows/conformance.yaml` that runs the harness on push to main + PRs touching rsky-pds, rsky-repo, rsky-lexicon, rsky-firehose
- [ ] 4.2 Workflow fails if any diff is non-empty
- [ ] 4.3 Upload captured frames + CARs as workflow artifacts on failure for offline diffing

## 5. Documentation

- [ ] 5.1 Document how to run the harness locally (`cd conformance && docker compose up && cargo run -p conformance-driver`)
- [ ] 5.2 Document expected runtime and fixture refresh cadence
