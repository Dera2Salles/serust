
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY, 
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    first_name TEXT,
    last_name TEXT,
    birth_date TEXT,
    location TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    storage_quota_bytes INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS admins (
    user_id TEXT PRIMARY KEY,
    access_level TEXT NOT NULL DEFAULT 'standard',
    last_action_at DATETIME,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY, 
    owner_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    mime_type TEXT,
    checksum TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
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
    can_read BOOLEAN DEFAULT 1,
    can_write BOOLEAN DEFAULT 0,
    can_reshare BOOLEAN DEFAULT 0,
    max_reads INTEGER, 
    expires_at DATETIME, 
    password_hash TEXT, 
    is_active BOOLEAN DEFAULT 1,
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
    max_reads INTEGER,
    expires_at DATETIME,
    granted_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(granted_by) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(granted_to) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id TEXT NOT NULL,
    accessed_by TEXT, 
    share_link_id TEXT, 
    grant_id TEXT, 
    action TEXT CHECK(action IN ('read', 'write', 'share')),
    accessed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT,
    bytes_transferred INTEGER,
    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY(accessed_by) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY(share_link_id) REFERENCES share_links(id) ON DELETE SET NULL,
    FOREIGN KEY(grant_id) REFERENCES share_grants(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS read_counters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    share_link_id TEXT UNIQUE, 
    grant_id TEXT UNIQUE, 
    read_count INTEGER NOT NULL DEFAULT 0,
    last_read_at DATETIME,
    is_exhausted BOOLEAN NOT NULL DEFAULT 0,
    refreshed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (share_link_id IS NOT NULL AND grant_id IS NULL) OR
        (share_link_id IS NULL AND grant_id IS NOT NULL)
    )
);

CREATE TRIGGER IF NOT EXISTS trg_update_read_counters_link
AFTER INSERT ON access_log
WHEN NEW.action = 'read' AND NEW.share_link_id IS NOT NULL
BEGIN
    INSERT INTO read_counters (share_link_id, read_count, last_read_at, is_exhausted)
    VALUES (
        NEW.share_link_id, 
        1, 
        NEW.accessed_at,
        (SELECT CASE WHEN max_reads IS NOT NULL AND 1 >= max_reads THEN 1 ELSE 0 END FROM share_links WHERE id = NEW.share_link_id)
    )
    ON CONFLICT (share_link_id) DO UPDATE SET
        read_count = read_counters.read_count + 1,
        last_read_at = NEW.accessed_at,
        is_exhausted = (SELECT CASE WHEN max_reads IS NOT NULL AND read_counters.read_count + 1 >= max_reads THEN 1 ELSE 0 END FROM share_links WHERE id = NEW.share_link_id);
END;

CREATE TRIGGER IF NOT EXISTS trg_update_read_counters_grant
AFTER INSERT ON access_log
WHEN NEW.action = 'read' AND NEW.grant_id IS NOT NULL
BEGIN
    INSERT INTO read_counters (grant_id, read_count, last_read_at, is_exhausted)
    VALUES (
        NEW.grant_id, 
        1, 
        NEW.accessed_at,
        (SELECT CASE WHEN max_reads IS NOT NULL AND 1 >= max_reads THEN 1 ELSE 0 END FROM share_grants WHERE id = NEW.grant_id)
    )
    ON CONFLICT (grant_id) DO UPDATE SET
        read_count = read_counters.read_count + 1,
        last_read_at = NEW.accessed_at,
        is_exhausted = (SELECT CASE WHEN max_reads IS NOT NULL AND read_counters.read_count + 1 >= max_reads THEN 1 ELSE 0 END FROM share_grants WHERE id = NEW.grant_id);
END;

CREATE VIEW IF NOT EXISTS v_effective_permissions AS
    SELECT 
        f.id AS file_id,
        f.owner_id AS user_id,
        'owner' AS source,
        1 AS can_read,
        1 AS can_write,
        1 AS can_reshare,
        NULL AS reads_remaining,
        0 AS is_expired,
        1 AS is_valid,
        NULL AS grant_id,
        NULL AS link_id
    FROM files f
    UNION ALL
    SELECT 
        g.file_id,
        g.granted_to AS user_id,
        'grant' AS source,
        g.can_read,
        g.can_write,
        g.can_reshare,
        CASE WHEN g.max_reads IS NULL THEN NULL ELSE g.max_reads - IFNULL(rc.read_count, 0) END AS reads_remaining,
        CASE WHEN g.expires_at IS NOT NULL AND g.expires_at < CURRENT_TIMESTAMP THEN 1 ELSE 0 END AS is_expired,
        CASE WHEN (g.expires_at IS NOT NULL AND g.expires_at < CURRENT_TIMESTAMP) OR (IFNULL(rc.is_exhausted, 0) = 1) THEN 0 ELSE 1 END AS is_valid,
        g.id AS grant_id,
        NULL AS link_id
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
        CASE WHEN l.max_reads IS NULL THEN NULL ELSE l.max_reads - IFNULL(rc.read_count, 0) END AS reads_remaining,
        CASE WHEN l.expires_at IS NOT NULL AND l.expires_at < CURRENT_TIMESTAMP THEN 1 ELSE 0 END AS is_expired,
        CASE WHEN l.is_active = 0 OR (l.expires_at IS NOT NULL AND l.expires_at < CURRENT_TIMESTAMP) OR (IFNULL(rc.is_exhausted, 0) = 1) THEN 0 ELSE 1 END AS is_valid,
        NULL AS grant_id,
        l.id AS link_id
    FROM share_links l
    LEFT JOIN read_counters rc ON rc.share_link_id = l.id;
