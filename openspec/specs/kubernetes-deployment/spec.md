# Capability: kubernetes-deployment

This capability defines how rsky services are deployed to GKE under `social.know-me.tools` via ArgoCD, Envoy Gateway, cert-manager, and GitHub Actions.

## Purpose

All rsky services run in the `atproto` namespace on the `client-cluster` GKE cluster. Each service has its own Gateway resource for independent TLS termination via Let's Encrypt (`ClusterIssuer: letsencrypt`), and ArgoCD Applications keep cluster state synced with this repository's `k8s/` directory.

## Requirements

### Requirement: Stateful services MUST use `StatefulSet`, stateless services MUST use `Deployment`

`rsky-pds` (which holds the per-actor repo state) and `postgres` MUST be deployed as `StatefulSet`s with stable storage identity. All other rsky services (`rsky-relay`, `rsky-feedgen`, `rsky-labeler`, `rsky-jetstream-subscriber`, web client) MUST be deployed as `Deployment`s and SHOULD scale horizontally.

#### Scenario: rsky-pds restart preserves volume identity

- **WHEN** the `rsky-pds` pod is deleted and recreated by Kubernetes
- **THEN** the new pod re-attaches to the same `PersistentVolumeClaim` and recovers its existing repo state, not a fresh empty volume

### Requirement: Each service MUST have its own Gateway resource and Certificate

Services MUST NOT share a single Gateway. Each public service (`pds.know-me.tools`, `relay.know-me.tools`, `feed.know-me.tools`, `social.know-me.tools`) MUST have a dedicated `Gateway` resource (gatewayClass `eg`, Envoy Gateway) and a matching cert-manager `Certificate` referencing `ClusterIssuer: letsencrypt`.

#### Scenario: New service gains its own TLS

- **WHEN** a new service is added to the stack with public DNS
- **THEN** its k8s manifest set includes a dedicated `Gateway` resource, a `Certificate` resource referencing `ClusterIssuer: letsencrypt`, and `HTTPRoute` resources binding the service's traffic to that Gateway

### Requirement: Secrets MUST be provided via `envsubst`-templated YAML, not committed

Each service that needs secrets MUST ship a `secret.yaml` template containing `${VAR}` placeholders, and CI MUST `envsubst < secret.yaml` before applying. Real secret values MUST never be committed to git.

#### Scenario: Deploy workflow injects secrets from GitHub Actions

- **WHEN** the GitHub Actions deploy workflow runs against the `atproto` namespace
- **THEN** it reads service-specific secrets from GitHub Actions secrets, runs `envsubst` against each `secret.yaml` template, and applies the result via `kubectl apply` — and the original templated `secret.yaml` files in git contain only `${VAR}` placeholders

### Requirement: ArgoCD Applications MUST use automated sync with prune

ArgoCD `Application` resources for rsky services MUST configure `syncPolicy.automated.prune: true` and `selfHeal: true` so cluster state cannot drift from the git source of truth.

#### Scenario: Manifest deletion in git removes the live resource

- **WHEN** a manifest file is removed from the repo's `k8s/` tree and pushed to `main`
- **THEN** ArgoCD prunes the corresponding live resource within its sync interval

### Requirement: Deploy workflow MUST wait for `rollout status` before reporting success

The GitHub Actions deploy workflow MUST run `kubectl rollout status` (or equivalent) for each affected workload after applying manifests, and MUST fail the workflow if rollout does not complete within a bounded timeout. Silent rollout failures are not acceptable.

#### Scenario: Failed rollout fails the workflow

- **WHEN** a deploy applies a manifest that produces a `CrashLoopBackOff` pod
- **THEN** the workflow's `rollout status` step times out and exits non-zero, marking the deploy run as failed

### Requirement: Database services MUST initialize required databases at first start

PostgreSQL MUST run with an `initdb` ConfigMap or equivalent so that all rsky-required databases (`rsky_pds`, `rsky_feedgen`, …) and the `pgvector` extension exist on first start without manual `psql` setup.

#### Scenario: Fresh cluster bootstrap

- **WHEN** the cluster is bootstrapped from an empty state and Postgres comes up for the first time
- **THEN** `psql -c "\\l"` lists all rsky databases and `psql -c "CREATE EXTENSION IF NOT EXISTS vector;"` succeeds without operator intervention

### Requirement: Public DNS MUST point at the cluster ingress

DNS records for `pds.know-me.tools`, `relay.know-me.tools`, `feed.know-me.tools`, and `social.know-me.tools` MUST resolve to the cluster's external load balancer / Envoy Gateway address.

#### Scenario: Smoke test resolves PDS endpoint

- **WHEN** an external client runs `curl https://pds.know-me.tools/xrpc/_health`
- **THEN** the request resolves to the cluster ingress and returns a 2xx response with the PDS's health payload

### Requirement: Deployed endpoints MUST pass smoke tests after every deploy

After each successful deploy, the workflow (or a dedicated smoke-test job) MUST hit the public endpoints (`/xrpc/_health`, `/xrpc/com.atproto.server.describeServer`, web client homepage) and confirm 2xx responses.

#### Scenario: Post-deploy smoke test failure halts further deploys

- **WHEN** a deploy completes but the post-deploy smoke test against `pds.know-me.tools/xrpc/com.atproto.server.describeServer` returns 5xx
- **THEN** the workflow run is marked failed and a follow-up incident is opened
