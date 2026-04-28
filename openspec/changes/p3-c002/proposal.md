# p3-c002: Low-effort PDS hardening sweep

## Why

The assessment surfaced a cluster of small, independent gaps that share a common shape: each is a one-to-three-file fix with high ratio of correctness improvement to engineering risk. Bundling them lets us land them in a single review pass rather than spinning up six tiny changes.

## What Changes

- **De-stub five panic points:**
  - `rsky-pds/src/apis/com/atproto/server/reserve_signing_key.rs:3` — implement `reserveSigningKey` properly (used during account migration).
  - `rsky-pds/src/apis/com/atproto/admin/get_invite_codes.rs:294, 327` — implement the two `unimplemented!()` paths.
  - `rsky-pds/src/apis/com/atproto/server/mod.rs:160` — replace `todo!()` in the helper with the real implementation.
  - `rsky-pds/src/db/mod.rs:25` — replace `todo!()` with the real implementation.
- **`did:web` support:** replace `bail!("Not yet supporting did:web")` at `apis/com/atproto/server/mod.rs:115` with a working DID-web resolution path through HTTP fetch + verification.
- **Upload size limits:** raise image upload limit to 2 MB and video upload limit to 100 MB in the relevant Rocket / Multipart config.
- **`getBlob` Content-Disposition:** set a download-style Content-Disposition header on the `com.atproto.sync.getBlob` response.
- **Pipethrough catchall:** widen the prefix match in `apis/mod.rs:23` to also include `tools.ozone.` so moderator-tooling requests proxy correctly.
- **`requestCrawl` debounce:** wrap the crawler-notify codepath so concurrent invocations against the same relay coalesce, matching upstream PR #4408.

## Capabilities

### Modified Capabilities

- `pds-server`: adds requirements for `did:web` support, upload size limits, `getBlob` Content-Disposition, ozone proxying, and `requestCrawl` debounce.

## Impact

- Touches several files in `rsky-pds/src/apis/com/atproto/` and one in `src/db/`.
- Removes runtime panic surfaces.
- No new dependencies expected.
