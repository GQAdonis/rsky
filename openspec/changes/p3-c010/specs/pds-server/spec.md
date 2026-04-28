# Spec Delta: p3-c010 — pds-server

## ADDED Requirements

### Requirement: OAuth scope grammar MUST be implemented as a workspace crate

A workspace crate `rsky-oauth-scopes` MUST exist that ports the public surface of upstream `@atproto/oauth-scopes`: `Scope` (parse / serialize), `PermissionSet`, `ScopePermissions`, `RpcPermissionMatch`. The crate MUST be consumable by `rsky-pds` and other rsky services without leaking PDS-specific assumptions.

#### Scenario: Round-trip parse

- **WHEN** an arbitrary upstream scope string is parsed into a `Scope` and serialized back
- **THEN** the output string is structurally equivalent to the input (syntactic equivalence per upstream rules)

#### Scenario: RPC method scope check

- **WHEN** `RpcPermissionMatch::check("com.atproto.repo.createRecord", &permission_set)` is called with a `permission_set` that grants only read access
- **THEN** the result is `false`, indicating the call must be rejected
