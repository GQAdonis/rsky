# Storage Architecture — rsky-pds

## Decision: Postgres-only (locked 2026-04-28)

rsky-pds uses **PostgreSQL for all persistent state**. This is a permanent, intentional divergence from the upstream `@atproto/pds` TypeScript reference implementation, which defaults to SQLite for everything.

## What upstream does vs. what rsky-pds does

| Component | Upstream `@atproto/pds` | rsky-pds |
|---|---|---|
| Account-manager DB | SQLite (`account.sqlite`) | PostgreSQL (`rsky` database) |
| Sequencer DB | SQLite (shared with account-manager) | PostgreSQL (`rsky` database) |
| DID cache DB | SQLite (`did-cache.sqlite`) | PostgreSQL (`rsky` database) |
| Per-actor store | One SQLite file per DID (`actors/<did>.sqlite`) | Shared PostgreSQL tables with `actor_did` discriminator column + Row-Level Security |
| Blob storage | On-disk under `$DATA_DIR/blobs/` | S3-compatible object storage (GCS, AWS S3, Tigris, etc.) |

## Why PostgreSQL

1. **Operational simplicity at scale.** A single PostgreSQL instance is easier to back up, replicate, and migrate than dozens or hundreds of SQLite files spread across a container filesystem.
2. **Cloud-native deployment.** PostgreSQL integrates with managed database offerings (Cloud SQL, RDS, Supabase, Neon). SQLite does not.
3. **Concurrent write safety.** PostgreSQL's MVCC concurrency model handles concurrent writes to the same DID's repo without the single-writer limitation of WAL-mode SQLite.
4. **Row-Level Security.** Per-DID isolation is enforced at the database layer via Postgres RLS policies, not via separate files. This makes actor store access auditable and enforceable without application-level discipline.

## What this does NOT mean

- Protocol-level behavior is identical. The firehose events, `getRepo` CAR bytes, XRPC response shapes, and DID documents produced by rsky-pds must match upstream bit-for-bit on the wire. Storage internals never leak into protocol output.
- You can still federate with the public Bluesky network. The federation conformance harness (`k8s/conformance/`) verifies this side-by-side against `@atproto/pds@0.4.220`.

## Operator notes

- Do not use the upstream `installer.sh` or `@atproto/pds` Docker image with this codebase. They assume SQLite.
- Provide a PostgreSQL connection string via `DATABASE_URL`. See `k8s/rsky-pds/secret.yaml` for the expected env var pattern.
- Provide S3-compatible credentials via `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_BUCKET`, and `AWS_ENDPOINT_URL`.
- The PostgreSQL init script at `k8s/postgresql/initdb-configmap.yaml` creates the `rsky` and `rsky_feedgen` databases and installs the `vector` extension.
