# Spec Delta: p2-c006 — kubernetes-deployment

## ADDED Requirements

### Requirement: Cloudflare DNS MUST resolve all `*.know-me.tools` service hostnames to the cluster ingress

Cloudflare DNS records for `pds.know-me.tools`, `relay.know-me.tools`, `feed.know-me.tools`, and `social.know-me.tools` MUST be configured to resolve to the cluster's external load balancer / Envoy Gateway address.

#### Scenario: External resolution

- **WHEN** an external client runs `dig +short pds.know-me.tools` (and similarly for the other three hostnames)
- **THEN** the response resolves to the cluster's ingress address, and `curl -I https://<host>/` over TLS succeeds with a valid Let's Encrypt certificate
