# rsky Project Constraints

## Blocking (NEVER do)
- Never upgrade Rust beyond 1.86 or change the workspace edition without explicit discussion
- Never add dependencies not already in the workspace without explicit discussion
- Never commit secrets, private keys, API tokens, or real service credentials
- Never perform large cross-crate refactors without explicit discussion
- Never use `unwrap()` or `expect()` in non-test code — use `?` and typed errors
- Never use `#[allow(unused)]` to silence real errors — fix them

## Style
- All Rust 2024 edition for new crates (rsky-wintermute, rsky-appview)
- Rust 2021 for existing crates — do not change their edition
- `cargo fmt` must pass before any commit touching Rust files
- Prefer narrow, crate-scoped `cargo check -p <crate>` over full workspace builds
- New crates go in `rsky-appview/crates/<name>/` as an inner workspace
- K8s manifests go in `k8s/rsky-appview/`
- GitHub Actions build job follows the pattern in deploy.yml

## Architecture
- AppView HTTP API: Axum 0.8, Tower middleware, no Rocket
- DB access: sqlx + deadpool-postgres (not Diesel — Diesel is for existing PDS/feedgen)
- Queue: Fjall (same as wintermute)
- DID cache: Moka async cache
- Custom lexicon namespace: `tools.know-me.*`
- LiveKit integration: livekit-server-sdk (Rust crate) for token generation
- CRDT: `yrs` (Rust Yjs port) for data channel session state
