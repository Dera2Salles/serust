use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareGrant {
    pub owner: String,
    /// Path relative to owner's storage root (normalized, no leading slash).
    pub path: String,
    pub grantee: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_download: bool,
    /// If set, decremented on each successful directory listing.
    pub remaining_reads: Option<u64>,
    /// If set, decremented on each successful file write.
    pub remaining_writes: Option<u64>,
    /// If set, decremented on each successful file download.
    pub remaining_downloads: Option<u64>,
    /// If true, the grantee can SHARE this grant onward.
    pub can_reshare: bool,
    /// Who granted this access (owner or a resharer).
    pub granted_by: String,
    /// Unix timestamp in seconds. Grant is invalid after this time.
    pub expires_at: Option<u64>,
}
