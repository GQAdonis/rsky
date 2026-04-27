# p2-c001: Add Ouranos git submodule

**Phase**: phase-2-commit-and-deploy  
**Priority**: 1 (unblocked; must complete before p2-c002)  
**Assigned to**: claude-code

## Overview

Register `github.com/sudoWright/ouranos_atproto` as a git submodule at `web-client/`.

`Dockerfile.web-client` contains `COPY web-client/ .` — this will fail at build time if
`web-client/` is empty. CI uses `actions/checkout@v4` with `submodules: recursive`,
which requires a `.gitmodules` entry to clone the submodule.

## Command

```bash
git submodule add https://github.com/sudoWright/ouranos_atproto web-client
```

## Files Modified

```
.gitmodules          — new file; registers submodule path and URL
web-client/          — populated with Ouranos HEAD checkout
```

## Verification

After the submodule is added:
1. `.gitmodules` exists and contains the `web-client` entry
2. `web-client/package.json` exists
3. `web-client/next.config.*` exists (confirms it is the Next.js app)
4. `docker build -f Dockerfile.web-client .` does not fail on COPY step
