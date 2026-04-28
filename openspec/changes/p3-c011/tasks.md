# Tasks: p3-c011

## 1. Migrations

- [ ] 1.1 `device` table — id, jwk, ip, ua, last_seen_at, created_at
- [ ] 1.2 `account_device` table — did, device_id, granted_at, revoked_at (nullable)
- [ ] 1.3 `authorized_client` table — did, client_id, scope text, last_used_at, granted_at
- [ ] 1.4 `authorization_request` table — request_uri (PK), client_id, state, code_challenge, code_challenge_method, scope, redirect_uri, did (nullable until consent), expires_at

## 2. Schema bindings

- [ ] 2.1 Add the four tables to `rsky-pds/src/schema.rs`

## 3. Helper modules

- [ ] 3.1 `account_manager/helpers/device.rs` — register, lookup, update_last_seen, prune_stale
- [ ] 3.2 `account_manager/helpers/account_device.rs` — grant, revoke, list
- [ ] 3.3 `account_manager/helpers/authorized_client.rs` — grant, lookup, revoke, list
- [ ] 3.4 `account_manager/helpers/authorization_request.rs` — insert, consume, lookup_by_uri, prune_expired
- [ ] 3.5 `account_manager/helpers/scope_reference_getter.rs` — port of upstream's scope-reference-getter for resolving short-form scope refs to full ScopePermissions

## 4. Tests

- [ ] 4.1 CRUD round-trip per helper
- [ ] 4.2 Expiry behavior for authorization_request and stale device pruning

## 5. Verify

- [ ] 5.1 `cargo test --release -p rsky-pds` passes
- [ ] 5.2 Migrations apply cleanly against an empty Postgres
