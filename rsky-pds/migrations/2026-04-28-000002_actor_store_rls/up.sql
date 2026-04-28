-- Enable Row Level Security infrastructure on per-actor tables.
-- Isolation is PERMISSIVE: rows are visible when either (a) app.current_did
-- matches the row's did, OR (b) app.current_did is not set (legacy / admin path).
-- This is non-breaking — existing code that does not set app.current_did
-- continues to see all rows. Once all ActorStore call sites set the session
-- variable via `SET LOCAL app.current_did = <did>`, tighten to RESTRICTIVE.
--
-- Upstream reference: per-DID isolation pattern for Postgres-only PDS
-- (replaces the per-file SQLite isolation used by @atproto/pds upstream).

-- blob
ALTER TABLE pds.blob ENABLE ROW LEVEL SECURITY;

CREATE POLICY blob_actor_isolation ON pds.blob
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );

-- record
ALTER TABLE pds.record ENABLE ROW LEVEL SECURITY;

CREATE POLICY record_actor_isolation ON pds.record
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );

-- record_blob
ALTER TABLE pds.record_blob ENABLE ROW LEVEL SECURITY;

CREATE POLICY record_blob_actor_isolation ON pds.record_blob
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );

-- repo_block
ALTER TABLE pds.repo_block ENABLE ROW LEVEL SECURITY;

CREATE POLICY repo_block_actor_isolation ON pds.repo_block
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );

-- repo_root
ALTER TABLE pds.repo_root ENABLE ROW LEVEL SECURITY;

CREATE POLICY repo_root_actor_isolation ON pds.repo_root
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );

-- account_pref (uses "did" column)
ALTER TABLE pds.account_pref ENABLE ROW LEVEL SECURITY;

CREATE POLICY account_pref_actor_isolation ON pds.account_pref
    AS PERMISSIVE
    USING (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    )
    WITH CHECK (
        current_setting('app.current_did', true) IS NULL
        OR current_setting('app.current_did', true) = ''
        OR did = current_setting('app.current_did', true)
    );
