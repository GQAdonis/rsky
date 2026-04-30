# Spec Delta: p2-c001 — web-client

## ADDED Requirements

### Requirement: Web client source MUST be tracked as a git submodule at `web-client/`

The Ouranos source MUST be present in the repository as a git submodule with `.gitmodules` registering its path (`web-client`) and URL (`https://github.com/sudoWright/ouranos_atproto`). CI MUST clone the repository with `submodules: recursive`.

#### Scenario: Fresh clone produces a buildable web-client tree

- **WHEN** a developer (or CI) runs `git clone --recurse-submodules <rsky-repo>`
- **THEN** `web-client/package.json` and `web-client/next.config.*` exist, and `docker build -f Dockerfile.web-client .` does not fail at the `COPY web-client/` step
