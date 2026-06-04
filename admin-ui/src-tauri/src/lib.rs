use std::sync::{Arc, Mutex};
use sysinfo::{Disks, System};
use serde::{Serialize, Deserialize};
use tauri::State;
use serde_json::Value;

struct ServerState {
    handle: Option<tokio::task::JoinHandle<()>>,
    sys: System,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            handle: None,
            sys: System::new_all(),
        }
    }
}

#[derive(Serialize)]
struct SystemInfo {
    total_disk: u64,
    used_disk: u64,
    os_name: String,
    cpu_usage: f32,
    memory_usage: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GlobalSettings {
    default_storage_quota_gb: i64,
    allow_public_registration: bool,
    allow_public_links: bool,
    server_maintenance_mode: bool,
    max_upload_size_mb: i64,
    mcp_port: u16,
    webdav_port: u16,
    s3_port: u16,
}

#[tauri::command]
async fn start_server(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|_| "Failed to lock state")?;
    if state.handle.is_some() {
        return Err("Server is already running".into());
    }

    let handle = tokio::spawn(async move {
        // Set environment variables for the internal server
        if std::env::var("DATABASE_URL").is_err() {
            std::env::set_var("DATABASE_URL", "sqlite:../../development.db");
        }
        if std::env::var("STORAGE_ROOT").is_err() {
            std::env::set_var("STORAGE_ROOT", "../../storage");
        }
        
        if let Err(e) = tcp_file_server::run_server().await {
            eprintln!("Internal server error: {}", e);
        }
    });

    state.handle = Some(handle);
    Ok("Server started internally".into())
}

#[tauri::command]
fn stop_server(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let mut state = state.lock().map_err(|_| "Failed to lock state")?;
    if let Some(handle) = state.handle.take() {
        handle.abort();
        Ok("Server stopped (aborted)".into())
    } else {
        Err("Server is not running".into())
    }
}

#[tauri::command]
fn get_server_status(state: State<'_, Arc<Mutex<ServerState>>>) -> bool {
    let state = state.lock().unwrap();
    state.handle.is_some()
}

#[tauri::command]
fn get_system_info(state: State<'_, Arc<Mutex<ServerState>>>) -> SystemInfo {
    let mut state = state.lock().unwrap();
    state.sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let mut total_disk = 0;
    let mut used_disk = 0;
    for disk in &disks {
        total_disk += disk.total_space();
        used_disk += disk.total_space() - disk.available_space();
    }

    SystemInfo {
        total_disk,
        used_disk,
        os_name: System::name().unwrap_or_else(|| "Unknown".into()),
        cpu_usage: state.sys.global_cpu_usage(),
        memory_usage: state.sys.used_memory(),
    }
}

#[tauri::command]
fn read_server_logs() -> Result<String, String> {
    std::fs::read_to_string("../../server.log")
        .map_err(|e| format!("Failed to read logs: {}", e))
}

