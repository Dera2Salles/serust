use crate::database::{
    domain::{DbAccessLog, DbFileMetadata, DbShareGrant, DbShareLink, DbUser},
    access_log_repository::AccessLogRepository as DbAccessLogRepository,
    file_repository::FileRepository as DbFileRepository,
    share_repository::ShareRepository as DbShareRepository,
    user_repository::UserRepository as DbUserRepository,
    Database,
    interfaces::{IAccessLogRepository, IFileDatabaseRepository, IShareDatabaseRepository, IUserRepository},
    user_usecases::{CreateUserUseCase, FindUserUseCase},
    file_usecases::{CreateFileUseCase, FindFileUseCase},
    share_usecases::{CreateLinkUseCase, CreateGrantUseCase},
    log_usecases::LogAccessUseCase,
};
use crate::file::local_repository::FileRepository;
use crate::file::service::FileService;
use crate::file::interfaces::IFileRepository;
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

    let create_user_usecase = Arc::new(CreateUserUseCase::new(Arc::clone(&db_user_repo) as Arc<dyn IUserRepository>));
    let find_user_usecase = Arc::new(FindUserUseCase::new(Arc::clone(&db_user_repo) as Arc<dyn IUserRepository>));
    let create_file_usecase = Arc::new(CreateFileUseCase::new(Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>));
    let find_file_usecase = Arc::new(FindFileUseCase::new(Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>));
    let create_link_usecase = Arc::new(CreateLinkUseCase::new(Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>));
    let create_grant_usecase = Arc::new(CreateGrantUseCase::new(Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>));
    let log_access_usecase = Arc::new(LogAccessUseCase::new(Arc::new(db_log_repo.clone()) as Arc<dyn IAccessLogRepository>));

    let auth_service = Arc::new(AuthService::new(Arc::clone(&find_user_usecase), Arc::clone(&create_user_usecase)));
    let share_service = Arc::new(ShareService::new(Arc::clone(&share_repo)));

    let download_usecase = Arc::new(crate::file::DownloadUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
    ));
    let upload_usecase = Arc::new(crate::file::UploadUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&create_file_usecase),
        Arc::clone(&find_user_usecase),
    ));
    let list_usecase = Arc::new(crate::file::ListUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
    ));
    let mkdir_usecase = Arc::new(crate::file::MkdirUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&create_file_usecase),
        Arc::clone(&find_user_usecase),
    ));
    let delete_usecase = Arc::new(crate::file::DeleteUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
    ));
    let stat_usecase = Arc::new(crate::file::StatUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
    ));
    let rename_usecase = Arc::new(crate::file::RenameUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
    ));
    let rmdir_usecase = Arc::new(crate::file::RemoveDirUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
    ));
    let dir_exists_usecase = Arc::new(crate::file::DirExistsUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
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
    if find_user_usecase
        .execute(admin_username)
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
        create_user_usecase.execute(&dev_user).await?;

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
        create_file_usecase.execute(&dev_file).await?;
        let _ = find_file_usecase.execute(dev_file.id).await;

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
        create_link_usecase.execute(&dev_link).await?;

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
        create_grant_usecase.execute(&dev_grant).await?;

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
        log_access_usecase.execute(&dev_log).await?;

        info!("Données SQLite de développement injectées avec succès.");
    }

    Ok(Services {
        auth_service,
        file_service,
        share_service,
    })
}
