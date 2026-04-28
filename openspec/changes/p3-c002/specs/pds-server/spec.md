# Spec Delta: p3-c002 — pds-server

## ADDED Requirements

### Requirement: PDS MUST support `did:web` accounts in addition to `did:plc`

Account creation, DID document validation, and PLC integration paths MUST handle `did:web` identifiers without panicking. `did:web` resolution MUST go through HTTP, not the PLC directory.

#### Scenario: did:web account create

- **WHEN** a user submits `com.atproto.server.createAccount` with a `did:web:example.com` DID
- **THEN** the request succeeds, the account is created, and no codepath returns `Not yet supporting did:web`

### Requirement: PDS MUST enforce upstream blob-upload size limits

Image blob uploads MUST allow up to 2 MB per upload (matching upstream 0.4.218). Video blob uploads MUST allow up to 100 MB per upload (matching upstream 0.4.105+).

#### Scenario: Image upload up to 2 MB

- **WHEN** an authenticated client uploads a 1.9 MB JPEG via `com.atproto.repo.uploadBlob`
- **THEN** the upload succeeds with a 200 response and a returned blob ref

#### Scenario: Video upload up to 100 MB

- **WHEN** an authenticated client uploads a 90 MB MP4 video blob via `com.atproto.repo.uploadBlob`
- **THEN** the upload succeeds with a 200 response

### Requirement: `getBlob` MUST set Content-Disposition for browser downloads

`com.atproto.sync.getBlob` MUST set a `Content-Disposition` header that triggers browser download behavior.

#### Scenario: Browser download

- **WHEN** a browser navigates to a `getBlob` URL
- **THEN** the response includes a `Content-Disposition: attachment; filename=...` header

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

### Requirement: PDS MUST NOT contain runtime `unimplemented!()` or `todo!()` panic surfaces in its handler tree

All `unimplemented!()` and `todo!()` macros under `rsky-pds/src/apis/` and `rsky-pds/src/db/` MUST be replaced with real implementations. CI MUST grep for these markers and fail if any remain.

#### Scenario: No runtime panic markers

- **WHEN** `grep -rn "todo!()\\|unimplemented!()" rsky-pds/src/` runs in CI
- **THEN** the grep returns zero hits
