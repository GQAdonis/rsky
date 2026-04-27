# OpenSpec: Infrastructure & Deployment Architecture

> **Status**: Active  
> **Pattern Reference**: `../conduit/k8s/` and `../conduit/.github/workflows/deploy.yaml`

## Cluster Context

The target cluster already runs:
- **Conduit** (Matrix server) in namespace `matrix`
- **Cinny** (Matrix web client) in namespace `matrix`
- **LiveKit** (WebRTC) in namespace `matrix`
- **Envoy Gateway** (`gatewayClassName: eg`) — shared gateway controller
- **cert-manager** with `ClusterIssuer: letsencrypt`
- **ArgoCD** — available for GitOps (not yet used by conduit, but available)

## Namespace

All ATProto services run in the `atproto` namespace.

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: atproto
```

## Gateway Pattern (from conduit)

Each service gets its own `Gateway` resource:

```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: Gateway
metadata:
  name: <service>-gateway
  namespace: atproto
spec:
  gatewayClassName: eg
  listeners:
    - name: https
      hostname: <service>.know-me.tools
      port: 443
      protocol: HTTPS
      tls:
        mode: Terminate
        certificateRefs:
          - kind: Secret
            name: <service>-tls
      allowedRoutes:
        namespaces:
          from: Same
    - name: http
      hostname: <service>.know-me.tools
      port: 80
      protocol: HTTP
      allowedRoutes:
        namespaces:
          from: Same
```

## Certificate Pattern (from conduit)

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: <service>-tls
  namespace: atproto
spec:
  secretName: <service>-tls
  issuerRef:
    name: letsencrypt
    kind: ClusterIssuer
  dnsNames:
    - <service>.know-me.tools
```

## HTTP→HTTPS Redirect Pattern (from conduit)

```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata:
  name: <service>-http-redirect
  namespace: atproto
spec:
  hostnames:
    - <service>.know-me.tools
  parentRefs:
    - name: <service>-gateway
      sectionName: http
  rules:
    - filters:
        - type: RequestRedirect
          requestRedirect:
            scheme: https
            statusCode: 301
```

## ArgoCD GitOps Pattern

Unlike conduit (direct `kubectl apply`), this stack uses ArgoCD:

### ArgoCD Application

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: atproto-stack
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/know-me-tools/rsky
    targetRevision: main
    path: k8s
  destination:
    server: https://kubernetes.default.svc
    namespace: atproto
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### GitHub Actions Deploy Flow

```
push to main
  → build Docker images (per-service)
  → push to ghcr.io/know-me-tools/<service>:<sha>
  → commit updated image tag to k8s/<service>/deployment.yaml
  → ArgoCD detects change → syncs cluster
```

This replaces the conduit pattern of `kubectl apply -f k8s/...` in CI.

## Service Inventory

| Service | Kind | Replicas | Port | PVC? |
|---------|------|----------|------|------|
| rsky-pds | StatefulSet | 1 | 3000 | Yes (data) |
| rsky-relay | Deployment | 1 | 8080 | Yes (rocksdb/sqlite) |
| rsky-feedgen | Deployment | 1 | 3000 | No |
| rsky-labeler | Deployment | 1 | — | No |
| rsky-jetstream | Deployment | 1 | — | No |
| web-client | Deployment | 2 | 80 | No |
| postgresql | StatefulSet | 1 | 5432 | Yes |

## Secrets Management

Follow conduit `envsubst` pattern:
- `secret.yaml` files contain `${VAR_NAME}` placeholders
- GitHub Actions substitutes values from repository secrets
- Secrets never committed to git

## DNS Requirements

Add A records pointing to Gateway LoadBalancer IPs:

```
social.know-me.tools  →  IP of web-client-gateway
pds.know-me.tools     →  IP of pds-gateway
relay.know-me.tools   →  IP of relay-gateway
feed.know-me.tools    →  IP of feedgen-gateway
```

## AT Protocol Well-Known Endpoints

rsky-pds must serve:
- `/.well-known/atproto-did` — returns the service DID
- `/.well-known/did.json` — DID document (for did:web resolution)

These are served by rsky-pds itself (not a separate nginx sidecar, unlike conduit's matrix well-known).
