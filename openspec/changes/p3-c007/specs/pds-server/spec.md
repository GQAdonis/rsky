# Spec Delta: p3-c007 — pds-server

## ADDED Requirements

### Requirement: Per-actor data MUST be strictly isolated in the Postgres actor store

Every query against the actor store MUST scope rows by `did` (or operate inside a per-DID schema). Cross-actor reads MUST require an explicit administrative codepath. Postgres Row-Level Security MUST enforce this as a defense-in-depth backstop against missing predicates.

#### Scenario: Per-actor export byte-equivalence

- **WHEN** the same canonical write sequence is applied to rsky-pds and to upstream `@atproto/pds@0.4.220`, then `com.atproto.sync.getRepo` is called for the actor's DID on each
- **THEN** both PDSes return the same CAR bytes (commit CID and MST root identical)

#### Scenario: No accidental cross-actor read

- **WHEN** a normal authenticated request handler queries a record by URI for actor A but, due to a bug, omits the `WHERE did = $A` predicate
- **THEN** the Postgres RLS policy denies the query and the handler surfaces an error rather than returning actor B's row
