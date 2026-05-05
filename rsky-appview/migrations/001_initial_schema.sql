-- AppView schema migration
-- Creates all tables needed by the appview indexer and API handlers.
-- Uses the public schema (appview connects to the rsky database).

CREATE TABLE IF NOT EXISTS actor (
    did         TEXT PRIMARY KEY,
    handle      TEXT,
    "indexedAt" TEXT
);

CREATE INDEX IF NOT EXISTS actor_handle_idx ON actor (handle);

CREATE TABLE IF NOT EXISTS profile (
    creator       TEXT PRIMARY KEY REFERENCES actor(did) ON DELETE CASCADE,
    "displayName" TEXT,
    description   TEXT,
    "avatarCid"   TEXT,
    "bannerCid"   TEXT,
    "indexedAt"   TEXT
);

-- Materialized aggregate counts — updated by triggers
CREATE TABLE IF NOT EXISTS profile_agg (
    did               TEXT PRIMARY KEY REFERENCES actor(did) ON DELETE CASCADE,
    "followersCount"  BIGINT NOT NULL DEFAULT 0,
    "followsCount"    BIGINT NOT NULL DEFAULT 0,
    "postsCount"      BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS post (
    uri           TEXT PRIMARY KEY,
    cid           TEXT NOT NULL,
    creator       TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    text          TEXT NOT NULL DEFAULT '',
    "replyRoot"   TEXT,
    "replyParent" TEXT,
    "replyCount"  BIGINT NOT NULL DEFAULT 0,
    "repostCount" BIGINT NOT NULL DEFAULT 0,
    "likeCount"   BIGINT NOT NULL DEFAULT 0,
    "quoteCount"  BIGINT NOT NULL DEFAULT 0,
    "createdAt"   TEXT NOT NULL,
    "indexedAt"   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS post_agg (
    uri           TEXT PRIMARY KEY REFERENCES post(uri) ON DELETE CASCADE,
    "replyCount"  BIGINT NOT NULL DEFAULT 0,
    "repostCount" BIGINT NOT NULL DEFAULT 0,
    "likeCount"   BIGINT NOT NULL DEFAULT 0,
    "quoteCount"  BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS post_creator_idx ON post (creator, "indexedAt" DESC);
CREATE INDEX IF NOT EXISTS post_reply_parent_idx ON post ("replyParent");

CREATE TABLE IF NOT EXISTS feed_item (
    "postUri"       TEXT NOT NULL REFERENCES post(uri) ON DELETE CASCADE,
    "originatorDid" TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "sortAt"        TEXT NOT NULL,
    PRIMARY KEY ("postUri", "originatorDid")
);

CREATE INDEX IF NOT EXISTS feed_item_originator_idx ON feed_item ("originatorDid", "sortAt" DESC);

CREATE TABLE IF NOT EXISTS "like" (
    uri          TEXT PRIMARY KEY,
    creator      TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    subject      TEXT NOT NULL,
    "subjectCid" TEXT NOT NULL,
    "createdAt"  TEXT NOT NULL,
    "indexedAt"  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS like_subject_idx ON "like" (subject);
CREATE INDEX IF NOT EXISTS like_creator_idx ON "like" (creator);

CREATE TABLE IF NOT EXISTS repost (
    uri         TEXT PRIMARY KEY,
    creator     TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    subject     TEXT NOT NULL,
    "createdAt" TEXT NOT NULL,
    "indexedAt" TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS repost_subject_idx ON repost (subject);
CREATE INDEX IF NOT EXISTS repost_creator_idx ON repost (creator);

CREATE TABLE IF NOT EXISTS follow (
    uri          TEXT PRIMARY KEY,
    creator      TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "subjectDid" TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "createdAt"  TEXT NOT NULL,
    "indexedAt"  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS follow_creator_idx ON follow (creator);
CREATE INDEX IF NOT EXISTS follow_subject_idx ON follow ("subjectDid");

CREATE TABLE IF NOT EXISTS actor_block (
    creator      TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "subjectDid" TEXT NOT NULL,
    "createdAt"  TEXT NOT NULL,
    "indexedAt"  TEXT NOT NULL,
    PRIMARY KEY (creator, "subjectDid")
);

CREATE TABLE IF NOT EXISTS actor_mute (
    creator      TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "subjectDid" TEXT NOT NULL,
    "createdAt"  TEXT NOT NULL,
    PRIMARY KEY (creator, "subjectDid")
);

CREATE TABLE IF NOT EXISTS notification (
    id              BIGSERIAL PRIMARY KEY,
    did             TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    author          TEXT NOT NULL,
    "recordUri"     TEXT NOT NULL,
    "recordCid"     TEXT NOT NULL,
    reason          TEXT NOT NULL,
    "reasonSubject" TEXT,
    "isRead"        BOOLEAN NOT NULL DEFAULT FALSE,
    "sortAt"        TEXT NOT NULL,
    UNIQUE ("recordUri", did)
);

CREATE INDEX IF NOT EXISTS notification_did_idx ON notification (did, "sortAt" DESC);

CREATE TABLE IF NOT EXISTS feed_generator (
    uri          TEXT PRIMARY KEY,
    cid          TEXT NOT NULL,
    creator      TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "feedDid"    TEXT NOT NULL,
    "displayName" TEXT NOT NULL,
    description  TEXT,
    "avatarCid"  TEXT,
    "likeCount"  BIGINT NOT NULL DEFAULT 0,
    "indexedAt"  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS list (
    uri         TEXT PRIMARY KEY,
    cid         TEXT NOT NULL,
    creator     TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    purpose     TEXT NOT NULL,
    description TEXT,
    "avatarCid" TEXT,
    "indexedAt" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS list_item (
    uri          TEXT PRIMARY KEY,
    list_uri     TEXT NOT NULL REFERENCES list(uri) ON DELETE CASCADE,
    "subjectDid" TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    "indexedAt"  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS list_block (
    creator  TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    list_uri TEXT NOT NULL REFERENCES list(uri) ON DELETE CASCADE,
    PRIMARY KEY (creator, list_uri)
);

CREATE TABLE IF NOT EXISTS list_mute (
    creator  TEXT NOT NULL REFERENCES actor(did) ON DELETE CASCADE,
    list_uri TEXT NOT NULL REFERENCES list(uri) ON DELETE CASCADE,
    PRIMARY KEY (creator, list_uri)
);

-- Trigger functions to maintain profile_agg counts
CREATE OR REPLACE FUNCTION update_follower_count() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO profile_agg (did, "followersCount") VALUES (NEW."subjectDid", 1)
        ON CONFLICT (did) DO UPDATE SET "followersCount" = profile_agg."followersCount" + 1;
        INSERT INTO profile_agg (did, "followsCount") VALUES (NEW.creator, 1)
        ON CONFLICT (did) DO UPDATE SET "followsCount" = profile_agg."followsCount" + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE profile_agg SET "followersCount" = GREATEST(0, "followersCount" - 1) WHERE did = OLD."subjectDid";
        UPDATE profile_agg SET "followsCount" = GREATEST(0, "followsCount" - 1) WHERE did = OLD.creator;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS follow_count_trigger ON follow;
CREATE TRIGGER follow_count_trigger
AFTER INSERT OR DELETE ON follow
FOR EACH ROW EXECUTE FUNCTION update_follower_count();

CREATE OR REPLACE FUNCTION update_post_count() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO profile_agg (did, "postsCount") VALUES (NEW.creator, 1)
        ON CONFLICT (did) DO UPDATE SET "postsCount" = profile_agg."postsCount" + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE profile_agg SET "postsCount" = GREATEST(0, "postsCount" - 1) WHERE did = OLD.creator;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS post_count_trigger ON post;
CREATE TRIGGER post_count_trigger
AFTER INSERT OR DELETE ON post
FOR EACH ROW EXECUTE FUNCTION update_post_count();
