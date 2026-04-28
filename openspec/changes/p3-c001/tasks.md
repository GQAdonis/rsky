# Tasks: p3-c001

## 1. Update top-level docs

- [ ] 1.1 Add "Operations / Storage" section to README.md naming PostgreSQL as the only supported backend
- [ ] 1.2 State explicitly that upstream `installer.sh` SQLite path does not apply
- [ ] 1.3 Add the same callout to CLAUDE.md (near the existing "Storage backends" line)

## 2. Update crate-level docs

- [ ] 2.1 Add the Postgres-only callout to `rsky-pds/README.md` Setup section
- [ ] 2.2 Cross-link from crate README to the top-level README's Operations / Storage section

## 3. Verify

- [ ] 3.1 Run `grep -r SQLite README.md CLAUDE.md rsky-pds/README.md` and confirm only intentional mentions remain (each one paired with a "not used by this fork" disclaimer)
