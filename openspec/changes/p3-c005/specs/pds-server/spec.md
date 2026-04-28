# Spec Delta: p3-c005 — pds-server

## ADDED Requirements

### Requirement: PDS MUST track upstream lexicons within one minor version

The lexicon definitions in `rsky-lexicon` MUST be kept within one minor version of upstream `bluesky-social/atproto` lexicons. A documented refresh procedure MUST exist so future bumps are routine.

#### Scenario: Lexicon refresh procedure documented

- **WHEN** an operator inspects `rsky-lexicon/README.md`
- **THEN** they find a documented procedure for syncing rsky-lexicon against upstream lexicons HEAD, including codegen invocation, consumer-reconciliation steps, and validation steps

#### Scenario: Recorded upstream version

- **WHEN** an operator inspects `rsky-lexicon/UPSTREAM_VERSION.md`
- **THEN** they find the commit SHA of the upstream lexicons that the current Rust types were generated from
