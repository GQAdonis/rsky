# Tasks: p2-c006 (human)

- [x] Run `kubectl get gateway -n atproto -o wide` — record all 4 external IPs
- [x] Create Cloudflare A record: `pds.know-me.tools` → pds-gateway IP (DNS only)
- [x] Create Cloudflare A record: `relay.know-me.tools` → relay-gateway IP (DNS only)
- [x] Create Cloudflare A record: `feed.know-me.tools` → feedgen-gateway IP (DNS only)
- [x] Create Cloudflare A record: `social.know-me.tools` → web-client-gateway IP (DNS only)
- [x] Wait for DNS propagation (`dig pds.know-me.tools +short` returns gateway IP)
- [x] Watch `kubectl get certificate -n atproto -w` — all 4 certificates reach `Ready: True`

Completed 2026-04-28: all 4 certs Ready, confirmed via kubectl.
