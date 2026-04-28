# Tasks: p3-c005

## 1. Snapshot upstream lexicons

- [ ] 1.1 Sparse-clone `bluesky-social/atproto` at commit `877e629` and copy `lexicons/` into `rsky-lexicon/lexicons/`
- [ ] 1.2 Record the upstream commit SHA in `rsky-lexicon/UPSTREAM_VERSION.md`

## 2. Regenerate Rust types

- [ ] 2.1 Run rsky-lexicon's codegen against the new snapshot
- [ ] 2.2 Commit the regenerated types

## 3. Reconcile consumers

- [ ] 3.1 `cargo check -p rsky-pds` and fix any breakage
- [ ] 3.2 `cargo check -p rsky-repo` and fix any breakage
- [ ] 3.3 `cargo check -p rsky-firehose` and fix any breakage
- [ ] 3.4 `cargo check -p rsky-feedgen` and fix any breakage
- [ ] 3.5 `cargo check -p rsky-labeler` and fix any breakage
- [ ] 3.6 `cargo check -p rsky-jetstream-subscriber` and fix any breakage

## 4. Document refresh procedure

- [ ] 4.1 In `rsky-lexicon/README.md`, document the snapshot → codegen → reconcile → test loop
- [ ] 4.2 Note the rolling target: stay within one minor version of upstream

## 5. Verify

- [ ] 5.1 `cargo check --workspace` passes
- [ ] 5.2 `cargo test --workspace` passes (or surfaces new test breakages tied to lexicon changes, which then become follow-up tasks)
