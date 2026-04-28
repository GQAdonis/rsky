-- Tracks consumed refresh token JTIs to prevent replay attacks.
-- When a refresh token is rotated, its JTI is recorded here.
-- A second use of the same JTI triggers session lineage revocation.
CREATE TABLE IF NOT EXISTS pds.used_refresh_token (
    id         BIGSERIAL    PRIMARY KEY,
    jti        VARCHAR      NOT NULL,
    did        VARCHAR      NOT NULL,
    expires_at VARCHAR      NOT NULL,
    created_at VARCHAR      NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
);

CREATE UNIQUE INDEX IF NOT EXISTS used_refresh_token_jti_idx
    ON pds.used_refresh_token (jti);

CREATE INDEX IF NOT EXISTS used_refresh_token_expires_at_idx
    ON pds.used_refresh_token (expires_at);
