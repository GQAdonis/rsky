# Spec Delta: p3-c001 — pds-server

## ADDED Requirements

### Requirement: Storage backend MUST be PostgreSQL for all PDS state

`rsky-pds` MUST use PostgreSQL for the account-manager database, sequencer database, did-cache database, and per-actor repo store. SQLite MUST NOT be a runtime dependency. The fork's documentation MUST state this divergence explicitly.

#### Scenario: Postgres-only operator install

- **WHEN** an operator deploys rsky-pds following this repo's documentation
- **THEN** the only database engine they need to provision is PostgreSQL, and the README, CLAUDE.md, and `rsky-pds/README.md` all state Postgres-only explicitly
