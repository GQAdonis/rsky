# p3-c011: Account-manager OAuth schema

## Why

Upstream `@atproto/pds@0.4.220` ships five OAuth-specific account-manager helpers and tables that don't exist in rsky: `device.ts`, `account-device.ts`, `authorized-client.ts`, `authorization-request.ts`, `scope-reference-getter.ts`. The OAuth provider routes from p3-c009 need this schema to persist authorization requests, device-bound sessions, and per-client authorizations. Build the schema as its own change so it lands cleanly without entangling with the route work.

## What Changes

- Diesel migrations adding tables:
  - `device` (deviceId, jwk, last_seen_at, …)
  - `account_device` (did, deviceId, granted_at)
  - `authorized_client` (did, client_id, scope, …)
  - `authorization_request` (request_uri, client_id, state, code_challenge, …)
- `rsky-pds/src/account_manager/helpers/` gets new files mirroring the upstream helper surface: `device.rs`, `account_device.rs`, `authorized_client.rs`, `authorization_request.rs`, `scope_reference_getter.rs`.
- Schema entries in `rsky-pds/src/schema.rs`.
- The helpers expose CRUD + lookup APIs needed by p3-c009 (PAR persistence) and p3-c012 (verifier).

## Capabilities

### Modified Capabilities

- `pds-server`: contributes to the "PDS MUST support OAuth as a first-class authentication path" requirement (storage layer).

## Impact

- Diesel migrations on the account-manager Postgres database.
- Five new helper modules.
- No public route changes (those land in p3-c009).
- Prerequisite for p3-c009.
