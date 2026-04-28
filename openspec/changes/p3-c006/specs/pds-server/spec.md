# Spec Delta: p3-c006 — pds-server

## ADDED Requirements

### Requirement: PDS MUST emit sync v1.1 firehose events

The `subscribeRepos` firehose MUST emit `#commit` events with `prev` CIDs and covering proofs, MUST emit `#sync` events on account creation and on certain identity transitions, MUST emit `#identity` and `#account` events, and MAY continue to emit `#handle` and `#tombstone` for back-compat as upstream does.

#### Scenario: Strict relay accepts the firehose

- **WHEN** a relay configured to enforce sync v1.1 strictly subscribes to rsky-pds's `com.atproto.sync.subscribeRepos`
- **THEN** the relay does not reject any frame for missing `prev`, missing covering proof, or use of unknown event types over a 24-hour soak

#### Scenario: Sync event on account creation

- **WHEN** a new account is created via `com.atproto.server.createAccount`
- **THEN** the firehose emits a `#sync` event for the new DID before any subsequent `#commit` events
