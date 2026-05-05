# Tasks: prod-c018

- [x] Add `app.bsky.feed.repost` variant to `Lexicon` enum in `rsky-feedgen/src/models/create_request.rs`
- [x] Add catch-all `Unknown(serde_json::Value)` variant to `CreateRecord` enum to prevent panics on future unknown record types
- [x] Run `cargo check -p rsky-feedgen` to verify compilation
