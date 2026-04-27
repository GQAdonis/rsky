# Tasks: p2-c005 (human)

- [ ] Confirm p2-c003 (all 17 secrets seeded) and p2-c004 (ArgoCD application applied) are complete
- [ ] Run `git push origin main`
- [ ] Monitor GitHub Actions — all 4 jobs must pass
- [ ] Run `kubectl get pods -n atproto` — confirm pods are Running or Pending (not CrashLoopBackOff)
- [ ] Run `git pull origin main` — confirm SHA tags were committed back by CI
