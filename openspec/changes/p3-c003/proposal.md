# p3-c003: used-refresh-token replay defense

## Why

`rsky-pds` rotates refresh tokens (`account_manager/mod.rs:256` `rotate_refresh_token`) but does not persist used tokens. A leaked refresh token can be replayed against the rotation endpoint until the legitimate session also rotates and silently mints a parallel one. Upstream `@atproto/pds` has a `used-refresh-token` table (added pre-0.4.107) that detects reuse and revokes the descendant session lineage. We need the same defense.

## What Changes

- New Diesel migration adding `used_refresh_token` table keyed by token JTI, with TTL/expiry index.
- New helper `account_manager/helpers/used_refresh_token.rs` mirroring the upstream helper surface (insert, exists, prune-expired).
- Modify `rotate_refresh_token` to: (1) check the incoming token's JTI against `used_refresh_token`; (2) on hit, refuse the rotation and revoke the entire session chain that derived from the same root; (3) on miss, insert the JTI into `used_refresh_token` before issuing the new token.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "PDS MUST detect and reject reused refresh tokens" requirement.

## Impact

- One Diesel migration in `rsky-pds/migrations/`.
- Schema entry in `rsky-pds/src/schema.rs`.
- New helper file under `rsky-pds/src/account_manager/helpers/`.
- Edits to `rsky-pds/src/account_manager/mod.rs` and the refresh-session handler.
- Background job (or transactional cleanup at insert time) to prune expired rows.
