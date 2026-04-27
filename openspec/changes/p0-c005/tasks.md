# Tasks: p0-c005

- [ ] Create `k8s/argocd/application.yaml` (ArgoCD Application, automated sync, prune: true, recurse k8s/, exclude argocd/**)
- [ ] Create `.github/workflows/deploy.yaml`:
  - [ ] Job: `build-and-push` (matrix: rsky-pds, rsky-feedgen, rsky-labeler, rsky-jetstream-subscriber, web-client)
  - [ ] Job: `update-image-tags` (sed IMAGE_TAG in k8s manifests, git commit --no-verify "[skip ci]", git push)
  - [ ] Job: `setup-namespace` (kubectl apply namespace + GHCR pull secret in atproto namespace)
  - [ ] Job: `inject-secrets` (envsubst all secret.yaml templates, kubectl apply)
  - [ ] Job: `verify-sync` (argocd app wait atproto-stack --timeout 300)
  - [ ] Job: `print-dns` (echo Cloudflare DNS instructions)
- [ ] Add `.github/scripts/update-image-tags.sh` helper (loops services, sed replaces IMAGE_TAG)
- [ ] Document required GitHub Secrets in `k8s/README.md` (complete list from proposal)
- [ ] Verify workflow YAML is valid with `actionlint` or equivalent
