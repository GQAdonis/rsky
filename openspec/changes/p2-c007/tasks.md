# Tasks: p2-c007 (human)

- [ ] `curl -s https://pds.know-me.tools/xrpc/_health` — returns 200 with version
- [ ] `curl -s https://relay.know-me.tools/_health` — returns 200
- [ ] `curl -s https://feed.know-me.tools/xrpc/_health` — returns 200
- [ ] `curl -sI https://social.know-me.tools` — returns HTTP/2 200
- [ ] `kubectl get pods -n atproto` — all pods Running, none in CrashLoopBackOff
- [ ] `kubectl get certificate -n atproto` — all 4 certificates Ready: True
