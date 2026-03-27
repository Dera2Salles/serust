use crate::domain::user::User;

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

    pub fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.starts_with('/') && !path.contains('\\')
    }
}
