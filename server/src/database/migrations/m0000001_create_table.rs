use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    first_name TEXT,
    last_name TEXT,
    birth_date TEXT,
    location TEXT,
    profile_pic_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    storage_quota_bytes BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS admins (
    user_id TEXT PRIMARY KEY,
    access_level TEXT NOT NULL DEFAULT 'standard',
    last_action_at TIMESTAMPTZ,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime_type TEXT,
    checksum TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_files_owner_path ON files (owner_id, storage_path);
CREATE INDEX IF NOT EXISTS idx_files_is_deleted ON files (is_deleted);
CREATE INDEX IF NOT EXISTS idx_files_storage_path ON files (storage_path);

CREATE TABLE IF NOT EXISTS share_links (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    created_by TEXT NOT NULL,
    token TEXT UNIQUE NOT NULL,
    label TEXT,
    can_read BOOLEAN DEFAULT TRUE,
    can_write BOOLEAN DEFAULT FALSE,
    can_reshare BOOLEAN DEFAULT FALSE,
    max_reads BIGINT,
    expires_at TIMESTAMPTZ,
    password_hash TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(created_by) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS share_grants (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    granted_by TEXT NOT NULL,
    granted_to TEXT NOT NULL,
    can_read BOOLEAN NOT NULL,
    can_write BOOLEAN NOT NULL,
    can_reshare BOOLEAN NOT NULL,
    max_reads BIGINT,
    expires_at TIMESTAMPTZ,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(granted_by) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(granted_to) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_log (
    id BIGSERIAL PRIMARY KEY,
    file_id TEXT NOT NULL,
    accessed_by TEXT,
    share_link_id TEXT,
    grant_id TEXT,
    action TEXT CHECK(action IN ('read', 'write', 'share')),
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address TEXT,
    user_agent TEXT,
    bytes_transferred BIGINT,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(accessed_by) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY(share_link_id) REFERENCES share_links(id) ON DELETE SET NULL,
    FOREIGN KEY(grant_id) REFERENCES share_grants(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS read_counters (
    id BIGSERIAL PRIMARY KEY,
    share_link_id TEXT UNIQUE,
    grant_id TEXT UNIQUE,
    read_count BIGINT NOT NULL DEFAULT 0,
    last_read_at TIMESTAMPTZ,
    is_exhausted BOOLEAN NOT NULL DEFAULT FALSE,
    refreshed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (share_link_id IS NOT NULL AND grant_id IS NULL) OR
        (share_link_id IS NULL AND grant_id IS NOT NULL)
    )
);

-- Trigger function for share links read counter
CREATE OR REPLACE FUNCTION trg_update_read_counters_link_fn()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.action = 'read' AND NEW.share_link_id IS NOT NULL THEN
        INSERT INTO read_counters (share_link_id, read_count, last_read_at, is_exhausted)
        VALUES (
            NEW.share_link_id,
            1,
            NEW.accessed_at,
            (SELECT CASE WHEN max_reads IS NOT NULL AND 1 >= max_reads THEN TRUE ELSE FALSE END FROM share_links WHERE id = NEW.share_link_id)
        )
        ON CONFLICT (share_link_id) DO UPDATE SET
            read_count = read_counters.read_count + 1,
            last_read_at = NEW.accessed_at,
            is_exhausted = (SELECT CASE WHEN max_reads IS NOT NULL AND read_counters.read_count + 1 >= max_reads THEN TRUE ELSE FALSE END FROM share_links WHERE id = NEW.share_link_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_update_read_counters_link
AFTER INSERT ON access_log
FOR EACH ROW EXECUTE FUNCTION trg_update_read_counters_link_fn();

-- Trigger function for share grants read counter
CREATE OR REPLACE FUNCTION trg_update_read_counters_grant_fn()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.action = 'read' AND NEW.grant_id IS NOT NULL THEN
        INSERT INTO read_counters (grant_id, read_count, last_read_at, is_exhausted)
        VALUES (
            NEW.grant_id,
            1,
            NEW.accessed_at,
            (SELECT CASE WHEN max_reads IS NOT NULL AND 1 >= max_reads THEN TRUE ELSE FALSE END FROM share_grants WHERE id = NEW.grant_id)
        )
        ON CONFLICT (grant_id) DO UPDATE SET
            read_count = read_counters.read_count + 1,
            last_read_at = NEW.accessed_at,
            is_exhausted = (SELECT CASE WHEN max_reads IS NOT NULL AND read_counters.read_count + 1 >= max_reads THEN TRUE ELSE FALSE END FROM share_grants WHERE id = NEW.grant_id);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_update_read_counters_grant
AFTER INSERT ON access_log
FOR EACH ROW EXECUTE FUNCTION trg_update_read_counters_grant_fn();

CREATE OR REPLACE VIEW v_effective_permissions AS
    SELECT
        f.id AS file_id,
        f.owner_id AS user_id,
        'owner' AS source,
        TRUE AS can_read,
        TRUE AS can_write,
        TRUE AS can_reshare,
        NULL::BIGINT AS reads_remaining,
        FALSE AS is_expired,
        TRUE AS is_valid,
        NULL::TEXT AS grant_id,
        NULL::TEXT AS link_id
    FROM files f
    UNION ALL
    SELECT
        g.file_id,
        g.granted_to AS user_id,
        'grant' AS source,
        g.can_read,
        g.can_write,
        g.can_reshare,
        CASE WHEN g.max_reads IS NULL THEN NULL ELSE g.max_reads - COALESCE(rc.read_count, 0) END AS reads_remaining,
        CASE WHEN g.expires_at IS NOT NULL AND g.expires_at < NOW() THEN TRUE ELSE FALSE END AS is_expired,
        CASE WHEN (g.expires_at IS NOT NULL AND g.expires_at < NOW()) OR (COALESCE(rc.is_exhausted, FALSE) = TRUE) THEN FALSE ELSE TRUE END AS is_valid,
        g.id AS grant_id,
        NULL::TEXT AS link_id
    FROM share_grants g
    LEFT JOIN read_counters rc ON rc.grant_id = g.id
    UNION ALL
    SELECT
        l.file_id,
        NULL AS user_id,
        'link' AS source,
        l.can_read,
        l.can_write,
        l.can_reshare,
        CASE WHEN l.max_reads IS NULL THEN NULL ELSE l.max_reads - COALESCE(rc.read_count, 0) END AS reads_remaining,
        CASE WHEN l.expires_at IS NOT NULL AND l.expires_at < NOW() THEN TRUE ELSE FALSE END AS is_expired,
        CASE WHEN l.is_active = FALSE OR (l.expires_at IS NOT NULL AND l.expires_at < NOW()) OR (COALESCE(rc.is_exhausted, FALSE) = TRUE) THEN FALSE ELSE TRUE END AS is_valid,
        NULL::TEXT AS grant_id,
        l.id AS link_id
    FROM share_links l
    LEFT JOIN read_counters rc ON rc.share_link_id = l.id;
                "#,
            )
            .await
            .map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP VIEW IF EXISTS v_effective_permissions;
DROP TRIGGER IF EXISTS trg_update_read_counters_grant ON access_log;
DROP FUNCTION IF EXISTS trg_update_read_counters_grant_fn;
DROP TRIGGER IF EXISTS trg_update_read_counters_link ON access_log;
DROP FUNCTION IF EXISTS trg_update_read_counters_link_fn;
DROP TABLE IF EXISTS read_counters;
DROP TABLE IF EXISTS access_log;
DROP TABLE IF EXISTS share_grants;
DROP TABLE IF EXISTS share_links;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS admins;
DROP TABLE IF EXISTS users;
                "#,
            )
            .await
            .map(|_| ())
    }
}