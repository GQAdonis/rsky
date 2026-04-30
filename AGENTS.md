<!-- superpowers-codex bootstrap (managed) -->
## Superpowers Bootstrap (Codex)

<IMPORTANT>
You have access to Superpowers skills.

**Skill bootstrap instructions:**
- Load and follow `/Users/gqadonis/.codex/superpowers/SKILL.md` before doing anything else.
- After loading it, announce: "Using Superpowers bootstrap skill to guide skill usage."
</IMPORTANT>
<!-- /superpowers-codex bootstrap -->

## Agent Operating Protocol

### 1. Skill Loading (Every Turn)

Before taking any action in each turn, identify ALL skills that might be relevant to the work at hand, then **load the top 3 most relevant**. Examples:

| Task | Likely Skills |
|---|---|
| Writing Rust code | `rust-patterns`, `async-patterns`, `error-handling` |
| Writing tests | `rust-testing`, `tdd-workflow`, `verification-loop` |
| Designing APIs | `api-design`, `axum-patterns` |
| Debugging build errors | `systematic-debugging`, `rust-patterns` |
| Planning work | `writing-plans`, `blueprint` |

Do not skip this step. Skill loading ensures idiomatic, high-quality output that improves iteratively.

### 2. Karpathy Guidelines

Follow the **karpathy-guidelines** skill principles on every code generation turn:

- **Surgical changes**: Make the smallest possible diff. Do not refactor surrounding code unless it directly blocks the change.
- **Surface assumptions**: Before writing code, state key assumptions explicitly. If unsure, ask.
- **Define success criteria**: Before implementing, define how to verify the change works (compile check, test, CI).
- **Avoid overcomplication**: Prefer the simplest solution that satisfies the requirement. Do not add abstractions, generics, or indirection unless clearly needed now.
- **Verify after every change**: Run `cargo check`, `cargo fmt`, or `cargo clippy` after modifying files. Never claim a change is done without verification.

### 3. Progress Reporting (Every Turn)

At the end of **every turn** — whether code generation, research, debugging, or planning — append a structured status block:

```markdown
---
## Status

**Task**: [current task name from the task list]
**Phase**: [phase name if applicable]
**Progress**: [X/Y complete] — [brief description of what was done this turn]

### Checklist (ordered)
- [x] Step 1 — description
- [x] Step 2 — description
- [ ] Step 3 — description *(next)*
- [ ] Step 4 — description

**Next**: [what the next turn should do]
**Blockers**: [any blockers, or "none"]
```

This ensures every turn ends with a clear picture of where we are, what just happened, and what comes next. Never omit this block.

# Repository Instructions

## What This Repository Is

`rsky` is a Rust implementation of the AT Protocol, the decentralized social media protocol behind Bluesky. It is maintained for the Blacksky/Know Me stack and is pre-1.0, so APIs and behavior may change.

The repo contains:
- Core Rust libraries for AT Protocol primitives.
- Rust services for PDS, relay, feed generation, moderation, indexing, video, and sync jobs.
- A Next.js web client under `web-client/`.
- A Dioxus CAR/repo browser under `rsky-satnav/`.
- Kubernetes and GitHub Actions deployment assets for `social.know-me.tools`.

## First Checks

Before changing files:
- Run `git status --short` and preserve user changes.
- Read the relevant crate or app README before editing that area.
- Prefer narrow, crate-specific changes over workspace-wide refactors.
- Do not add dependencies, change tooling, or perform large cross-crate refactors without explicit discussion.
- Never commit secrets, private keys, API tokens, real service credentials, or generated local environment files.

## Toolchain

Rust is pinned by `rust-toolchain.toml`:

```bash
rustc 1.86
components: clippy, rustfmt
```

Use the pinned toolchain. Do not upgrade Rust or the workspace edition policy unless that is the task.

The root Cargo workspace includes most crates, but `rsky-pdsadmin/` has its own local workspace. Run `cargo` from `rsky-pdsadmin/` when working on that CLI.

