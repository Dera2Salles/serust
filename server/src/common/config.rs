use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSettings {
    pub default_storage_quota_gb: i64,
    pub allow_public_registration: bool,
    pub allow_public_links: bool,
    pub server_maintenance_mode: bool,
    pub max_upload_size_mb: i64,
    pub mcp_port: u16,
    pub webdav_port: u16,
    pub s3_port: u16,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_storage_quota_gb: 5,
            allow_public_registration: true,
            allow_public_links: true,
            server_maintenance_mode: false,
            max_upload_size_mb: 500,
            mcp_port: 8081,
            webdav_port: 8083,
            s3_port: 8084,
        }
    }
}

pub fn load_config() -> GlobalSettings {
    let path = "global_settings.json";
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(config) = serde_json::from_str(&data) {
            return config;
        }
    }
    
    let path_parent = "../global_settings.json";
    if let Ok(data) = fs::read_to_string(path_parent) {
        if let Ok(config) = serde_json::from_str(&data) {
            return config;
        }
    }

    GlobalSettings::default()
}
