-- PLC directory export storage.
-- Replaces the SQLite plc_directory.db file which used WAL shared-memory
-- incompatible with GKE SSD PVC (xShmMap error).

CREATE TABLE IF NOT EXISTS plc_operations (
    cid        TEXT        PRIMARY KEY,
    did        TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    nullified  BOOLEAN     NOT NULL DEFAULT FALSE,
    operation  BYTEA       NOT NULL
);

CREATE INDEX IF NOT EXISTS plc_ops_created_at
    ON plc_operations (created_at DESC);

-- Derived key cache populated from plc_operations bulk export.
-- Keyed by DID; updated on each export pass.
CREATE TABLE IF NOT EXISTS plc_keys (
    did          TEXT PRIMARY KEY,
    pds_endpoint TEXT,
    pds_key      TEXT
);