## Build, Test, and Format

Prefer package-scoped commands:

```bash
cargo check -p rsky-pds
cargo build --release -p rsky-relay
cargo test --release -p rsky-repo
cargo test --release -p rsky-repo -- merkle_search_tree
cargo fmt -- --check
cargo fmt
```

Avoid defaulting to full-workspace build/test commands for routine work. CI builds changed crates with `-p <crate>`, and Docker builds should stay service-scoped.

For `rsky-pdsadmin`:

```bash
cd rsky-pdsadmin
cargo check
cargo test
```

For `web-client`:

```bash
cd web-client
npm install
npm run dev
npm run build
npm run lint
npm run gen-api
```

For `rsky-satnav`:

```bash
cd rsky-satnav
dx serve
npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch
```

`dx` requires `dioxus-cli`; Satnav also uses its own `package.json` for Tailwind.

## Workspace Map

Core libraries:
- `rsky-syntax`: DID, handle, NSID, TID, and AT URI parsing/validation.
- `rsky-crypto`: secp256k1 and p256 signing plus key serialization.
- `rsky-identity`: DID and handle resolution over DNS/HTTP.
- `rsky-common`: shared utilities and data structures.
- `rsky-lexicon`: AT Protocol schema definitions and Bluesky API types.
- `rsky-repo`: Merkle Search Tree, repo operations, CBOR/DAG-CBOR, and CAR handling.
- `rsky-firehose`: WebSocket firehose subscriber/client utilities.

Services and apps:
- `rsky-pds`: Rocket + Diesel Personal Data Server using PostgreSQL, S3-compatible blob storage, and Mailgun.
- `rsky-relay`: Tokio relay with SQLite and `fjall` storage.
- `rsky-wintermute`: AppView/indexer using Tokio and `heed`/LMDB.
- `rsky-feedgen`: Rocket + Diesel feed generator using PostgreSQL.
- `rsky-labeler`: Firehose consumer for moderation labels.
- `rsky-jetstream-subscriber`: Jetstream to JSON event transformer.
- `rsky-satnav`: Dioxus CAR/repository explorer.
- `rsky-video`: Axum video upload/transcoding service.
- `palomar-sync`: PostgreSQL to OpenSearch sync job.
- `rsky-pdsadmin`: separate administrative CLI workspace.
- `web-client`: Ouranos Next.js client for the hosted social UI.

Edition notes:
- Most crates are Rust 2021.
- `rsky-relay`, `rsky-wintermute`, `rsky-video`, and `rsky-pdsadmin` use Rust 2024.

## Architecture Notes

Core dependency direction is broadly:

```text
rsky-syntax
  -> rsky-crypto
    -> rsky-identity
      -> rsky-common
        -> rsky-lexicon
          -> rsky-repo
            -> rsky-firehose
```

Network flow:

```text
Users -> rsky-pds -> rsky-relay -> rsky-wintermute
                         |          -> rsky-feedgen
                         |          -> rsky-labeler
                         -> rsky-jetstream-subscriber
```

Important data formats and structures:
- MST in `rsky-repo` is the self-authenticating repo structure.
- Repo serialization uses CBOR/DAG-CBOR via `serde_ipld_dagcbor`.
- CIDs use the `lexicon_cid` alias/package.
- CAR files are central to repo export, sync, and Satnav browsing.

## Database and Migrations

Migration directories:
- `rsky-pds/migrations/`
- `rsky-feedgen/migrations/`
- `rsky-wintermute/migrations/`

When touching Diesel models, schema, or SQL:
- Keep migrations reversible when the migration framework expects `down.sql`.
- Verify affected crate tests or at least `cargo check -p <crate>`.
- Preserve PostgreSQL assumptions for PDS/feedgen and LMDB assumptions for Wintermute.

## Environment and Secrets

`rsky-pds` expects environment such as:

