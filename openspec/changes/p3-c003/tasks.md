# Tasks: p3-c003

## 1. Schema

- [ ] 1.1 Create Diesel migration `add_used_refresh_token` adding the table (id, jti UNIQUE, did, expires_at, created_at)
- [ ] 1.2 Add the table to `rsky-pds/src/schema.rs`
- [ ] 1.3 Index on `jti` and `expires_at`

## 2. Helper

- [ ] 2.1 Create `rsky-pds/src/account_manager/helpers/used_refresh_token.rs` with `insert(jti)`, `exists(jti) -> bool`, and `prune_expired()`
- [ ] 2.2 Wire the helper into the account_manager helpers module

## 3. Rotate-token logic

- [ ] 3.1 In `account_manager/mod.rs::rotate_refresh_token`, check `used_refresh_token::exists(jti)` before rotating
- [ ] 3.2 On hit: revoke the session lineage (delete all descendant refresh tokens / sessions sharing the lineage root) and return an authentication error
- [ ] 3.3 On miss: insert the JTI into `used_refresh_token`, then proceed with the rotation atomically (one DB transaction)

## 4. Tests

- [ ] 4.1 Unit test: rotate then re-present the same token → second call rejected and lineage revoked
- [ ] 4.2 Unit test: prune_expired removes rows past their expiry without touching active rows
- [ ] 4.3 Integration test: full createSession → refreshSession → refreshSession (replay) flow

## 5. Cleanup

- [ ] 5.1 Hook `prune_expired()` into a background task (runs hourly)
- [ ] 5.2 Document the new table and its retention policy in `rsky-pds/README.md`
