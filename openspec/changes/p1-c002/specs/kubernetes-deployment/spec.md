# Spec Delta: p1-c002 — kubernetes-deployment

## ADDED Requirements

### Requirement: Postgres initdb MUST create the `rsky_feedgen` database

The Postgres `initdb` ConfigMap (or equivalent) MUST create the `rsky_feedgen` database on first boot, alongside any other rsky databases declared by the deployment.

#### Scenario: Fresh Postgres bootstrap

- **WHEN** Postgres starts for the first time in a fresh cluster
- **THEN** `psql -c "\\l"` lists `rsky_feedgen` and `rsky-feedgen` can connect using its configured credentials without the operator running any manual `CREATE DATABASE` step
