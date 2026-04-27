# Tasks: p0-c003

- [ ] Create `k8s/rsky-pds/configmap.yaml` (PDS_HOSTNAME, PDS_SERVICE_DID, PDS_CRAWLERS, ROCKET_PORT/ADDRESS, etc.)
- [ ] Create `k8s/rsky-pds/secret.yaml` (envsubst template: all PDS_* keys, DATABASE_URL, GCS HMAC as AWS_*, GCS_BUCKET_NAME)
- [ ] Create `k8s/rsky-pds/pvc.yaml` (10Gi, atproto-ssd-immediate — Immediate binding)
- [ ] Create `k8s/rsky-pds/statefulset.yaml` (image: IMAGE_TAG placeholder, envFrom configmap + secret, PVC mount, liveness/readiness /xrpc/_health)
- [ ] Create `k8s/rsky-pds/service.yaml` (ClusterIP, port 3000)
- [ ] Create `k8s/rsky-pds/gateway.yaml` (pds.know-me.tools, HTTPS+HTTP, gatewayClassName: eg)
- [ ] Create `k8s/rsky-pds/certificate.yaml` (ClusterIssuer: letsencrypt, secretName: pds-tls)
- [ ] Create `k8s/rsky-pds/httproute-https.yaml` (pds.know-me.tools → rsky-pds:3000, parent: pds-gateway https)
- [ ] Create `k8s/rsky-pds/httproute-redirect.yaml` (HTTP → HTTPS 301, parent: pds-gateway http)
- [ ] Add Cloudflare DNS note to `k8s/rsky-pds/README.md` (DNS-only / grey cloud required for ACME)
