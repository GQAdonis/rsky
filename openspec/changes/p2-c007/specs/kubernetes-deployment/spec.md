# Spec Delta: p2-c007 — kubernetes-deployment

## ADDED Requirements

### Requirement: Post-deploy smoke tests MUST pass for every public endpoint

After the initial deploy completes, a smoke test MUST exercise each public endpoint and confirm 2xx responses on the documented health and protocol endpoints. The smoke test MUST be runnable both manually and as a workflow step on subsequent deploys.

#### Scenario: PDS health and describeServer

- **WHEN** the post-deploy smoke test runs against `https://pds.know-me.tools`
- **THEN** `GET /xrpc/_health` returns 200 and `GET /xrpc/com.atproto.server.describeServer` returns a JSON body containing `availableUserDomains`

#### Scenario: Relay reachable

- **WHEN** the smoke test connects to `wss://relay.know-me.tools/xrpc/com.atproto.sync.subscribeRepos`
- **THEN** the WebSocket handshake succeeds and at least one frame is received within 30 seconds

#### Scenario: Web client home page

- **WHEN** the smoke test runs `curl -I https://social.know-me.tools/`
- **THEN** the response is 200 with a valid Let's Encrypt TLS certificate
