# Tasks: p2-c003 (human)

- [ ] Set `GKE_SA_KEY`, `GKE_PROJECT_ID`, `GHCR_PAT` in GitHub Secrets
- [ ] Generate `POSTGRES_USER` + `POSTGRES_PASSWORD`; set in GitHub Secrets
- [ ] Generate 3 secp256k1 keys for PDS; set `PDS_JWT_KEY_K256_PRIVATE_KEY_HEX`, `PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX`, `PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX`
- [ ] Set `PDS_ADMIN_PASS`, `PDS_MAILGUN_API_KEY`, `PDS_MAILGUN_DOMAIN`
- [ ] Create GCS bucket; generate HMAC key; set `GCS_HMAC_ACCESS_KEY`, `GCS_HMAC_SECRET_KEY`, `GCS_BUCKET_NAME`
- [ ] Set `RELAY_ADMIN_PASSWORD`, `RSKY_API_KEY`
- [ ] Set `MOD_SERVICE_DID=placeholder`, `MOD_SERVICE_EMAIL=placeholder@placeholder.invalid`, `MOD_SERVICE_PASSWORD=placeholder`
- [ ] Verify all 17 secrets appear in GitHub Secrets list
