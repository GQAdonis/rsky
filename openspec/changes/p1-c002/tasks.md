# Tasks: p1-c002

- [ ] Create `k8s/postgresql/initdb-configmap.yaml` — SQL script creating `rsky` and `rsky_feedgen` databases with pgvector enabled
- [ ] Update `k8s/postgresql/statefulset.yaml` — add `initdb-scripts` volume and volumeMount at `/docker-entrypoint-initdb.d`
