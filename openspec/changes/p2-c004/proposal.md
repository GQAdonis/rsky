# p2-c004: Bootstrap ArgoCD Application

**Phase**: phase-2-commit-and-deploy  
**Priority**: 4 (depends on p2-c002; human task; can run in parallel with p2-c003)  
**Assigned to**: human  
**Type**: operational runbook

## Overview

The ArgoCD `Application` resource in `k8s/argocd/application.yaml` must be applied manually once.
ArgoCD will not sync the atproto stack until this object exists in the cluster.

## Command

```bash
# Authenticate to GKE (use same SA as CI)
gcloud container clusters get-credentials client-cluster \
  --region us-central1 \
  --project <GKE_PROJECT_ID>

# Apply the ArgoCD Application
kubectl apply -f k8s/argocd/application.yaml
```

## Verification

```bash
# Check Application was created
kubectl get application atproto-stack -n argocd

# Check ArgoCD starts syncing (may be OutOfSync initially — that is expected)
kubectl get application atproto-stack -n argocd -o jsonpath='{.status.sync.status}'
```

## Notes

- ArgoCD will attempt to sync as soon as the Application is applied
- The first sync may fail if secrets are not yet injected (p2-c003)
- Complete p2-c003 and p2-c005 (first push) to bring ArgoCD to Healthy state
- `automated.prune: true` means ArgoCD will delete k8s resources removed from git
