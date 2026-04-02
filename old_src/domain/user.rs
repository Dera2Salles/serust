use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

impl User {
    pub fn new(username: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password_hash: password_hash.into(),
        }
    }
}
