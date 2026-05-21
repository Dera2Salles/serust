use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password_hash: String,
}

#[allow(dead_code)]
impl User {
    pub fn new(id: uuid::Uuid, username: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            id,
            username: username.into(),
            password_hash: password_hash.into(),
        }
    }
}