#[tauri::command]
async fn get_users_from_db() -> Result<Vec<Value>, String> {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    let rows = sqlx::query(
        "SELECT u.id, u.username, u.email, u.is_active, u.storage_quota_bytes, u.created_at, \
         COALESCE(SUM(f.size_bytes), 0) as storage_used_bytes \
         FROM users u \
         LEFT JOIN files f ON f.owner_id = u.id AND f.is_deleted = 0 \
         GROUP BY u.id"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut users = Vec::new();
    for r in rows {
        users.push(serde_json::json!({
            "id": r.get::<String, _>("id"),
            "username": r.get::<String, _>("username"),
            "email": r.get::<String, _>("email"),
            "is_active": r.get::<bool, _>("is_active"),
            "storage_quota_bytes": r.get::<i64, _>("storage_quota_bytes"),
            "storage_used_bytes": r.get::<i64, _>("storage_used_bytes"),
            "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        }));
    }
    Ok(users)
}

#[tauri::command]
async fn create_user_db(username: String, email: String, password_raw: String, quota: i64) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password_raw.as_bytes());
    let password_hash = hex::encode(hasher.finalize());
    let id = uuid::Uuid::new_v4().to_string();

    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO users (id, username, password_hash, email, storage_quota_bytes, is_active) VALUES (?, ?, ?, ?, ?, 0)")
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .bind(email)
        .bind(quota)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_user_db(id: String, email: String, quota: i64, is_active: bool) -> Result<(), String> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE users SET email = ?, storage_quota_bytes = ?, is_active = ? WHERE id = ?")
        .bind(email)
        .bind(quota)
        .bind(is_active)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_user_db(id: String) -> Result<(), String> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_all_shares_db() -> Result<Vec<Value>, String> {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Row;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    let grants = sqlx::query(
        "SELECT sg.id, u1.username as owner, u2.username as grantee, f.filename, f.storage_path, sg.can_read, sg.can_write, sg.expires_at 
         FROM share_grants sg 
         JOIN files f ON sg.file_id = f.id 
         JOIN users u1 ON sg.granted_by = u1.id 
         JOIN users u2 ON sg.granted_to = u2.id"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in grants {
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = r.try_get("expires_at").unwrap_or(None);
        list.push(serde_json::json!({
            "id": r.get::<String, _>("id"),
            "type": "direct",
            "owner": r.get::<String, _>("owner"),
            "grantee": r.get::<String, _>("grantee"),
            "filename": r.get::<String, _>("filename"),
            "path": r.get::<String, _>("storage_path"),
            "can_read": r.get::<bool, _>("can_read"),
            "can_write": r.get::<bool, _>("can_write"),
            "expires_at": expires_at,
        }));
    }

    let links = sqlx::query(
        "SELECT sl.id, u.username as owner, f.filename, f.storage_path, sl.token, sl.label, sl.can_read, sl.can_write, sl.expires_at, sl.is_active 
         FROM share_links sl 
         JOIN files f ON sl.file_id = f.id 
         JOIN users u ON sl.created_by = u.id"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    for r in links {
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = r.try_get("expires_at").unwrap_or(None);
        list.push(serde_json::json!({
            "id": r.get::<String, _>("id"),
            "type": "link",
            "owner": r.get::<String, _>("owner"),
            "grantee": "Public (Link)",
            "filename": r.get::<String, _>("filename"),
            "path": r.get::<String, _>("storage_path"),
            "token": r.get::<String, _>("token"),
            "label": r.get::<Option<String>, _>("label"),
            "can_read": r.get::<bool, _>("can_read"),
            "can_write": r.get::<bool, _>("can_write"),
            "expires_at": expires_at,
            "is_active": r.get::<bool, _>("is_active"),
        }));
    }

    Ok(list)
}

#[tauri::command]
async fn revoke_share_grant_db(id: String) -> Result<(), String> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM share_grants WHERE id = ?").bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn revoke_share_link_db(id: String) -> Result<(), String> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite:../../development.db")
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM share_links WHERE id = ?").bind(id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_global_settings() -> Result<GlobalSettings, String> {
    let path = "../../global_settings.json";
    if !std::path::Path::new(path).exists() {
        let default_settings = GlobalSettings {
            default_storage_quota_gb: 5,
            allow_public_registration: true,
            allow_public_links: true,
            server_maintenance_mode: false,
            max_upload_size_mb: 500,
            mcp_port: 8081,
            webdav_port: 8083,
            s3_port: 8084,
        };
        let json = serde_json::to_string_pretty(&default_settings).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        return Ok(default_settings);
    }
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let val: GlobalSettings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(val)
}

#[tauri::command]
fn save_global_settings(settings: GlobalSettings) -> Result<(), String> {
    let path = "../../global_settings.json";
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let server_state = Arc::new(Mutex::new(ServerState::default()));

    tauri::Builder::default()
        .manage(server_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_server_status,
            get_system_info,
            read_server_logs,
            get_users_from_db,
            create_user_db,
            update_user_db,
            delete_user_db,
            get_all_shares_db,
            revoke_share_grant_db,
            revoke_share_link_db,
            get_global_settings,
            save_global_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
