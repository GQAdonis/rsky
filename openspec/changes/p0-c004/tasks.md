# Tasks: p0-c004

- [ ] Add Ouranos as git submodule: `git submodule add https://github.com/sudoWright/ouranos_atproto web-client`
- [ ] Create `Dockerfile.web-client` at repo root (Next.js standalone build, node:20-alpine)
- [ ] Create `k8s/web-client/configmap.yaml` (NEXT_PUBLIC_ATP_SERVICE_URL, NEXT_PUBLIC_ATP_APPVIEW_URL)
- [ ] Create `k8s/web-client/deployment.yaml` (2 replicas, IMAGE_TAG placeholder, envFrom configmap)
- [ ] Create `k8s/web-client/service.yaml` (ClusterIP, port 3000)
- [ ] Create `k8s/web-client/gateway.yaml` (social.know-me.tools, HTTPS+HTTP, gatewayClassName: eg)
- [ ] Create `k8s/web-client/certificate.yaml` (ClusterIssuer: letsencrypt, secretName: web-client-tls)
- [ ] Create `k8s/web-client/httproute-https.yaml` (social.know-me.tools → web-client:3000)
- [ ] Create `k8s/web-client/httproute-redirect.yaml` (HTTP → HTTPS 301)
- [ ] Add Cloudflare DNS note to `k8s/web-client/README.md`
