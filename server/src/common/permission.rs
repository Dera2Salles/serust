use crate::user::domain::User;

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
}

pub struct PermissionChecker;

impl PermissionChecker {
    /// Returns true if `user` may access a resource owned by `file_owner`.
    ///
    /// Currently, both Read and Write access require ownership.
    /// If an admin/role system is introduced, differentiate here.
    pub fn can_access(user: &User, file_owner: &str, permission: &Permission) -> bool {
        let _ = permission; // reserved for future role-based differentiation
        user.username == file_owner
    }

    /// Accepts relative paths (no leading /) and absolute paths (starting with /).
    /// Blocks path traversal (..) and backslashes.
    pub fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.contains('\\')
    }

    /// Resolves a relative or absolute path against a cwd into a clean relative path
    /// (relative to the user's storage root, no leading slash).
    pub fn resolve_path(cwd: &str, path: &str) -> String {
        let full = if path.starts_with('/') {
            path.to_string()
        } else if cwd == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", cwd, path)
        };

        let mut parts: Vec<&str> = Vec::new();
        for segment in full.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                s => parts.push(s),
            }
        }
        parts.join("/")
    }

    /// Parses a path starting with "shared/" into (owner, inner_path).
    pub fn parse_shared(resolved: &str) -> Option<(String, String)> {
        let rest = resolved.strip_prefix("shared/")?;
        let mut parts = rest.splitn(2, '/');
        let owner = parts.next()?.to_string();
        let inner = parts.next().unwrap_or("").to_string();
        if owner.is_empty() {
            return None;
        }
        Some((owner, inner))
    }
}
