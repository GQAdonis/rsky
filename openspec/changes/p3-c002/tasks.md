# Tasks: p3-c002

## 1. De-stub panic points

- [ ] 1.1 Implement `reserveSigningKey` handler at `rsky-pds/src/apis/com/atproto/server/reserve_signing_key.rs`
- [ ] 1.2 Implement the two `unimplemented!()` branches at `rsky-pds/src/apis/com/atproto/admin/get_invite_codes.rs:294, 327`
- [ ] 1.3 Replace `todo!()` at `rsky-pds/src/apis/com/atproto/server/mod.rs:160` with the real helper logic
- [ ] 1.4 Replace `todo!()` at `rsky-pds/src/db/mod.rs:25` with the real DB helper logic
- [ ] 1.5 Add unit tests for each formerly-stubbed handler

## 2. did:web support

- [ ] 2.1 Replace `bail!("Not yet supporting did:web")` at `rsky-pds/src/apis/com/atproto/server/mod.rs:115` with a working `did:web` resolver path
- [ ] 2.2 Wire `did:web` through `assert_valid_did_documents_for_service` and `assert_valid_doc_contents` so service identity checks pass for `did:web` operators
- [ ] 2.3 Add an integration test that creates a `did:web:example.com` account end-to-end (fixture HTTP server)

## 3. Upload size limits

- [ ] 3.1 Raise image upload limit to 2 MB in the Rocket multipart configuration (matching upstream PR #4823)
- [ ] 3.2 Raise video upload limit to 100 MB (matching upstream PR #3602)
- [ ] 3.3 Confirm Rocket `limits` config + `uploadBlob` accept both new ceilings without truncating

## 4. getBlob Content-Disposition

- [ ] 4.1 Set `Content-Disposition: attachment; filename="<cid>"` on `com.atproto.sync.getBlob` responses
- [ ] 4.2 Add an HTTP-level test asserting the header is present

## 5. Pipethrough widen

- [ ] 5.1 In `rsky-pds/src/apis/mod.rs:23`, extend the prefix match to include `tools.ozone.`
- [ ] 5.2 Add a test that calls `tools.ozone.moderation.queryStatuses` and asserts the request proxies (mock Ozone)

## 6. requestCrawl debounce

- [ ] 6.1 In `rsky-pds/src/crawlers.rs`, coalesce concurrent invocations against the same relay using a per-relay async mutex or singleflight
- [ ] 6.2 Add a unit test that issues N concurrent `requestCrawl` calls and asserts at most one outbound request

## 7. Verify

- [ ] 7.1 `cargo test --release -p rsky-pds` passes
- [ ] 7.2 `grep -rn "todo!()|unimplemented!()|Not yet supporting" rsky-pds/src/` returns 0 hits
