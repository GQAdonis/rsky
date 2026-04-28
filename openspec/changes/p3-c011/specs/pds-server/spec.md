# Spec Delta: p3-c011 — pds-server

## ADDED Requirements

### Requirement: Account-manager MUST persist OAuth-supporting state

The PDS account-manager Postgres database MUST contain tables and helpers for: devices (`device`), per-account device grants (`account_device`), per-client authorizations (`authorized_client`), and pending authorization requests (`authorization_request`). Helpers for each MUST exist in `rsky-pds/src/account_manager/helpers/`.

#### Scenario: Authorization request persistence

- **WHEN** the OAuth provider receives a Pushed Authorization Request and persists it
- **THEN** an `authorization_request` row exists keyed by `request_uri`, with `client_id`, `code_challenge`, `state`, and `scope` populated, and the row is removed (consumed) on successful code exchange

#### Scenario: Device-bound session

- **WHEN** a user grants an OAuth client access from a device for the first time
- **THEN** a `device` row is created (or updated), an `account_device` row links the user's DID to the device, and an `authorized_client` row records the per-client scope grant
