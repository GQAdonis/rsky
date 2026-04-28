# p3-c010: oauth-scopes Rust port

## Why

Upstream `@atproto/oauth-scopes` is a separate package (referenced from `auth-verifier.ts:17`, `pipethrough.ts:11`, `auth-output.ts:1` in `@atproto/pds@0.4.220`). It encodes the granular scope grammar used to gate per-method access: `RpcPermissionMatch`, `ScopePermissions`, `PermissionSet`, lexicon-aware checks against an XRPC method's scope tag. The OAuth provider work in p3-c009 has nothing to evaluate scopes against without this. Building it as a dedicated crate makes it reusable by `rsky-relay` and `rsky-feedgen` later.

## What Changes

- New crate `rsky-oauth-scopes` in the workspace, mirroring the public surface of `@atproto/oauth-scopes`:
  - `Scope` parser (string ↔ structured)
  - `PermissionSet` (the union of scopes a token grants)
  - `ScopePermissions` (per-resource-type permissions: profile, repo, blob, …)
  - `RpcPermissionMatch` (XRPC method → required scope check)
- Unit tests covering the upstream test fixtures (port them).

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "OAuth scope enforcement MUST honor lexicon-aware permissions" requirement (the wiring lands in p3-c012; this change provides the engine).

## Impact

- New workspace crate.
- Per C-005, this is a new crate-level dep but contained within the workspace; document the rationale (parity with upstream OAuth scope grammar).
- Consumed by p3-c012; built independently so it can be tested in isolation.
