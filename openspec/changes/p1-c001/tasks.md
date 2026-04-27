# Tasks: p1-c001

- [ ] Patch `rsky-relay/src/server/server.rs:157` — change `127.0.0.1` to `0.0.0.0`
- [ ] Create `rsky-relay/Dockerfile` with WORKDIR `/data` in runtime stage
- [ ] Verify `cargo check -p rsky-relay` passes after patch
