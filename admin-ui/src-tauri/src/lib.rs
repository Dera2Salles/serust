use sea_orm::{entity::*, query::*, Database as SeaDatabase, DatabaseConnection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use sysinfo::{Disks, System};
use tauri::State;
use tcp_file_server::database::entities::{prelude::*, users};
use tokio::sync::Mutex;

struct ServerState {
    handle: Option<tokio::task::JoinHandle<()>>,
    sys: System,
    db: Option<DatabaseConnection>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            handle: None,
            sys: System::new_all(),
            db: None,
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

async fn get_db(state: &State<'_, Arc<Mutex<ServerState>>>) -> Result<DatabaseConnection, String> {
    let mut state_lock = state.lock().await;
    if let Some(db) = &state_lock.db {
        return Ok(db.clone());
    }
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://aro:aropasssecret@localhost:5432/arodb".to_string());
    let db = SeaDatabase::connect(&db_url)
        .await
        .map_err(|e| e.to_string())?;
    state_lock.db = Some(db.clone());
    Ok(db)
}

#[tauri::command]
async fn start_server(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let mut state = state.lock().await;
    if state.handle.is_some() {
        return Err("Server is already running".into());
    }

    let handle = tokio::spawn(async move {
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
async fn stop_server(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<String, String> {
    let mut state = state.lock().await;
    if let Some(handle) = state.handle.take() {
        handle.abort();
        Ok("Server stopped (aborted)".into())
    } else {
        Err("Server is not running".into())
    }
}

#[tauri::command]
async fn get_server_status(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<bool, String> {
    let state = state.lock().await;
    Ok(state.handle.is_some())
}

#[tauri::command]
async fn get_system_info(state: State<'_, Arc<Mutex<ServerState>>>) -> Result<SystemInfo, String> {
    let mut state = state.lock().await;
    state.sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let mut total_disk = 0;
    let mut used_disk = 0;
    for disk in &disks {
        total_disk += disk.total_space();
        used_disk += disk.total_space() - disk.available_space();
    }

    Ok(SystemInfo {
        total_disk,
        used_disk,
        os_name: System::name().unwrap_or_else(|| "Unknown".into()),
        cpu_usage: state.sys.global_cpu_usage(),
        memory_usage: state.sys.used_memory(),
    })
}

#[tauri::command]
fn read_server_logs() -> Result<String, String> {
    std::fs::read_to_string("../../server.log").map_err(|e| format!("Failed to read logs: {}", e))
}

#[tauri::command]
async fn get_users_from_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
) -> Result<Vec<Value>, String> {
    let db = get_db(&state).await?;

    let rows = db
        .query_all(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT u.id, u.username, u.email, u.is_active, u.storage_quota_bytes, u.created_at, \
         COALESCE(SUM(f.size_bytes), 0)::BIGINT as storage_used_bytes \
         FROM users u \
         LEFT JOIN files f ON f.owner_id = u.id AND f.is_deleted = false \
         GROUP BY u.id, u.username, u.email, u.is_active, u.storage_quota_bytes, u.created_at"
                .to_string(),
        ))
        .await
        .map_err(|e| e.to_string())?;

    let mut users = Vec::new();
    for row in rows {
        let created_at: chrono::DateTime<chrono::FixedOffset> =
            row.try_get("", "created_at").map_err(|e| e.to_string())?;
        users.push(serde_json::json!({
            "id": row.try_get::<String>("", "id").map_err(|e| e.to_string())?,
            "username": row.try_get::<String>("", "username").map_err(|e| e.to_string())?,
            "email": row.try_get::<String>("", "email").map_err(|e| e.to_string())?,
            "is_active": row.try_get::<bool>("", "is_active").map_err(|e| e.to_string())?,
            "storage_quota_bytes": row.try_get::<i64>("", "storage_quota_bytes").map_err(|e| e.to_string())?,
            "storage_used_bytes": row.try_get::<i64>("", "storage_used_bytes").map_err(|e| e.to_string())?,
            "created_at": created_at.to_rfc3339(),
        }));
    }

    Ok(users)
}

#[tauri::command]
async fn create_user_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    username: String,
    email: String,
    password_raw: String,
    quota: i64,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let db = get_db(&state).await?;

    let mut hasher = Sha256::new();
    hasher.update(password_raw.as_bytes());
    let password_hash = hex::encode(hasher.finalize());
    let id = uuid::Uuid::new_v4().to_string();

    let new_user = users::ActiveModel {
        id: Set(id),
        username: Set(username),
        password_hash: Set(password_hash),
        email: Set(email),
        storage_quota_bytes: Set(quota),
        is_active: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    new_user.insert(&db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_user_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    id: String,
    email: String,
    quota: i64,
    is_active: bool,
) -> Result<(), String> {
    let db = get_db(&state).await?;

    let user = Users::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(user) = user {
        let mut user: users::ActiveModel = user.into();
        user.email = Set(email);
        user.storage_quota_bytes = Set(quota);
        user.is_active = Set(is_active);
        user.update(&db).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("User not found".into())
    }
}

#[tauri::command]
async fn delete_user_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    id: String,
) -> Result<(), String> {
    let db = get_db(&state).await?;
    Users::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_all_shares_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
) -> Result<Vec<Value>, String> {
    let db = get_db(&state).await?;

    let grants = db.query_all(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT sg.id, u1.username as owner, u2.username as grantee, f.filename, f.storage_path as path, \
         sg.can_read, sg.can_write, sg.expires_at \
         FROM share_grants sg \
         JOIN files f ON sg.file_id = f.id \
         JOIN users u1 ON sg.granted_by = u1.id \
         JOIN users u2 ON sg.granted_to = u2.id".to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in grants {
        let expires_at: Option<chrono::DateTime<chrono::FixedOffset>> =
            row.try_get("", "expires_at").unwrap_or(None);
        list.push(serde_json::json!({
            "id": row.try_get::<String>("", "id").map_err(|e| e.to_string())?,
            "type": "direct",
            "owner": row.try_get::<String>("", "owner").map_err(|e| e.to_string())?,
            "grantee": row.try_get::<String>("", "grantee").map_err(|e| e.to_string())?,
            "filename": row.try_get::<String>("", "filename").map_err(|e| e.to_string())?,
            "path": row.try_get::<String>("", "path").map_err(|e| e.to_string())?,
            "can_read": row.try_get::<bool>("", "can_read").map_err(|e| e.to_string())?,
            "can_write": row.try_get::<bool>("", "can_write").map_err(|e| e.to_string())?,
            "expires_at": expires_at.map(|d| d.to_rfc3339()),
        }));
    }

    let links = db.query_all(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT sl.id, u.username as owner, f.filename, f.storage_path as path, sl.token, sl.label, \
         sl.can_read, sl.can_write, sl.expires_at, sl.is_active \
         FROM share_links sl \
         JOIN files f ON sl.file_id = f.id \
         JOIN users u ON sl.created_by = u.id".to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    for row in links {
        let expires_at: Option<chrono::DateTime<chrono::FixedOffset>> =
            row.try_get("", "expires_at").unwrap_or(None);
        list.push(serde_json::json!({
            "id": row.try_get::<String>("", "id").map_err(|e| e.to_string())?,
            "type": "link",
            "owner": row.try_get::<String>("", "owner").map_err(|e| e.to_string())?,
            "grantee": "Public (Link)",
            "filename": row.try_get::<String>("", "filename").map_err(|e| e.to_string())?,
            "path": row.try_get::<String>("", "path").map_err(|e| e.to_string())?,
            "token": row.try_get::<String>("", "token").map_err(|e| e.to_string())?,
            "label": row.try_get::<Option<String>>("", "label").unwrap_or(None),
            "can_read": row.try_get::<bool>("", "can_read").map_err(|e| e.to_string())?,
            "can_write": row.try_get::<bool>("", "can_write").map_err(|e| e.to_string())?,
            "expires_at": expires_at.map(|d| d.to_rfc3339()),
            "is_active": row.try_get::<bool>("", "is_active").map_err(|e| e.to_string())?,
        }));
    }

    Ok(list)
}

#[tauri::command]
async fn revoke_share_grant_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    id: String,
) -> Result<(), String> {
    let db = get_db(&state).await?;
    ShareGrants::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn revoke_share_link_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    id: String,
) -> Result<(), String> {
    let db = get_db(&state).await?;
    ShareLinks::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn login_admin_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
    email: String,
    password_raw: String,
) -> Result<Value, String> {
    use sha2::{Digest, Sha256};
    let db = get_db(&state).await?;

    let mut hasher = Sha256::new();
    hasher.update(password_raw.as_bytes());
    let password_hash = hex::encode(hasher.finalize());

    // 1. Find user by email and password
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .filter(users::Column::PasswordHash.eq(password_hash))
        .one(&db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(u) = user {
        // 2. Check if user is an admin
        let is_admin = Admins::find_by_id(u.id.clone())
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .is_some();

        if !is_admin {
            return Err("Accès refusé : vous n'êtes pas administrateur".into());
        }

        if !u.is_active {
            return Err("Compte désactivé ou en attente d'approbation".into());
        }

        Ok(serde_json::json!({
            "id": u.id,
            "username": u.username,
            "email": u.email,
        }))
    } else {
        Err("Identifiants invalides".into())
    }
}

#[tauri::command]
async fn get_activity_logs_db(
    state: State<'_, Arc<Mutex<ServerState>>>,
) -> Result<Vec<Value>, String> {
    let db = get_db(&state).await?;

    let logs = db.query_all(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT al.id, u.username, f.filename, al.action, al.accessed_at, al.ip_address, al.bytes_transferred \
         FROM access_log al \
         LEFT JOIN users u ON al.accessed_by = u.id \
         JOIN files f ON al.file_id = f.id \
         ORDER BY al.accessed_at DESC \
         LIMIT 100".to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in logs {
        let accessed_at: chrono::DateTime<chrono::FixedOffset> =
            row.try_get("", "accessed_at").map_err(|e| e.to_string())?;
        list.push(serde_json::json!({
            "id": row.try_get::<i64>("", "id").map_err(|e| e.to_string())?,
            "username": row.try_get::<Option<String>>("", "username").unwrap_or(None),
            "filename": row.try_get::<String>("", "filename").map_err(|e| e.to_string())?,
            "action": row.try_get::<String>("", "action").map_err(|e| e.to_string())?,
            "accessed_at": accessed_at.to_rfc3339(),
            "ip_address": row.try_get::<Option<String>>("", "ip_address").unwrap_or(None),
            "bytes_transferred": row.try_get::<Option<i64>>("", "bytes_transferred").unwrap_or(None),
        }));
    }

    Ok(list)
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

#[tauri::command]
fn clear_server_logs() -> Result<(), String> {
    let path = "../../server.log";
    std::fs::write(path, "").map_err(|e| format!("Failed to clear logs: {}", e))
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
            clear_server_logs,
            get_users_from_db,
            create_user_db,
            update_user_db,
            delete_user_db,
            get_all_shares_db,
            revoke_share_grant_db,
            revoke_share_link_db,
            get_activity_logs_db,
            login_admin_db,
            get_global_settings,
            save_global_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
