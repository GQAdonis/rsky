# p1-c004: rsky-feedgen Kubernetes Manifests

**Phase**: phase-1-relay-feedgen
**Priority**: 4 (depends on p1-c002 for database)
**Assigned to**: claude-code

## Decision Summary

- **Kind**: Deployment (stateless — PostgreSQL is the state)
- **Replicas**: 1 (scale up after initial validation)
- **Image**: `ghcr.io/know-me-tools/rsky-feedgen:IMAGE_TAG`
- **Port**: 3000 (Rocket)
- **Storage**: PostgreSQL only — no PVC needed
- **Domain**: `feed.know-me.tools`
- **TLS**: cert-manager ClusterIssuer `letsencrypt`
- **Gateway**: `gatewayClassName: eg`

## Files to Create

```
k8s/rsky-feedgen/
├── configmap.yaml
├── secret.yaml          # envsubst template
├── deployment.yaml
├── service.yaml
├── certificate.yaml
├── gateway.yaml
├── httproute-https.yaml
└── httproute-redirect.yaml
```

## Key Configuration

### ConfigMap (non-secret)
```
FEEDGEN_SERVICE_DID=did:web:feed.know-me.tools
FEEDGEN_HOSTNAME=feed.know-me.tools
SHOW_SPONSORED_POST=0
TRENDING_PERCENTILE=0.9
ROCKET_PORT=3000
ROCKET_ADDRESS=0.0.0.0
```

### Secret (envsubst template)
```
DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
READ_REPLICA_URL_1=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
READ_REPLICA_URL_2=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgresql:5432/rsky_feedgen
RSKY_API_KEY=${RSKY_API_KEY}
```

### Readiness Probe

No standard health endpoint. Use `tcpSocket` on port 3000 as readiness probe (confirms Rocket is accepting connections). The `/well_known` route returns 200 once Rocket is up.

```yaml
readinessProbe:
  tcpSocket:
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 10
```

## DNS (Cloudflare)

After deploy, add A record:
```
feed.know-me.tools  →  IP of feedgen-gateway LoadBalancer
```
Proxy status: **DNS only** (grey cloud) for cert issuance.
