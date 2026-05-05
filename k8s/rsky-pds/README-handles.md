# Handle Resolution Setup for *.know-me.tools

AT Protocol handles under `.know-me.tools` (e.g., `alice.know-me.tools`) require
handle resolution so the web client can map `handle → DID → PDS`.

## How Handle Resolution Works

The `@atproto/identity` HandleResolver tries two methods in order:

1. **DNS TXT** — `dig TXT _atproto.alice.know-me.tools` → `did=did:plc:...`
2. **HTTP well-known** — `GET https://alice.know-me.tools/.well-known/atproto-did` → `did:plc:...`

The PDS already serves `/.well-known/atproto-did` using the `Host` header to look up
the account. The handler is in `rsky-pds/src/well_known.rs`.

## Option A: DNS TXT per user (current approach — manual, no wildcard needed)

For each user `{handle}.know-me.tools`, add a Cloudflare DNS TXT record:

```
Name:    _atproto.{handle}.know-me.tools
Type:    TXT
Value:   did=did:plc:{user-did}
TTL:     300
```

This unblocks login for specific test accounts without wildcard infrastructure.

## Option B: Wildcard (scalable — requires Cloudflare setup)

### Step 1: Cloudflare DNS — add wildcard A record

In Cloudflare DNS for `know-me.tools`:
```
Name:    *.know-me.tools
Type:    A
Value:   35.238.217.92   (pds-gateway IP)
Proxy:   DNS only (grey cloud)
TTL:     Auto
```

### Step 2: cert-manager DNS-01 solver (Cloudflare API token)

Create a Cloudflare API token with `Zone:DNS:Edit` permission for `know-me.tools`.

Apply to cluster:
```bash
kubectl create secret generic cloudflare-api-token \
  --from-literal=api-token=<YOUR_TOKEN> \
  -n cert-manager
```

Update `letsencrypt` ClusterIssuer to add a DNS-01 solver:
```yaml
solvers:
  - dns01:
      cloudflare:
        apiTokenSecretRef:
          name: cloudflare-api-token
          key: api-token
    selector:
      dnsNames:
        - "*.know-me.tools"
```

### Step 3: Re-apply wildcard cert + gateway listeners

```bash
# Apply wildcard cert
kubectl apply -f k8s/rsky-pds/certificate.yaml

# Update gateway with wildcard listeners + apply httproute-handles.yaml
kubectl apply -f k8s/rsky-pds/gateway.yaml
kubectl apply -f k8s/rsky-pds/httproute-handles.yaml
```

The gateway and HTTPRoute manifests are pre-configured in this directory —
just uncomment the wildcard listeners in `gateway.yaml` and apply.

## Current Status (2026-05-05)

- Option A: Manual DNS TXT records can be added for test accounts
- Option B: Blocked on Cloudflare API token + wildcard A record DNS change
- Test account `alice.know-me.tools` (DID: `did:plc:ipc34kig42tau6k25ff35v2f`)
  exists in PLC directory. Add DNS TXT `_atproto.alice.know-me.tools` to unblock login.
