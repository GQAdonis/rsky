# rsky-lexicon upstream version

This crate's Rust types were last regenerated against:

```
Repository: https://github.com/bluesky-social/atproto
Commit:     877e629
Tag:        (nearest: v0.4.220-equivalent lexicon snapshot, 2026-04-24)
```

## Tracking policy

rsky-lexicon targets staying within **one minor version** of upstream `bluesky-social/atproto` lexicons.

## Refresh procedure

When upstream lexicons have moved ahead, run the following:

### 1. Snapshot upstream lexicons

```bash
# Sparse-clone the atproto repo at the target commit
git clone --filter=blob:none --sparse https://github.com/bluesky-social/atproto atproto-upstream
cd atproto-upstream
git sparse-checkout set lexicons
git checkout <TARGET_COMMIT>
cp -r lexicons/ /path/to/rsky/rsky-lexicon/lexicons/
cd ..
rm -rf atproto-upstream
```

### 2. Record the upstream commit

Update `rsky-lexicon/UPSTREAM_VERSION.md` with the new commit SHA and date.

### 3. Apply Rust type changes

Because `rsky-lexicon` currently uses hand-authored Rust types (not auto-generated from the lexicon JSON), the refresh is manual:

1. Diff the changed lexicon JSON files against the previous snapshot
2. For each changed lexicon, update the corresponding `.rs` file under `rsky-lexicon/src/`
3. Add new types for new lexicon definitions; remove or deprecate removed ones
4. Run `cargo check -p rsky-lexicon` — must pass with zero errors

### 4. Verify consumers

```bash
cargo check -p rsky-pds
cargo check -p rsky-repo
cargo check -p rsky-firehose
cargo check -p rsky-feedgen
cargo check -p rsky-labeler
cargo check -p rsky-jetstream-subscriber
```

Fix any breakage before committing.

### 5. Commit

```bash
git add rsky-lexicon/
git commit -m "feat(rsky-lexicon): refresh against upstream lexicons <COMMIT_SHA>"
```

Update `UPSTREAM_VERSION.md` in the same commit.
