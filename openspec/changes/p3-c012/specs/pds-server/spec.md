# Spec Delta: p3-c012 — pds-server

## ADDED Requirements

### Requirement: OAuth scope enforcement MUST honor lexicon-aware permissions

The OAuth path MUST evaluate scopes via the `rsky-oauth-scopes` crate. `auth_verifier.rs` MUST resolve an OAuth token's scope claim into a `PermissionSet` and expose it on the request guard. `pipethrough.rs` MUST call `RpcPermissionMatch::check(method_nsid, permission_set)` and reject proxied requests whose scope does not cover the method.

#### Scenario: Token without write scope rejected on createRecord

- **WHEN** a client presents an OAuth access token whose scope grants only read access and calls `com.atproto.repo.createRecord`
- **THEN** the PDS rejects the request with an OAuth `insufficient_scope` error and does not perform the write

#### Scenario: DPoP-bound token used without DPoP proof

- **WHEN** an OAuth access token bound to DPoP key `K` is presented on a request that does not include a matching DPoP proof header
- **THEN** the PDS rejects the request with an OAuth authentication error

#### Scenario: Legacy session JWT still accepted (transitional)

- **WHEN** a client presents a legacy session JWT issued by `createSession` against a handler that previously accepted it
- **THEN** the handler still accepts the token until legacy auth is fully retired in a later phase
