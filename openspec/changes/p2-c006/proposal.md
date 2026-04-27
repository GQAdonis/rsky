# p2-c006: Configure Cloudflare DNS

**Phase**: phase-2-commit-and-deploy  
**Priority**: 6 (depends on p2-c005; human task)  
**Assigned to**: human  
**Type**: operational runbook

## Overview

Four Cloudflare DNS A records must be created pointing at GKE Gateway IPs.
DNS must be grey-cloud (DNS-only, not proxied) during initial cert issuance
so cert-manager's ACME HTTP-01 challenge can reach the cluster directly.

## Get Gateway IPs

After ArgoCD syncs and Envoy assigns external IPs (~2-5 min after first sync):

```bash
kubectl get gateway -n atproto -o wide
# Or individually:
kubectl get gateway pds-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
kubectl get gateway relay-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
kubectl get gateway feedgen-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
kubectl get gateway web-client-gateway -n atproto -o jsonpath='{.status.addresses[0].value}'
```

## Cloudflare DNS Records

Set in Cloudflare Dashboard → know-me.tools → DNS:

| Name | Type | Value | Proxy |
|------|------|-------|-------|
| `pds` | A | `<pds-gateway IP>` | DNS only (grey cloud) |
| `relay` | A | `<relay-gateway IP>` | DNS only (grey cloud) |
| `feed` | A | `<feedgen-gateway IP>` | DNS only (grey cloud) |
| `social` | A | `<web-client-gateway IP>` | DNS only (grey cloud) |

## Certificate Issuance

After DNS propagates (~1-2 min), cert-manager issues Let's Encrypt certificates:

```bash
# Watch certificate issuance
kubectl get certificate -n atproto -w

# Check certificate challenges
kubectl get certificaterequest -n atproto
kubectl describe challenge -n atproto
```

All certificates should reach `Ready: True` within 3-5 minutes of DNS propagation.

## After Certs Are Issued

Optionally enable Cloudflare proxy (orange cloud) for DDoS protection.
Note: enabling proxy changes IPs — certs will remain valid (Cloudflare handles TLS termination in proxy mode).
