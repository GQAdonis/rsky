-- OAuth provider schema for rsky-pds.
-- Ports account-device.ts, authorized-client.ts, and related tables from
-- @atproto/pds account-manager/ OAuth additions (post-0.4.107).

-- oauth_client: registered OAuth clients
CREATE TABLE IF NOT EXISTS pds.oauth_client (
    client_id           VARCHAR     PRIMARY KEY,
    client_secret       VARCHAR,
    redirect_uris       TEXT        NOT NULL,   -- JSON array
    scope               VARCHAR     NOT NULL DEFAULT 'atproto',
    grant_types         TEXT        NOT NULL DEFAULT '["authorization_code","refresh_token"]',
    response_types      TEXT        NOT NULL DEFAULT '["code"]',
    token_endpoint_auth_method VARCHAR NOT NULL DEFAULT 'none',
    client_name         VARCHAR,
    client_uri          VARCHAR,
    logo_uri            VARCHAR,
    policy_uri          VARCHAR,
    tos_uri             VARCHAR,
    created_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    updated_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
);

-- oauth_device: tracks authorization devices/sessions per DID
CREATE TABLE IF NOT EXISTS pds.oauth_device (
    id                  VARCHAR     PRIMARY KEY,
    did                 VARCHAR     NOT NULL REFERENCES pds.actor(did) ON DELETE CASCADE,
    client_id           VARCHAR     NOT NULL,
    scope               VARCHAR     NOT NULL,
    code_challenge      VARCHAR,
    code_challenge_method VARCHAR,
    redirect_uri        VARCHAR,
    state               VARCHAR,
    created_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    expires_at          VARCHAR     NOT NULL,
    authorized          BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS oauth_device_did_idx ON pds.oauth_device (did);
CREATE INDEX IF NOT EXISTS oauth_device_expires_at_idx ON pds.oauth_device (expires_at);

-- oauth_authorized_client: long-lived OAuth consents
CREATE TABLE IF NOT EXISTS pds.oauth_authorized_client (
    id                  BIGSERIAL   PRIMARY KEY,
    did                 VARCHAR     NOT NULL REFERENCES pds.actor(did) ON DELETE CASCADE,
    client_id           VARCHAR     NOT NULL,
    scope               VARCHAR     NOT NULL,
    created_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    updated_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    UNIQUE (did, client_id)
);

CREATE INDEX IF NOT EXISTS oauth_authorized_client_did_idx ON pds.oauth_authorized_client (did);

-- oauth_token: issued access/refresh tokens for OAuth flows
CREATE TABLE IF NOT EXISTS pds.oauth_token (
    id                  BIGSERIAL   PRIMARY KEY,
    did                 VARCHAR     NOT NULL,
    client_id           VARCHAR     NOT NULL,
    scope               VARCHAR     NOT NULL,
    access_token_jti    VARCHAR     NOT NULL UNIQUE,
    refresh_token_jti   VARCHAR     UNIQUE,
    access_expires_at   VARCHAR     NOT NULL,
    refresh_expires_at  VARCHAR,
    created_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
    revoked_at          VARCHAR
);

CREATE INDEX IF NOT EXISTS oauth_token_did_idx ON pds.oauth_token (did);
CREATE INDEX IF NOT EXISTS oauth_token_access_expires_at_idx ON pds.oauth_token (access_expires_at);

-- oauth_par_request: Pushed Authorization Requests (PAR) store
CREATE TABLE IF NOT EXISTS pds.oauth_par_request (
    request_uri         VARCHAR     PRIMARY KEY,
    client_id           VARCHAR     NOT NULL,
    request_params      TEXT        NOT NULL,   -- JSON
    expires_at          VARCHAR     NOT NULL,
    created_at          VARCHAR     NOT NULL DEFAULT TO_CHAR(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
);

CREATE INDEX IF NOT EXISTS oauth_par_request_expires_at_idx ON pds.oauth_par_request (expires_at);
