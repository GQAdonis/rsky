# Tasks: p3-c009

## 1. Module skeleton

- [ ] 1.1 Create `rsky-pds/src/oauth_provider/` with `mod.rs`, `routes.rs`, `dpop.rs`, `client.rs`, `flow.rs`, `errors.rs`
- [ ] 1.2 Wire the module into `rsky-pds/src/lib.rs` Rocket route assembly

## 2. Well-known metadata endpoints

- [ ] 2.1 `GET /.well-known/oauth-authorization-server` returns the AS metadata document
- [ ] 2.2 `GET /.well-known/oauth-protected-resource` returns the resource metadata document with `resource`, `authorization_servers`, `bearer_methods_supported: ["header"]`
- [ ] 2.3 `GET /oauth/jwks` returns the AS signing key JWK set
- [ ] 2.4 `GET /oauth/client-metadata?client_id=...` validates and returns client metadata per ATProto profile

## 3. PAR + authorize

- [ ] 3.1 `POST /oauth/par` accepts a Pushed Authorization Request, validates client + DPoP, persists the authz request (via p3-c011 schema), returns request_uri
- [ ] 3.2 `GET /oauth/authorize?request_uri=...` renders the consent UI (or redirects through it for OOB clients)
- [ ] 3.3 Consent submission persists the user grant + issues an authorization code

## 4. Token endpoint

- [ ] 4.1 `POST /oauth/token` validates DPoP proof, exchanges code for DPoP-bound access token + rotating refresh token
- [ ] 4.2 Refresh-token grant: rotate, check used-refresh-token (p3-c003), issue new pair
- [ ] 4.3 Issued tokens carry the granted scope set (per p3-c010)

## 5. DPoP enforcement

- [ ] 5.1 In `dpop.rs`: verify DPoP JWT signature, check `htu` matches the request URL, check `htm` matches the HTTP method, check `jti` against a replay cache (Redis or in-memory with TTL)
- [ ] 5.2 Bind the access token's `cnf.jkt` to the DPoP key thumbprint
- [ ] 5.3 Reject any access-token usage not accompanied by a matching DPoP proof

## 6. Tests

- [ ] 6.1 Unit tests for each submodule
- [ ] 6.2 End-to-end test: full PAR → authorize → consent → token flow against a mock client
- [ ] 6.3 DPoP replay test: same DPoP `jti` rejected on second presentation

## 7. Verify

- [ ] 7.1 `cargo test --release -p rsky-pds` passes
- [ ] 7.2 The conformance harness (p3-c008) is extended with an OAuth login scenario that succeeds
- [ ] 7.3 The official Bluesky web client (or a stand-in `@atproto/api ≥ 0.19.x` client) completes an OAuth login against rsky-pds and posts a record
