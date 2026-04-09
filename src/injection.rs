use crate::database::{
    domain::{DbAccessLog, DbFileMetadata, DbShareGrant, DbShareLink, DbUser},
    AccessLogRepository as DbAccessLogRepository, Database,
    FileDatabaseRepository as DbFileRepository, IAccessLogRepository,
    IFileDatabaseRepository, IShareDatabaseRepository, IUserRepository,
    ShareDatabaseRepository as DbShareRepository, UserDatabaseRepository as DbUserRepository,
};
use crate::file::local_repository::FileRepository;
use crate::file::service::FileService;
use crate::share::local_repository::ShareRepository;
use crate::share::service::ShareService;
use crate::user::service::AuthService;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

pub struct Services {
    pub auth_service: Arc<AuthService>,
    pub file_service: Arc<FileService>,
    pub share_service: Arc<ShareService>,
}

pub async fn setup_injection() -> Result<Services> {
    let file_repo = Arc::new(FileRepository::new("storage"));
    let share_repo = Arc::new(ShareRepository::new("shares.json").await);

    info!("Initialisation de la base de données SQLite...");
    let db = Database::new("sqlite:development.db").await?;
    let db_user_repo = Arc::new(DbUserRepository::new(db.clone()));
    let db_file_repo = Arc::new(DbFileRepository::new(db.clone()));
    let db_share_repo = DbShareRepository::new(db.clone());
    let db_log_repo = DbAccessLogRepository::new(db.clone());

    let auth_service = Arc::new(AuthService::new(Arc::clone(&db_user_repo)));
    let share_service = Arc::new(ShareService::new(Arc::clone(&share_repo)));

    // File Use Cases
    let download_usecase = Arc::new(crate::file::DownloadUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
    ));
    let upload_usecase = Arc::new(crate::file::UploadUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
        Arc::clone(&db_file_repo),
        Arc::clone(&db_user_repo),
    ));
    let list_usecase = Arc::new(crate::file::ListUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
    ));
    let mkdir_usecase = Arc::new(crate::file::MkdirUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
        Arc::clone(&db_file_repo),
        Arc::clone(&db_user_repo),
    ));
    let delete_usecase = Arc::new(crate::file::DeleteUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
    ));
    let stat_usecase = Arc::new(crate::file::StatUseCase::new(
        Arc::clone(&file_repo),
        Arc::clone(&share_service),
    ));
    let rename_usecase = Arc::new(crate::file::RenameUseCase::new(
        Arc::clone(&file_repo),
    ));
    let rmdir_usecase = Arc::new(crate::file::RemoveDirUseCase::new(
        Arc::clone(&file_repo),
    ));
    let dir_exists_usecase = Arc::new(crate::file::DirExistsUseCase::new(
        Arc::clone(&file_repo),
    ));

    let file_service = Arc::new(FileService::new(
        download_usecase,
        upload_usecase,
        list_usecase,
        mkdir_usecase,
        delete_usecase,
        stat_usecase,
        rename_usecase,
        rmdir_usecase,
        dir_exists_usecase,
    ));



    for (name, pass) in [
        ("alice", "alice123"),
        ("bob", "bob456"),
        ("carol", "carol789"),
        ("testuser", "password123"),
        ("ayanokoji", "mastermind"),
        ("developer", "dev123"),
        ("dera", "dera123"),
    ] {
        let _ = auth_service.register(name, pass).await;
        info!("Utilisateur prêt : {}", name);
    }

    let admin_username = "admin_dev";
    if db_user_repo
        .find_by_username(admin_username)
        .await?
        .is_none()
    {
        let dev_user = DbUser {
            id: uuid::Uuid::new_v4(),
            username: admin_username.to_string(),
            password_hash: AuthService::hash_password("admin123"),
            email: "admin@local".to_string(),
            created_at: chrono::Utc::now(),
            storage_quota_bytes: 1048576,
            is_active: true,
        };
        db_user_repo.create(&dev_user).await?;

        let dev_file = DbFileMetadata {
            id: uuid::Uuid::new_v4(),
            owner_id: dev_user.id,
            filename: "welcome.txt".into(),
            storage_path: "/welcome.txt".into(),
            size_bytes: 12,
            mime_type: Some("text/plain".into()),
            checksum: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_deleted: false,
        };
        db_file_repo.create(&dev_file).await?;
        let _ = db_file_repo.find_by_id(dev_file.id).await;

        let dev_link = DbShareLink {
            id: uuid::Uuid::new_v4(),
            file_id: dev_file.id,
            created_by: dev_user.id,
            token: "demo_token".into(),
            label: None,
            can_read: true,
            can_write: false,
            can_reshare: false,
            max_reads: None,
            expires_at: None,
            password_hash: None,
            is_active: true,
        };
        db_share_repo.create_link(&dev_link).await?;

        let dev_grant = DbShareGrant {
            id: uuid::Uuid::new_v4(),
            file_id: dev_file.id,
            granted_by: dev_user.id,
            granted_to: dev_user.id,
            can_read: true,
            can_write: true,
            can_reshare: true,
            max_reads: None,
            expires_at: None,
            granted_at: chrono::Utc::now(),
        };
        db_share_repo.create_grant(&dev_grant).await?;

        let dev_log = DbAccessLog {
            id: 0,
            file_id: dev_file.id,
            accessed_by: Some(dev_user.id),
            share_link_id: None,
            grant_id: None,
            action: "read".into(),
            accessed_at: chrono::Utc::now(),
            ip_address: None,
            user_agent: None,
            bytes_transferred: None,
        };
        db_log_repo.create(&dev_log).await?;

        info!("Données SQLite de développement injectées avec succès.");
    }

    Ok(Services {
        auth_service,
        file_service,
        share_service,
    })
}
