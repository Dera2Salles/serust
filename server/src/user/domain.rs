use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_date: Option<String>,
    pub location: Option<String>,
    pub profile_pic_path: Option<String>,
}

#[allow(dead_code)]
impl User {
    pub fn new(
        id: uuid::Uuid,
        username: impl Into<String>,
        password_hash: impl Into<String>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            id,
            username: username.into(),
            password_hash: password_hash.into(),
            email: email.into(),
            first_name: None,
            last_name: None,
            birth_date: None,
            location: None,
            profile_pic_path: None,
        }
    }
}
