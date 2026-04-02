use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub size: u64,
    pub owner: String,
}

impl FileMetadata {
    pub fn new(filename: impl Into<String>, size: u64, owner: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            size,
            owner: owner.into(),
        }
    }
}
