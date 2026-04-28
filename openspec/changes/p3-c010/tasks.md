# Tasks: p3-c010

## 1. Crate scaffold

- [ ] 1.1 Add `rsky-oauth-scopes` to the Cargo workspace
- [ ] 1.2 `lib.rs` with `Scope`, `PermissionSet`, `ScopePermissions`, `RpcPermissionMatch` types

## 2. Scope grammar

- [ ] 2.1 Implement `Scope::parse` and `Scope::to_string` matching upstream's syntax
- [ ] 2.2 Implement `PermissionSet::from_scopes(&[Scope])`
- [ ] 2.3 Implement `ScopePermissions` with per-resource accessors

## 3. RPC matching

- [ ] 3.1 Implement `RpcPermissionMatch::check(method_nsid, permission_set) -> bool` using upstream's scope-tag table
- [ ] 3.2 Vendor the upstream scope-tag map for the lexicons we ship (after p3-c005 lexicon refresh)

## 4. Tests

- [ ] 4.1 Port the upstream `@atproto/oauth-scopes` test fixtures
- [ ] 4.2 Cover edge cases: empty scopes, malformed scopes, unknown methods, admin scopes

## 5. Verify

- [ ] 5.1 `cargo test --release -p rsky-oauth-scopes` passes
- [ ] 5.2 `cargo check --workspace` still passes
