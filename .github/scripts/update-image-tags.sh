#!/usr/bin/env bash
set -euo pipefail

SHA="${1:?SHA argument required}"

SERVICES=(
  "rsky-pds"
  "rsky-relay"
  "rsky-feedgen"
  "rsky-labeler"
  "rsky-jetstream-subscriber"
  "web-client"
)

for service in "${SERVICES[@]}"; do
  dir="k8s/${service}"
  if [ ! -d "$dir" ]; then
    continue
  fi
  find "$dir" -name "*.yaml" -exec \
    sed -i "s|ghcr.io/know-me-tools/${service}:IMAGE_TAG|ghcr.io/know-me-tools/${service}:${SHA}|g" {} +
  echo "Updated ${service} → ${SHA}"
done
