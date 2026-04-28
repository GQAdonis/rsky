# p3-c012: Wire OAuth into auth_verifier and pipethrough

## Why

p3-c009 lands the OAuth provider routes; p3-c010 lands the scope grammar; p3-c011 lands the persistence schema. None of that gates a single XRPC handler until `rsky-pds/src/auth_verifier.rs` and `rsky-pds/src/pipethrough.rs` are taught to recognize OAuth-issued, DPoP-bound access tokens and consult `rsky-oauth-scopes` for per-method authorization. This change closes the loop: existing handlers automatically accept OAuth tokens with correct scope coverage and reject those without.

## What Changes

- `rsky-pds/src/auth_verifier.rs`:
  - Replace the current `// @TODO: Implement DPop/OAuth` comment at line 788 with real DPoP-bound bearer-token recognition.
  - Validate the access-token JWT signature, expiry, audience, and DPoP `cnf.jkt` binding.
  - Resolve the token's scope set into a `PermissionSet` via `rsky-oauth-scopes`.
  - Expose the `PermissionSet` on the request guard so handlers can reach it.
- `rsky-pds/src/pipethrough.rs`:
  - For each proxied request, call `RpcPermissionMatch::check(method_nsid, permission_set)`.
  - Reject with the appropriate OAuth error if the token's scope does not cover the method.
  - Forward the request only if scope check passes.
- Update the per-method handlers that currently hard-require legacy auth to accept either legacy or OAuth (transitional).

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "OAuth scope enforcement MUST honor lexicon-aware permissions" requirement and reinforces the existing OAuth requirement from p3-c009.

## Impact

- Touches the most security-sensitive file in the PDS (`auth_verifier.rs`) — careful review required.
- Depends on p3-c009 (OAuth routes), p3-c010 (scopes), p3-c011 (account-manager schema).
- Verified by p3-c008 conformance harness extended with OAuth scope tests.
- Marks the close of the OAuth subphase.
