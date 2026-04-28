# p3-c009: OAuth provider core — PAR / authorize / token / JWKS / DPoP

## Why

Modern Bluesky clients use OAuth for sign-in (added upstream as `@atproto/oauth-provider`, currently at 0.16.1, with PAR + DPoP-bound tokens + rotating refresh tokens + scope grants). `rsky-pds` has zero OAuth surface — only password-based `createSession` + app-passwords. Without OAuth, modern `@atproto/api ≥ 0.19.x` clients cannot authenticate. This is the dominant blocker on the parity list.

This change lands the OAuth provider's *core HTTP surface* in Rust: PAR, authorize, token, JWKS, client metadata, and DPoP enforcement. It depends on p3-c010 (`oauth-scopes` Rust port) for scope semantics and p3-c011 (account-manager schema) for persistence.

## What Changes

- New module `rsky-pds/src/oauth_provider/` with submodules:
  - `routes.rs` — Rocket routes for `/.well-known/oauth-authorization-server`, `/.well-known/oauth-protected-resource`, `/oauth/par`, `/oauth/authorize`, `/oauth/token`, `/oauth/jwks`, `/oauth/client-metadata`.
  - `dpop.rs` — DPoP proof verification (JWT validation, jti replay cache, htu/htm match).
  - `client.rs` — Client metadata fetch + validation (per ATProto profile).
  - `flow.rs` — Authorization-flow state machine (PAR → consent → code → token).
  - `errors.rs` — OAuth error envelope helpers.
- Wire the routes into `rsky-pds/src/lib.rs` Rocket assembly.
- Add `/.well-known/oauth-protected-resource` metadata response per ATProto OAuth profile.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "PDS MUST support OAuth as a first-class authentication path" requirement.
- `kubernetes-deployment`: minor — the rsky-pds `Deployment` env vars and Gateway routes need to expose the new endpoints.

## Impact

- Largest single change in the phase (XL effort).
- Depends on p3-c010 (scopes) and p3-c011 (account-manager schema) — those should land first.
- Verified by p3-c008 (federation conformance harness) extended with OAuth flow tests.
- New crate-level deps likely: `jsonwebtoken` (or `jose`-equivalent), DPoP-specific jwk handling. Per C-005, deps need discussion before adding — flag for review.
