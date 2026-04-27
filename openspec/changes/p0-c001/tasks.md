# Tasks: p0-c001

- [ ] Rewrite `rsky-pds/Dockerfile` — local COPY, multi-stage, debian:bookworm-slim runtime
- [ ] Rewrite `rsky-feedgen/Dockerfile` — same pattern
- [ ] Rewrite `rsky-labeler/Dockerfile` — same pattern
- [ ] Rewrite `rsky-jetstream-subscriber/Dockerfile` — same pattern
- [ ] Rewrite `rsky-firehose/Dockerfile` — same pattern
- [ ] Verify `docker build -f rsky-pds/Dockerfile .` succeeds locally (or note CI-only)
- [ ] Update `openspec/changes/p0-c001/tasks.md` — mark all complete
