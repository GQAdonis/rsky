# OpenSpec: Web Client Selection

> **Status**: Decision Made (2026-04-27, kbd-assess research)  
> **Decision**: Ouranos (`github.com/sudoWright/ouranos_atproto`) for Phase 1

## Context

AT Protocol separates concerns into PDS (data), Relay (indexing), and AppView (query backend). Most web clients are tightly coupled to the Bluesky AppView. For a sovereign deployment, the client must either:

1. Connect to Bluesky's AppView (network effects, but dependency on Bluesky infrastructure), OR
2. Connect to a self-hosted AppView (full sovereignty, more work), OR
3. Operate in "hybrid" mode (own PDS + Bluesky AppView for read queries)

## Candidates

### Option A: Bluesky Web (social-app)
- **Repo**: `github.com/bluesky-social/social-app`
- **Stack**: React Native Web + Expo
- **AppView**: Bluesky's AppView (default), but configurable
- **Self-host complexity**: Medium — can be configured with custom PDS endpoint
- **UX**: ⭐⭐⭐⭐⭐ — reference implementation, most features
- **Verdict**: Best UX, requires Bluesky AppView for full feature set

### Option B: Graysky
- **Repo**: `github.com/mozzius/graysky`
- **Stack**: React Native + Expo
- **AppView**: Bluesky AppView
- **Self-host complexity**: Low — primarily a mobile/web client
- **UX**: ⭐⭐⭐⭐ — clean, performant
- **Verdict**: Good but still tied to Bluesky AppView

### Option C: Skeetly / Klearsky / Langit
- Various community clients
- **Verdict**: Less maintained, not production-ready for self-hosting

### Option D: Custom lightweight client
- Build a minimal React/Next.js client against rsky-wintermute (AppView)
- **Verdict**: Phase 4+ work; too much for Phase 1

## Decision (Phase 1): Ouranos

**Use Ouranos** (`github.com/sudoWright/ouranos_atproto`) — a friendly Bluesky web client built on Next.js.

Rationale over social-app:
- Standard Next.js app → `docker build` works cleanly, no Expo/EAS build pipeline
- `NEXT_PUBLIC_*` env vars configure PDS + AppView endpoints — no runtime patching
- Works in hybrid mode: own PDS for writes, Bluesky AppView for network-wide reads
- Actively maintained, MIT licensed

social-app remains the UX gold standard but requires Expo managed workflow / EAS for web builds — not suitable for a clean k8s Deployment in Phase 0.

## Configuration Requirements

```env
# Ouranos environment variables (Next.js NEXT_PUBLIC_ pattern)
NEXT_PUBLIC_ATP_SERVICE_URL=https://pds.know-me.tools
NEXT_PUBLIC_ATP_APPVIEW_URL=https://api.bsky.app   # Bluesky AppView (Phase 1)
# Phase 3+: change NEXT_PUBLIC_ATP_APPVIEW_URL to https://appview.know-me.tools
```

## Deployment

- Build from source: `docker build -t ghcr.io/know-me-tools/web-client:<sha> .`
- Runs as stateless `Deployment` (replicas: 2, no persistent storage)
- Exposed via Envoy Gateway at `social.know-me.tools`
- No secrets required (all ATProto auth happens client-side via XRPC)

## Future: Full Sovereignty

Once `rsky-wintermute` is production-ready:
- Point client's AppView URL to `appview.know-me.tools` (rsky-wintermute)
- This eliminates dependency on Bluesky infrastructure entirely
- Agent-generated content and feeds become fully controllable

## Open Questions

1. Does `social-app` support building a static web export (no Node.js server)?
2. What is the minimal config to set a default PDS for new account registration?
3. Is there a community-maintained Docker image, or do we build from source?

> These questions should be resolved via Tavily research in `/kbd-assess`.
