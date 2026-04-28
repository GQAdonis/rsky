# p3-c007: actor_store per-DID isolation hardening on Postgres

## Why

Upstream `@atproto/pds` gets per-actor isolation for free via one SQLite database per DID. Under our locked Postgres-only model, isolation must be enforced explicitly at the SQL layer. Today `rsky-pds/src/actor_store/repo/sql_repo.rs` uses DID discriminators in shared tables, which is correct in spirit but easy to break: a forgotten `WHERE did = $1` predicate, a `JOIN` that drops the discriminator, or an admin codepath that intentionally crosses actors but leaks into a normal handler — any of these cause cross-actor reads. We need a defense-in-depth pass.

## What Changes

- Audit every SQL statement under `rsky-pds/src/actor_store/` and confirm each one either (a) has a `WHERE did = $X` predicate, or (b) is an explicitly-named admin codepath.
- Add Postgres Row-Level Security (RLS) policies on each actor-store table requiring `current_setting('app.current_did')` to match the row's `did`. Set the variable per request inside the same transaction that runs the actor-store SQL.
- Add a regression test that runs a normal handler request for actor A and confirms the underlying SQL plan cannot return rows owned by actor B (use `EXPLAIN` or RLS-violation error).
- Confirm that `com.atproto.sync.getRepo` for a given DID produces CAR bytes byte-identical to upstream (used as a fixture by p3-c008's harness).

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "Per-actor data MUST be strictly isolated in the Postgres actor store" requirement.

## Impact

- Diesel migration adding RLS policies on actor-store tables.
- New per-request middleware that sets `app.current_did` GUC at the start of each authenticated request handler.
- Audit pass through `rsky-pds/src/actor_store/`.
- Performance impact of RLS is generally low for B-tree-indexed `did` columns; benchmark before claiming victory.
