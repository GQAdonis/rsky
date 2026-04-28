# Tasks: p3-c012

## 1. auth_verifier changes

- [ ] 1.1 Remove the `// @TODO: Implement DPop/OAuth` comment at `rsky-pds/src/auth_verifier.rs:788`
- [ ] 1.2 Add a token-type discriminator: legacy session JWT vs OAuth access token
- [ ] 1.3 For OAuth tokens: validate signature, expiry, audience, DPoP `cnf.jkt` binding
- [ ] 1.4 Resolve the token's scope claim into a `PermissionSet` via `rsky-oauth-scopes`
- [ ] 1.5 Attach the `PermissionSet` to the Rocket request guard so handlers can read it

## 2. pipethrough changes

- [ ] 2.1 In `rsky-pds/src/pipethrough.rs`, before forwarding a proxied request, call `RpcPermissionMatch::check(method_nsid, permission_set)`
- [ ] 2.2 On reject: return the appropriate OAuth error
- [ ] 2.3 On accept: forward unchanged

## 3. Per-handler updates

- [ ] 3.1 For each handler currently requiring `AuthScope::Access` or similar, ensure the OAuth-derived `PermissionSet` is also accepted
- [ ] 3.2 Add a transitional fallback so legacy app-password sessions still work

## 4. Tests

- [ ] 4.1 Unit test: OAuth token with read scope rejected on `createRecord`
- [ ] 4.2 Unit test: OAuth token with write scope accepted on `createRecord`
- [ ] 4.3 Unit test: DPoP-bound token without DPoP proof on request rejected
- [ ] 4.4 Unit test: legacy session JWT still works (transitional)
- [ ] 4.5 Integration test: full OAuth flow end-to-end against rsky-pds

## 5. Conformance harness extension

- [ ] 5.1 Extend p3-c008 harness with an OAuth scenario: PAR → authorize → token → createRecord
- [ ] 5.2 Compare flow outcomes against upstream

## 6. Verify

- [ ] 6.1 `cargo test --release -p rsky-pds` passes
- [ ] 6.2 Conformance harness OAuth scenario passes
- [ ] 6.3 Official Bluesky web client (or `@atproto/api ≥ 0.19.x`) successfully completes OAuth login + post-record against `pds.know-me.tools`
