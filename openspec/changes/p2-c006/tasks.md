# Tasks: p2-c006 (human)

- [ ] Run `kubectl get gateway -n atproto -o wide` — record all 4 external IPs
- [ ] Create Cloudflare A record: `pds.know-me.tools` → pds-gateway IP (DNS only)
- [ ] Create Cloudflare A record: `relay.know-me.tools` → relay-gateway IP (DNS only)
- [ ] Create Cloudflare A record: `feed.know-me.tools` → feedgen-gateway IP (DNS only)
- [ ] Create Cloudflare A record: `social.know-me.tools` → web-client-gateway IP (DNS only)
- [ ] Wait for DNS propagation (`dig pds.know-me.tools +short` returns gateway IP)
- [ ] Watch `kubectl get certificate -n atproto -w` — all 4 certificates reach `Ready: True`
