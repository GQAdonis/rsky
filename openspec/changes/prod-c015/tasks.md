# Tasks: prod-c015

- [x] Check `appview-auth/Cargo.toml` for `base64` dependency; add if missing
- [x] Rewrite `decode_token` in `appview-auth/src/lib.rs` to manually parse JWT payload
- [x] Update tests in `appview-auth/src/lib.rs` to cover ES256K-style tokens (arbitrary alg header)
- [x] Run `cargo check -p appview-auth` to verify compilation
- [x] Run `cargo test -p appview-auth` to verify tests pass
