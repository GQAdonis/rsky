# OpenSpec: social.know-me.tools ATProto Stack

> **Status**: Active  
> **Domain**: `social.know-me.tools`  
> **Stack**: Rust (rsky) + GKE + ArgoCD + Envoy Gateway + Let's Encrypt  

## Project Goal

Deploy a fully sovereign AT Protocol social network at `social.know-me.tools` using the `rsky` Rust implementation. The deployment must be production-ready, federated with the Bluesky network, and architecturally prepared for AI agent participation via decentralized identity and Matrix-based encrypted communications.

## Network Topology

```
Internet
  └── Envoy Gateway (GKE, gatewayClassName: eg)
        ├── social.know-me.tools     → Web Client (best-of-breed ATProto UI)
        ├── pds.know-me.tools        → rsky-pds (Personal Data Server)
        ├── relay.know-me.tools      → rsky-relay (Firehose relay)
        ├── feed.know-me.tools       → rsky-feedgen (Feed generator)
        └── [future] agent.know-me.tools → UAR agent endpoints
```

## Core Services

| Service | Crate | Role | Storage |
|---------|-------|------|---------|
| PDS | `rsky-pds` | AT Protocol Personal Data Server | PostgreSQL + S3 |
| Relay | `rsky-relay` | Network firehose aggregator | SQLite + fjall |
| Feed Generator | `rsky-feedgen` | Algorithmic feeds | PostgreSQL |
| Labeler | `rsky-labeler` | Content moderation | — |
| Jetstream | `rsky-jetstream-subscriber` | JSON event stream | — |
| Web Client | TBD (see spec 01) | Browser UI over ATProto AppView | — |

## Infrastructure Requirements

- **Cluster**: GKE `client-cluster`, region `us-central1`
- **Namespace**: `atproto`
- **Gateway**: Envoy Gateway (`gatewayClassName: eg`) — same cluster as `conduit` (matrix namespace)
- **TLS**: cert-manager `ClusterIssuer: letsencrypt` — same issuer as conduit
- **Images**: `ghcr.io/know-me-tools/<service>:<sha>`
- **Deploy**: GitHub Actions → ArgoCD sync (not direct `kubectl apply` like conduit)
- **Secrets**: `envsubst`-injected from GitHub Actions secrets

## Deployment Strategy: ArgoCD over Direct kubectl

Unlike the conduit deployment (direct `kubectl apply`), this stack uses ArgoCD:

- GitHub Actions builds and pushes Docker images, then commits updated image tags to a `k8s/` directory
- ArgoCD watches the repo and syncs the cluster state automatically
- This enables GitOps: any tool (Codex, Cursor, human) can open a PR with a k8s change and ArgoCD deploys it

## Agent Integration (Future Phases)

- **Librefang** (`github.com/GQAdonis/librefang`): agent framework for content management agents
- **Universal Agent Runtime** (`github.com/Prometheus-AGS/universal-agent-runtime`): orchestrates agent personas
- **Agent DIDs**: each agent gets an ATProto DID via rsky-pds, enabling them to post/interact as accounts
- **Matrix coordination**: agents communicate via encrypted Matrix rooms (conduit server on same cluster)
- **Digital twins**: account owner personas managed by agents for social network marketing automation

## Web Client Selection Criteria

See `openspec/specs/01-web-client.md` for detailed analysis. Summary:
- Must work with a self-hosted PDS (not locked to Bluesky AppView)
- Must support handle-based login to `pds.know-me.tools`
- Prefer open source with active maintenance
- Target: best UX available without building a custom client in Phase 1

## Phase Structure

| Phase | Name | Goal |
|-------|------|------|
| 0 | Foundation & KBD Setup | Project scaffolding, OpenSpec, k8s namespace, ArgoCD app |
| 1 | Core Infrastructure | rsky-pds + PostgreSQL + S3 deployed and federated |
| 2 | Relay + FeedGen | Relay crawling network; custom feeds live |
| 3 | Web Client | Best-of-breed client deployed at social.know-me.tools |
| 4 | Agent Integration | Librefang agents with ATProto DIDs; UAR orchestration |
| 5 | Matrix Bridge | Agent-to-agent encrypted comms via conduit |
