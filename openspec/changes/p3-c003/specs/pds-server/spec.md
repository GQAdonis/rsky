# Spec Delta: p3-c003 — pds-server

## ADDED Requirements

### Requirement: PDS MUST detect and reject reused refresh tokens

The PDS MUST persist used refresh tokens (in a `used_refresh_token` table keyed by JTI) and reject any refresh attempt that re-presents a token already rotated out. Detection of reuse MUST also revoke the descendant token chain.

#### Scenario: Replay attempt fails

- **WHEN** a client presents a refresh token that has already been rotated (a stolen copy)
- **THEN** the PDS rejects the refresh with an authentication error and revokes the corresponding session lineage so the legitimate parallel session is also forced to re-authenticate

#### Scenario: Expired entries are pruned

- **WHEN** the periodic prune job runs against the `used_refresh_token` table
- **THEN** rows whose `expires_at` is in the past are removed and rows whose tokens are still active remain
