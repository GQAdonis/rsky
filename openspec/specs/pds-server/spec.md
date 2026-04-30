# Capability: pds-server

This capability defines what `rsky-pds` must do to function as a substitutable AT Protocol Personal Data Server on the public network — at parity with `@atproto/pds@0.4.220` (upstream HEAD `877e629`, 2026-04-24), with the explicit divergence that storage stays on PostgreSQL only.

## Purpose

`rsky-pds` is the user-facing repository server in the rsky stack. It speaks XRPC over HTTPS, hosts per-actor repositories backed by an MST, signs commits, emits the `subscribeRepos` firehose, proxies `app.bsky.*`, `chat.bsky.*`, and `tools.ozone.*` to the configured AppView and Ozone services, and integrates with the PLC directory for did:plc identity operations.

The reference implementation is `@atproto/pds`. We track its protocol behavior, not its storage shape: where upstream uses one SQLite file per actor and SQLite for shared services (account-manager, sequencer, did-cache), `rsky-pds` uses PostgreSQL for everything. Any requirement below that does not specify storage applies regardless of backend; requirements that name PostgreSQL explicitly capture the divergence.

## Requirements

### Requirement: Storage backend MUST be PostgreSQL for all PDS state

`rsky-pds` MUST use PostgreSQL for the account-manager database, sequencer database, did-cache database, and per-actor repo store. SQLite MUST NOT be a runtime dependency.

#### Scenario: Postgres-only operator install

- **WHEN** an operator deploys rsky-pds following this repo's documentation
- **THEN** the only database engine they need to provision is PostgreSQL, the README and CLAUDE.md state Postgres-only explicitly, and the upstream `installer.sh` SQLite path is documented as not applicable to this fork

### Requirement: PDS MUST emit sync v1.1 firehose events

The `subscribeRepos` firehose MUST emit `#commit` events with `prev` CIDs and covering proofs, MUST emit `#sync` events on account creation and on certain identity transitions, MUST emit `#identity` and `#account` events, and MUST treat `#handle` and `#tombstone` as deprecated.

#### Scenario: Strict relay accepts the firehose

- **WHEN** a relay configured to enforce sync v1.1 strictly subscribes to rsky-pds's `com.atproto.sync.subscribeRepos`
- **THEN** the relay does not reject any frame for missing `prev`, missing covering proof, or use of deprecated event types over a 24-hour soak

### Requirement: PDS MUST support `did:web` accounts in addition to `did:plc`

Account creation, DID document validation, and PLC integration paths MUST handle `did:web` identifiers without panicking. Where appropriate, `did:web` resolution MUST go through HTTP, not the PLC directory.

#### Scenario: did:web account create

- **WHEN** a user submits `com.atproto.server.createAccount` with a `did:web:example.com` DID
- **THEN** the request succeeds, the account is created, and no codepath returns `Not yet supporting did:web`

### Requirement: PDS MUST detect and reject reused refresh tokens

The PDS MUST persist used refresh tokens (in a `used_refresh_token`-equivalent table) and reject any refresh attempt that re-presents a token already rotated out. Detection of reuse MUST also invalidate the descendant token chain.

#### Scenario: Replay attempt fails

- **WHEN** a client presents a refresh token that has already been rotated (a copy from a stolen session)
- **THEN** the PDS rejects the refresh with an authentication error and revokes the corresponding session lineage

### Requirement: Sequencer MUST sequence concurrent writes to the same repo in a deterministic order

Concurrent writes against the same repository MUST receive monotonically increasing sequence numbers without out-of-order gaps. The recovery script MUST tolerate invalid `<nsid>/<rkey>` paths in stored commit operations and skip them rather than aborting.

#### Scenario: Concurrent applyWrites

- **WHEN** two `com.atproto.repo.applyWrites` requests for the same DID hit the PDS within milliseconds of each other
- **THEN** both succeed, both produce sequencer rows with strictly increasing `seq` values, and the firehose emits the events in `seq` order

### Requirement: PDS MUST track upstream lexicons within one minor version

The lexicon definitions in `rsky-lexicon` MUST be kept within one minor version of upstream `bluesky-social/atproto` lexicons. A documented refresh procedure MUST exist.

#### Scenario: Lexicon refresh cadence

- **WHEN** an operator inspects the rsky-lexicon README or maintenance docs
- **THEN** they find a documented procedure for syncing rsky-lexicon against upstream lexicons HEAD, including codegen invocation and validation steps

### Requirement: PDS MUST enforce upstream blob-upload size limits

Image blob uploads MUST allow up to 2 MB per upload (matching upstream 0.4.218). Video blob uploads MUST allow up to 100 MB per upload (matching upstream 0.4.105+).

