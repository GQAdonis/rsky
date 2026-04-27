# p0-c004: Ouranos Web Client Kubernetes Manifests

**Phase**: phase-0-foundation  
**Priority**: 4 (depends on p0-c001 for Dockerfile pattern; can parallel with p0-c003)  
**Assigned to**: claude-code  

## Decision Summary

- **Client**: Ouranos (`github.com/sudoWright/ouranos_atproto`) — Next.js 14
- **Kind**: Deployment (stateless, 2 replicas)
- **Image**: `ghcr.io/know-me-tools/web-client:IMAGE_TAG`
- **Build**: `docker build` from a submodule or copied source at `web-client/`
- **Domain**: `social.know-me.tools`
- **AppView**: Bluesky public API (`https://api.bsky.app`) for Phase 1 — network reads
- **PDS**: `https://pds.know-me.tools` — our sovereign PDS for auth + writes

## Repository Setup

Ouranos source lives at `web-client/` — added as a git submodule:
```
git submodule add https://github.com/sudoWright/ouranos_atproto web-client
```

Dockerfile at `web-client/Dockerfile` (or root `Dockerfile.web-client`).

## Dockerfile for Ouranos

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY web-client/package*.json ./
RUN npm ci
COPY web-client/ .
ARG NEXT_PUBLIC_ATP_SERVICE_URL
ARG NEXT_PUBLIC_ATP_APPVIEW_URL
ENV NEXT_PUBLIC_ATP_SERVICE_URL=$NEXT_PUBLIC_ATP_SERVICE_URL
ENV NEXT_PUBLIC_ATP_APPVIEW_URL=$NEXT_PUBLIC_ATP_APPVIEW_URL
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public
EXPOSE 3000
CMD ["node", "server.js"]
```

## Environment Variables

### ConfigMap (non-secret, baked at build time via ARG)
```
NEXT_PUBLIC_ATP_SERVICE_URL=https://pds.know-me.tools
NEXT_PUBLIC_ATP_APPVIEW_URL=https://api.bsky.app
```

No runtime secrets needed — ATProto auth is fully client-side via XRPC.

## Files to Create

```
k8s/web-client/
├── deployment.yaml
├── service.yaml
├── configmap.yaml
├── gateway.yaml
├── certificate.yaml
├── httproute-https.yaml
└── httproute-redirect.yaml
```

## Deployment Key Config

```yaml
replicas: 2
image: ghcr.io/know-me-tools/web-client:IMAGE_TAG
ports:
  - containerPort: 3000
envFrom:
  - configMapRef:
      name: web-client-config
resources:
  requests:
    cpu: 100m
    memory: 256Mi
  limits:
    cpu: 500m
    memory: 512Mi
```

## DNS (Cloudflare)

```
social.know-me.tools  →  IP of web-client-gateway LoadBalancer
```
Proxy status: **DNS only** (grey cloud) for initial cert issuance. Can enable proxy after first successful cert.
