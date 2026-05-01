# rsky-lexicon

Rust types for the AT Protocol [`lexicon`](https://atproto.com/guides/lexicon) — hand-authored types covering `com.atproto.*`, `app.bsky.*`, and `chat.bsky.*` namespaces.

[![Crate](https://img.shields.io/crates/v/rsky-lexicon?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44)](https://crates.io/crates/rsky-lexicon)

## Upstream tracking

This crate targets staying within **one minor version** of the upstream
[bluesky-social/atproto](https://github.com/bluesky-social/atproto) lexicon definitions.

The current upstream snapshot is recorded in [`UPSTREAM_VERSION.md`](./UPSTREAM_VERSION.md).

For the full refresh procedure (snapshot → update types → verify consumers → commit),
see [`UPSTREAM_VERSION.md`](./UPSTREAM_VERSION.md#refresh-procedure).

## License

rsky is released under the [Apache License 2.0](../LICENSE).