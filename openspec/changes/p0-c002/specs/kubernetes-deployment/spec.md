# Spec Delta: p0-c002 — kubernetes-deployment

## ADDED Requirements

### Requirement: PostgreSQL MUST run as a StatefulSet with pgvector available

Persistent Postgres MUST be deployed as a `StatefulSet` with a `PersistentVolumeClaim` template. The image (or an `initdb` step) MUST make the `vector` extension available so rsky services that depend on pgvector can `CREATE EXTENSION IF NOT EXISTS vector` without operator intervention.

#### Scenario: Postgres pod restart preserves data

- **WHEN** the Postgres pod is deleted and the StatefulSet recreates it
- **THEN** the new pod re-attaches to its existing PVC and previous database contents are intact

#### Scenario: pgvector extension available

- **WHEN** an rsky service connects to Postgres for the first time and runs `CREATE EXTENSION IF NOT EXISTS vector`
- **THEN** the statement succeeds without manual installation of the pgvector binary