#### Scenario: Image upload up to 2 MB

- **WHEN** an authenticated client uploads a 1.9 MB JPEG via `com.atproto.repo.uploadBlob`
- **THEN** the upload succeeds with a 200 response and a returned blob ref

### Requirement: `getBlob` MUST set Content-Disposition for browser downloads

`com.atproto.sync.getBlob` MUST set a `Content-Disposition` header that triggers browser download behavior, matching upstream 0.4.209.

#### Scenario: Browser download

- **WHEN** a browser navigates to a `getBlob` URL
- **THEN** the response includes a `Content-Disposition: attachment; filename=...` (or equivalent) header

### Requirement: Pipethrough MUST proxy `tools.ozone.*` and `chat.bsky.*` along with `app.bsky.*`

The XRPC catchall MUST forward any unmatched lexicon under `app.bsky.*`, `chat.bsky.*`, or `tools.ozone.*` to the configured AppView / Chat / Ozone host.

#### Scenario: Ozone moderator endpoint reachable through PDS

- **WHEN** a moderator client authenticated against rsky-pds calls `tools.ozone.moderation.queryStatuses`
- **THEN** the PDS forwards the request to the configured Ozone service and returns the response without erroring as `XRPCNotImplemented`

### Requirement: PDS MUST debounce `requestCrawl` to its configured relays

The PDS MUST NOT call `com.atproto.sync.requestCrawl` against the same relay multiple times concurrently. Concurrent attempts MUST coalesce.

#### Scenario: Concurrent requestCrawl coalesces

- **WHEN** two unrelated server events both trigger `requestCrawl` against the same relay within seconds of each other
- **THEN** at most one outbound `requestCrawl` request reaches the relay during that window

### Requirement: Per-actor data MUST be strictly isolated in the Postgres actor store

Every query against the actor store MUST scope rows by `did` (or operate inside a per-DID schema). Cross-actor reads MUST require an explicit administrative codepath. Per-DID export MUST produce a CAR byte-identical to what an upstream PDS would produce for the same writes.

#### Scenario: Per-actor export

- **WHEN** the same canonical write sequence is applied to rsky-pds and to upstream `@atproto/pds@0.4.220`, then `com.atproto.sync.getRepo` is called for the actor's DID on each
- **THEN** both PDSes return the same CAR bytes (commit CID and MST root identical)

#### Scenario: No accidental cross-actor read

- **WHEN** a normal request handler queries a record by URI for actor A
- **THEN** the SQL executed includes a `WHERE did = $A` predicate (or runs inside actor A's per-DID schema), and the query plan cannot return any row owned by actor B

### Requirement: PDS MUST support OAuth as a first-class authentication path

The PDS MUST accept OAuth-issued, DPoP-bound access tokens at `auth-verifier`. It MUST expose `/.well-known/oauth-protected-resource`, the OAuth provider routes (PAR, authorize, token, JWKS, client metadata), and MUST persist the supporting account-manager schema (device, account-device, authorized-client, authorization-request, used-refresh-token).

#### Scenario: Bluesky web client OAuth login

- **WHEN** the official Bluesky web client (or any modern `@atproto/api ≥ 0.19.x` client) initiates an OAuth login against rsky-pds
- **THEN** PAR succeeds, authorize redirects through the user-consent step, the token endpoint issues a DPoP-bound access token plus a rotating refresh token, and the client can subsequently call `com.atproto.repo.createRecord` using that token

### Requirement: OAuth scope enforcement MUST honor lexicon-aware permissions

The OAuth path MUST evaluate scopes via a Rust port of `@atproto/oauth-scopes` semantics: `RpcPermissionMatch`, `ScopePermissions`, and `PermissionSet` MUST be used by `auth_verifier.rs` and `pipethrough.rs` to gate per-method access.

#### Scenario: Token without write scope is rejected on createRecord

- **WHEN** a client presents an OAuth token whose scope grants only read access and calls `com.atproto.repo.createRecord`
- **THEN** the PDS rejects the request with an authentication / authorization error and does not perform the write

### Requirement: Federation conformance harness MUST exist and MUST pass

The repository MUST include a conformance harness that runs rsky-pds and `@atproto/pds@0.4.220` side by side, applies a canonical write workload to each, and diffs (a) the firehose frames, (b) the `getRepo` CAR bytes, and (c) the per-record `getRecord` payloads. The harness MUST be runnable from CI.

#### Scenario: Side-by-side conformance pass

- **WHEN** the conformance harness runs the canonical workload on a fresh checkout
- **THEN** it reports zero diffs on lexicon-defined fields of `subscribeRepos` frames, byte-identical CAR output for `getRepo`, and byte-identical record payloads for `getRecord` across the two PDS implementations