```text
PDS_HOSTNAME
PDS_SERVICE_DID
PDS_SERVICE_HANDLE_DOMAINS
PDS_ADMIN_PASS
PDS_JWT_KEY_K256_PRIVATE_KEY_HEX
PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX
PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX
PDS_MAILGUN_API_KEY
PDS_MAILGUN_DOMAIN
PDS_EMAIL_FROM_ADDRESS
PDS_EMAIL_FROM_NAME
DATABASE_URL
```

CI contains placeholder/reference values for some PDS variables. Treat real values as secrets. Kubernetes `secret.yaml` files should stay `envsubst`-compatible with `${VAR}` placeholders.

## Kubernetes and Deployment

The deployment target is the Know Me ATProto stack:
- Namespace: `atproto`
- Registry: `ghcr.io/know-me-tools`
- Domain family: `social.know-me.tools`, `pds.know-me.tools`, `relay.know-me.tools`, `feed.know-me.tools`
- Gateway: Envoy Gateway with cert-manager `ClusterIssuer: letsencrypt`
- GitOps: GitHub Actions builds images and ArgoCD syncs `k8s/`

Deployment conventions:
- Do not hardcode `ghcr.io/blacksky-algorithms` in `k8s/`; this deployment uses `ghcr.io/know-me-tools`.
- Use SHA image tags for deterministic manifests.
- Keep Secret templates paired with deployments that require secrets.
- Certificates and HTTPRoutes/Gateways should be updated together.
- Prefer StatefulSet for stateful services such as PDS and storage-backed relay. Prefer Deployment for stateless services and web client.

## Web Client

`web-client/` is Ouranos, a Next.js 14 client. It is configured for the hosted stack with build-time `NEXT_PUBLIC_*` values:

```text
NEXT_PUBLIC_ATP_SERVICE_URL=https://pds.know-me.tools
NEXT_PUBLIC_ATP_APPVIEW_URL=https://api.bsky.app
```

Local development also needs:

```text
NEXTAUTH_SECRET
NEXTAUTH_URL=http://localhost:3000
```

Do not put PDS credentials or server-side ATProto secrets into the web client.

## Coding Style

Rust:
- Follow existing module layout and error-handling style in the crate being edited.
- Prefer typed parsing/serialization over ad hoc string manipulation, especially for AT Protocol identifiers and lexicon data.
- Keep async boundaries clear; many services use Tokio, while Rocket/Diesel areas may be synchronous.
- Add focused tests for protocol parsing, repo/MST behavior, auth, database logic, and bug fixes.
- Run `cargo fmt` before finishing Rust edits.

Frontend:
- Follow the existing Next.js, Tailwind, Radix, and React Query patterns in `web-client/`.
- Keep UI changes consistent with the current app structure under `src/app`, `src/components`, `src/containers`, and `src/lib`.
- Run `npm run lint` or `npm run build` when touching behavior or shared frontend code.

Docs:
- Keep `README.md`, crate READMEs, `openspec/specs/`, and `k8s/README.md` aligned when changing architecture or deployment behavior.

## Contribution Constraints

From `CONTRIBUTING.md` and local planning constraints:
- Avoid large refactors.
- Avoid unnecessary dependencies.
- Discuss new features or tooling changes first.
- Keep library changes separate from service changes when practical.
- Understand AT Protocol concepts before changing protocol behavior: PDS, Relay, AppView, DIDs, handles, repos, MSTs, and firehose events.

## CI Reference

Primary workflows:
- `.github/workflows/rust.yml`: per-crate `cargo check`, `cargo build --release`, `cargo test --release`, and `cargo fmt -- --check`.
- `.github/workflows/build-web-client.yml`: builds `Dockerfile.web-client`.
- Service-specific Docker workflows exist for PDS, relay, feedgen, labeler, jetstream subscriber, and pdsadmin.

When practical, mirror CI locally with the narrowest command that covers the touched area.
