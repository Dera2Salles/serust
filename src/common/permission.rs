use crate::user::domain::User;

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
}

pub struct PermissionChecker;

impl PermissionChecker {
    pub fn can_access(user: &User, file_owner: &str, _permission: &Permission) -> bool {
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
}
