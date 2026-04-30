# Capability: web-client

This capability defines how the public web client for `social.know-me.tools` is sourced, built, and deployed.

## Purpose

The web client is the user-facing AT Protocol social app served at `https://social.know-me.tools`. It is sourced from the `Ouranos` Next.js project (`github.com/sudoWright/ouranos_atproto`) tracked as a git submodule at `web-client/`, built into a container image, and deployed as a stateless Kubernetes `Deployment`.

## Requirements

### Requirement: Web client source MUST be tracked as a git submodule at `web-client/`

The Ouranos source MUST be present in the repository as a git submodule with `.gitmodules` registering its path and URL. CI MUST clone the repository with `submodules: recursive`.

#### Scenario: Fresh clone produces a buildable web-client

- **WHEN** a developer (or CI) runs `git clone --recurse-submodules <rsky-repo>`
- **THEN** `web-client/package.json` and `web-client/next.config.*` exist and `docker build -f Dockerfile.web-client .` succeeds

### Requirement: Web client image MUST be built and tagged identically to rsky service images

The web client image MUST be built and pushed to `ghcr.io/know-me-tools/web-client` with both `${{ github.sha }}` and `latest` tags, following the same image-tagging discipline as rsky services.

#### Scenario: Web client deploy uses SHA-pinned image

- **WHEN** the web client deploy workflow runs successfully
- **THEN** the live `Deployment`'s `image:` field references `ghcr.io/know-me-tools/web-client:<sha>`, where `<sha>` matches the commit that triggered the deploy

### Requirement: Web client build MUST point at `pds.know-me.tools`

The web client image MUST be built with `NEXT_PUBLIC_ATP_SERVICE_URL` set to `https://pds.know-me.tools` (or the configured PDS host) so that browser-side code talks to this stack's PDS, not Bluesky's public service.

#### Scenario: Browser requests target this PDS

- **WHEN** a user loads `https://social.know-me.tools` in a browser and signs in
- **THEN** the browser's network requests for `com.atproto.server.createSession` go to `https://pds.know-me.tools`, not `https://bsky.social`

### Requirement: Web client deployment MUST NOT carry server-side PDS credentials

The web client is a pure client of the PDS over standard ATProto XRPC. Its k8s `Deployment` and image MUST NOT contain PDS admin passwords, JWT signing keys, or any other server-side secret.

#### Scenario: Web client manifest review

- **WHEN** the web client `Deployment` and any associated `Secret` are inspected
- **THEN** they reference only client-safe configuration (`NEXT_PUBLIC_*` env vars, public DNS hosts) and contain no admin credentials, JWT keys, or PLC rotation keys
