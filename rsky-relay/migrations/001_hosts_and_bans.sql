-- Relay host list and ban list.
-- Replaces the SQLite relay.db file which used WAL shared-memory
-- incompatible with GKE SSD PVC (xShmMap error).

CREATE TABLE IF NOT EXISTS hosts (
    host       TEXT        PRIMARY KEY,
    cursor     BIGINT      NOT NULL DEFAULT 0,
    latest     TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

CREATE TABLE IF NOT EXISTS banned_hosts (
    host       TEXT        PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- LISTEN/NOTIFY support: fire pg_notify whenever the ban list changes so the
-- crawler manager reacts instantly instead of waiting for BAN_REFRESH_INTERVAL.
CREATE OR REPLACE FUNCTION notify_banned_hosts_changed()
    RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM pg_notify('banned_hosts_changed', OLD.host);
    ELSE
        PERFORM pg_notify('banned_hosts_changed', NEW.host);
    END IF;
    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS trg_banned_hosts_changed ON banned_hosts;
CREATE TRIGGER trg_banned_hosts_changed
    AFTER INSERT OR DELETE ON banned_hosts
    FOR EACH ROW EXECUTE FUNCTION notify_banned_hosts_changed();
