# Tasks: p1-c003

- [ ] Create `k8s/rsky-relay/secret.yaml` — envsubst template for `RELAY_ADMIN_PASSWORD`
- [ ] Create `k8s/rsky-relay/statefulset.yaml` — StatefulSet, 100Gi PVC at `/data`, health probe `/_health`
- [ ] Create `k8s/rsky-relay/service.yaml` — ClusterIP port 9000
- [ ] Create `k8s/rsky-relay/certificate.yaml` — `relay.know-me.tools`, ClusterIssuer letsencrypt
- [ ] Create `k8s/rsky-relay/gateway.yaml` — `gatewayClassName: eg`, listeners HTTPS+HTTP for `relay.know-me.tools`
- [ ] Create `k8s/rsky-relay/httproute-https.yaml` — route HTTPS to rsky-relay:9000
- [ ] Create `k8s/rsky-relay/httproute-redirect.yaml` — HTTP→HTTPS 301
