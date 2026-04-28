# Spec Delta: p3-c008 — pds-server

## ADDED Requirements

### Requirement: Federation conformance harness MUST exist and MUST pass

The repository MUST include a conformance harness that runs rsky-pds and `@atproto/pds@0.4.220` side by side, applies a canonical write workload to each, and diffs (a) the firehose frames on lexicon-defined fields, (b) the `getRepo` CAR bytes, and (c) the per-record `getRecord` payloads. The harness MUST be runnable from CI and MUST fail the workflow on any diff.

#### Scenario: Side-by-side conformance pass

- **WHEN** the conformance harness runs the canonical workload on a fresh checkout
- **THEN** it reports zero diffs on lexicon-defined fields of `subscribeRepos` frames, byte-identical CAR output for `getRepo`, and byte-identical record payloads for `getRecord` across the two PDS implementations

#### Scenario: Conformance regression fails CI

- **WHEN** a PR introduces a change that produces a different firehose frame shape from upstream
- **THEN** the conformance workflow fails, the captured frames are uploaded as artifacts, and the PR cannot merge until the diff is resolved
