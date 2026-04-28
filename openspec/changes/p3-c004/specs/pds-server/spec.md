# Spec Delta: p3-c004 — pds-server

## ADDED Requirements

### Requirement: Sequencer MUST sequence concurrent writes to the same repo in a deterministic order

Concurrent writes against the same repository MUST receive monotonically increasing sequence numbers without out-of-order gaps. The recovery / replay path MUST tolerate invalid `<nsid>/<rkey>` paths in stored commit operations and skip them rather than aborting.

#### Scenario: Concurrent applyWrites

- **WHEN** two `com.atproto.repo.applyWrites` requests for the same DID hit the PDS within milliseconds of each other
- **THEN** both succeed, both produce sequencer rows with strictly increasing `seq` values, and the firehose emits the events in `seq` order

#### Scenario: Recovery skips malformed commit op path

- **WHEN** the sequencer recovery / replay path encounters a stored commit op whose path is not a valid `<nsid>/<rkey>`
- **THEN** the path is logged at WARN level with `did`, `seq`, and offending path, and recovery continues with the next op
