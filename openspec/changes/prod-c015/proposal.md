# prod-c015: ES256K JWT Support in appview-auth

## Problem

The appview rejects all tokens issued by rsky-pds because the PDS signs JWTs with
`alg: ES256K` (secp256k1). The `jsonwebtoken` crate's `Algorithm` enum doesn't include
ES256K, so header parsing fails before `insecure_disable_signature_validation()` can
even suppress the signature check.

Error seen: `unknown variant 'ES256K', expected one of 'HS256', 'HS384' ...`

This blocks all authenticated appview endpoints (getPreferences, getTimeline, etc.)
and is the primary cause of the Ouranos web client login failure.

## Solution

Bypass `jsonwebtoken::decode` entirely. Manually base64url-decode the JWT payload
segment and deserialize it into `Claims`. Validate `exp` only. This is correct AT
Protocol behavior: an appview trusts PDS-issued tokens by DID key resolution, not
by shared secrets. Signature validation is intentionally skipped (as the original
`insecure_disable_signature_validation()` already expressed).

## Files Changed

- `rsky-appview/crates/appview-auth/src/lib.rs`
- `rsky-appview/crates/appview-auth/Cargo.toml` (add `base64` dep if missing)
