# Spec Delta: p3-c009 — pds-server

## ADDED Requirements

### Requirement: PDS MUST support OAuth as a first-class authentication path

The PDS MUST accept OAuth-issued, DPoP-bound access tokens at its auth verifier. It MUST expose `/.well-known/oauth-authorization-server`, `/.well-known/oauth-protected-resource`, `/oauth/par`, `/oauth/authorize`, `/oauth/token`, `/oauth/jwks`, and `/oauth/client-metadata` per the ATProto OAuth profile, and MUST enforce DPoP proof validation on every token issuance and access-token use.

#### Scenario: Bluesky web client OAuth login

- **WHEN** a modern `@atproto/api ≥ 0.19.x` client initiates an OAuth login against rsky-pds
- **THEN** PAR succeeds, authorize redirects through the user-consent step, the token endpoint issues a DPoP-bound access token plus a rotating refresh token, and the client can subsequently call `com.atproto.repo.createRecord` using that token

#### Scenario: DPoP replay rejected

- **WHEN** an attacker captures a valid DPoP proof and presents it a second time on a different request
- **THEN** the PDS rejects the request because the DPoP `jti` is in the replay cache
