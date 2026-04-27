# Tasks: p1-c004

- [ ] Create `k8s/rsky-feedgen/configmap.yaml` — FEEDGEN_SERVICE_DID, FEEDGEN_HOSTNAME, ROCKET_PORT, ROCKET_ADDRESS
- [ ] Create `k8s/rsky-feedgen/secret.yaml` — envsubst template for DATABASE_URL, READ_REPLICA_URL_{1,2}, RSKY_API_KEY
- [ ] Create `k8s/rsky-feedgen/deployment.yaml` — 1 replica, tcpSocket readiness probe on port 3000
- [ ] Create `k8s/rsky-feedgen/service.yaml` — ClusterIP port 3000
- [ ] Create `k8s/rsky-feedgen/certificate.yaml` — `feed.know-me.tools`, ClusterIssuer letsencrypt
- [ ] Create `k8s/rsky-feedgen/gateway.yaml` — `gatewayClassName: eg`, HTTPS+HTTP listeners
- [ ] Create `k8s/rsky-feedgen/httproute-https.yaml` — route to rsky-feedgen:3000
- [ ] Create `k8s/rsky-feedgen/httproute-redirect.yaml` — HTTP→HTTPS 301
