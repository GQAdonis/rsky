# Tasks: p3-c007

## 1. Audit existing SQL

- [ ] 1.1 List every Diesel `query!` and `sql!` invocation under `rsky-pds/src/actor_store/`
- [ ] 1.2 Confirm each non-admin invocation includes a `WHERE did = $X` predicate (or scopes via a per-DID schema)
- [ ] 1.3 Document remaining admin/cross-actor codepaths and add NOTE comments naming them as such

## 2. Add Row-Level Security

- [ ] 2.1 Diesel migration that enables RLS on each actor-store table and adds a policy `did = current_setting('app.current_did')`
- [ ] 2.2 Add a privileged role bypass for admin codepaths (`SET LOCAL app.is_admin = on` inside admin transactions)

## 3. Per-request GUC plumbing

- [ ] 3.1 In the request guard / Rocket fairing for authenticated handlers, set `SET LOCAL app.current_did = $did` at the start of each transaction
- [ ] 3.2 Ensure the GUC scope is the transaction (LOCAL), not the session

## 4. Regression test

- [ ] 4.1 Test: handler authenticated as actor A, attempts SQL referencing actor B's records → RLS error (or zero rows)
- [ ] 4.2 Test: explicit admin codepath setting `app.is_admin = on` can read across actors

## 5. CAR byte-equivalence smoke check

- [ ] 5.1 Apply the canonical write sequence to rsky-pds; capture `getRepo` CAR
- [ ] 5.2 Apply the same sequence to upstream `@atproto/pds@0.4.220`; capture `getRepo` CAR
- [ ] 5.3 Diff — must match (full check delegated to p3-c008)

## 6. Verify

- [ ] 6.1 `cargo test --release -p rsky-pds` passes
- [ ] 6.2 Benchmark a representative handler to confirm RLS overhead is acceptable
