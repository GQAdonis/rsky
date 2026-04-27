# Tasks: p2-c002

- [ ] Commit 1: stage `rsky-relay/src/server/server.rs` + `rsky-relay/Dockerfile` and commit with message "feat: patch rsky-relay bind address and add Dockerfile"
- [ ] Commit 2: stage all 6 service Dockerfiles + `Dockerfile.web-client` and commit with message "feat: rewrite service Dockerfiles for containerized k8s deployment"
- [ ] Commit 3: stage `.gitignore`, `CLAUDE.md`, `k8s/`, `.github/`, `.kbd-orchestrator/`, `openspec/` and commit with message "feat: add k8s manifests, CI/CD, and KBD orchestration for atproto stack"
- [ ] Verify `git log --oneline -5` shows all 3 commits
- [ ] Verify no secret values appear in `git diff HEAD~3` (only `${VAR}` placeholders)
