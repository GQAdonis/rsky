# p3-c001: Document Postgres-only divergence

## Why

The user has locked storage on PostgreSQL for everything (account-manager, sequencer, did-cache, actor store), explicitly diverging from upstream `@atproto/pds@0.4.220` which defaults to SQLite. Operators following upstream's `installer.sh` would expect SQLite to work — and they will fail. Document the divergence prominently so nobody loses time hunting for SQLite support.

## What Changes

- README.md gains an "Operations / Storage" section that names PostgreSQL as the only supported database and explicitly calls out that the upstream `installer.sh` SQLite path does not apply.
- CLAUDE.md gains the same statement near the "Storage backends" line so AI agents working on the codebase don't propose SQLite work.
- `rsky-pds/README.md` (the crate's README) gets the same callout in its Setup section.

## Capabilities

### Modified Capabilities

- `pds-server`: adds the "Storage backend MUST be PostgreSQL for all PDS state" requirement and its scenario.

## Impact

- Documentation only — no code changes.
- Establishes the canonical reference operators and contributors should consult before proposing storage-related changes.
