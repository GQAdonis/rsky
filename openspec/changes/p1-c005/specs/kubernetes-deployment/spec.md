# Spec Delta: p1-c005 — kubernetes-deployment

## ADDED Requirements

### Requirement: rsky-labeler and rsky-jetstream-subscriber MUST be deployed as internal-only Deployments

Both `rsky-labeler` and `rsky-jetstream-subscriber` MUST be deployed as `Deployment` resources in the `atproto` namespace. They are internal-only consumers of the firehose and MUST NOT be exposed via a public `Gateway` or `HTTPRoute`.

#### Scenario: Labeler runs without public ingress

- **WHEN** the rsky-labeler `Deployment` is applied
- **THEN** `kubectl get gateway,httproute -n atproto -l app=rsky-labeler` returns no resources, and the labeler still receives firehose events from `rsky-relay` over the internal `Service`

#### Scenario: Jetstream subscriber runs without public ingress

- **WHEN** the rsky-jetstream-subscriber `Deployment` is applied
- **THEN** `kubectl get gateway,httproute -n atproto -l app=rsky-jetstream-subscriber` returns no resources, and the subscriber processes events delivered over the internal `Service`
