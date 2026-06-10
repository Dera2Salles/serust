use crate::database::{
    access_log_repository::AccessLogRepository as DbAccessLogRepository,
    admin_repository::AdminRepository as DbAdminRepository,
    domain::{DbAccessLog, DbAdmin, DbFileMetadata, DbShareGrant, DbShareLink, DbUser},
    file_repository::FileRepository as DbFileRepository,
    file_usecases::{
        CreateFileUseCase, DeleteByPathPrefixDbUseCase, DeletePermanentlyDbUseCase,
        FindDeletedFilesDbUseCase, FindFileByPathUseCase, FindFileUseCase, RenameFileDbUseCase,
        RestoreFileDbUseCase, SoftDeleteFileDbUseCase, UpdateFileUseCase,
        UpdatePathPrefixDbUseCase,
    },
    interfaces::{
        IAccessLogRepository, IAdminRepository, IFileDatabaseRepository, IShareDatabaseRepository,
        IUserRepository,
    },
    log_usecases::LogAccessUseCase,
    share_repository::ShareRepository as DbShareRepository,
    share_usecases::{
        CreateGrantUseCase, CreateLinkUseCase, ListMyGrantsUseCase, ListMyLinksUseCase,
        RevokeGrantUseCase, RevokeLinkUseCase,
    },
    user_repository::UserRepository as DbUserRepository,
    user_usecases::{CreateUserUseCase, FindUserByEmailUseCase, FindUserUseCase},
    Database,
};
use crate::file::compression_service::CompressionService;
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::file::service::FileService;
use crate::share::local_repository::ShareRepository;
use crate::share::service::ShareService;
use crate::user::service::AuthService;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub struct Services {
    pub auth_service: Arc<AuthService>,
    pub file_service: Arc<FileService>,
    pub share_service: Arc<ShareService>,
    pub log_access_usecase: Arc<LogAccessUseCase>,
    pub db: Database,
}

pub async fn setup_injection() -> Result<Services> {
    let storage_root = std::env::var("STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("storage"));

    let bucket = std::env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "arosaina-storage".to_string());
    info!("Mode Stockage Local activé (Interface S3-compatible)");

    let file_repo = Arc::new(crate::file::local_repository::FileRepository::new(
        storage_root.clone(),
    ));
    let share_repo = Arc::new(ShareRepository::new("shares.json").await);

    info!("Initialisation de la base de données PostgreSQL...");
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        info!("DATABASE_URL non défini, utilisation de la valeur par défaut pour PostgreSQL.");
        "postgres://aro:aropasssecret@localhost:5432/arodb".to_string()
    });
    let db = Database::new(&db_url).await?;
    let settings = crate::common::config::load_config();

    let db_user_repo = Arc::new(DbUserRepository::new(db.clone()));
    let db_file_repo = Arc::new(DbFileRepository::new(db.clone()));
    let db_admin_repo = Arc::new(DbAdminRepository::new(db.clone()));
    let db_share_repo = DbShareRepository::new(db.clone());
    let db_log_repo = DbAccessLogRepository::new(db.clone());

    let create_user_usecase = Arc::new(CreateUserUseCase::new(
        Arc::clone(&db_user_repo) as Arc<dyn IUserRepository>
    ));
    let find_user_usecase = Arc::new(FindUserUseCase::new(
        Arc::clone(&db_user_repo) as Arc<dyn IUserRepository>
    ));
    let find_user_by_email_usecase = Arc::new(FindUserByEmailUseCase::new(
        Arc::clone(&db_user_repo) as Arc<dyn IUserRepository>,
    ));
    let create_file_usecase = Arc::new(CreateFileUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let find_file_usecase = Arc::new(FindFileUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let update_file_usecase = Arc::new(UpdateFileUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let find_file_by_path_usecase = Arc::new(FindFileByPathUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let rename_db_file_usecase = Arc::new(RenameFileDbUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let soft_delete_db_file_usecase = Arc::new(SoftDeleteFileDbUseCase::new(Arc::clone(
        &db_file_repo,
    )
        as Arc<dyn IFileDatabaseRepository>));
    let restore_db_file_usecase = Arc::new(RestoreFileDbUseCase::new(
        Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
    ));
    let delete_permanently_db_file_usecase = Arc::new(DeletePermanentlyDbUseCase::new(Arc::clone(
        &db_file_repo,
    )
        as Arc<dyn IFileDatabaseRepository>));
    let delete_by_path_prefix_db_usecase = Arc::new(DeleteByPathPrefixDbUseCase::new(Arc::clone(
        &db_file_repo,
    )
        as Arc<dyn IFileDatabaseRepository>));
    let find_deleted_files_db_usecase = Arc::new(FindDeletedFilesDbUseCase::new(Arc::clone(
        &db_file_repo,
    )
        as Arc<dyn IFileDatabaseRepository>));
    let update_path_prefix_db_usecase = Arc::new(UpdatePathPrefixDbUseCase::new(Arc::clone(
        &db_file_repo,
    )
        as Arc<dyn IFileDatabaseRepository>));

    let list_files_by_parent_usecase = Arc::new(
        crate::database::file_usecases::ListFilesByParentUseCase::new(
            Arc::clone(&db_file_repo) as Arc<dyn IFileDatabaseRepository>
        ),
    );
    let create_link_usecase = Arc::new(CreateLinkUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>
    ));
    let create_grant_usecase = Arc::new(CreateGrantUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>
    ));
    let list_my_links_usecase = Arc::new(ListMyLinksUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>
    ));
    let list_my_grants_usecase = Arc::new(ListMyGrantsUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>,
    ));
    let revoke_link_usecase = Arc::new(RevokeLinkUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>
    ));
    let revoke_grant_usecase = Arc::new(RevokeGrantUseCase::new(
        Arc::new(db_share_repo.clone()) as Arc<dyn IShareDatabaseRepository>
    ));
    let log_access_usecase = Arc::new(LogAccessUseCase::new(
        Arc::new(db_log_repo.clone()) as Arc<dyn IAccessLogRepository>
    ));

    let auth_service = Arc::new(AuthService::new(
        Arc::clone(&find_user_by_email_usecase),
        Arc::clone(&find_user_usecase),
        Arc::clone(&create_user_usecase),
        settings.clone(),
    ));
    let share_service = Arc::new(ShareService::new(
        Arc::clone(&share_repo),
        db.clone(),
        Arc::clone(&create_link_usecase),
        Arc::clone(&create_grant_usecase),
        Arc::clone(&list_my_links_usecase),
        Arc::clone(&list_my_grants_usecase),
        Arc::clone(&revoke_link_usecase),
        Arc::clone(&revoke_grant_usecase),
    ));

    let git_service = Arc::new(GitService::new(Some(bucket.clone())));
    let compression_service = Arc::new(CompressionService::new());

    let download_usecase = Arc::new(crate::file::DownloadUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&auth_service),
    ));
    let upload_usecase = Arc::new(crate::file::UploadUseCase::new(
        storage_root.clone(),
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&create_file_usecase),
        Arc::clone(&update_file_usecase),
        Arc::clone(&find_file_by_path_usecase),
        Arc::clone(&git_service),
    ));
    let list_usecase = Arc::new(crate::file::ListUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&find_file_by_path_usecase),
        Arc::clone(&list_files_by_parent_usecase),
    ));
    let mkdir_usecase = Arc::new(crate::file::MkdirUseCase::new(
        storage_root.clone(),
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&create_file_usecase),
        Arc::clone(&find_file_by_path_usecase),
        Arc::clone(&git_service),
    ));
    let delete_usecase = Arc::new(crate::file::DeleteUseCase::new(
        storage_root.clone(),
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&find_file_by_path_usecase),
        Arc::clone(&create_file_usecase),
        Arc::clone(&soft_delete_db_file_usecase),
        Arc::clone(&git_service),
    ));
    let stat_usecase = Arc::new(crate::file::StatUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&share_service),
        Arc::clone(&find_file_by_path_usecase),
    ));
    let rename_usecase = Arc::new(crate::file::RenameUseCase::new(
        storage_root.clone(),
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&find_file_by_path_usecase),
        Arc::clone(&create_file_usecase),
        Arc::clone(&rename_db_file_usecase),
        Arc::clone(&update_path_prefix_db_usecase),
        Arc::clone(&git_service),
    ));
    let restore_usecase = Arc::new(crate::file::RestoreUseCase::new(Arc::clone(
        &restore_db_file_usecase,
    )));
    let purge_usecase = Arc::new(crate::file::PurgeUseCase::new(
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        Arc::clone(&find_file_usecase),
        Arc::clone(&delete_permanently_db_file_usecase),
        Arc::clone(&delete_by_path_prefix_db_usecase),
    ));

    let file_service = Arc::new(FileService::new(
        storage_root.clone(),
        Arc::clone(&file_repo) as Arc<dyn IFileRepository>,
        download_usecase,
        upload_usecase,
        list_usecase,
        mkdir_usecase,
        delete_usecase,
        stat_usecase,
        rename_usecase,
        restore_usecase,
        purge_usecase,
        find_deleted_files_db_usecase,
        find_file_by_path_usecase,
        git_service.clone(),
        compression_service,
        share_service.clone(),
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
        let _ = auth_service
            .register(
                name,
                &format!("{}@local", name),
                pass,
                None,
                None,
                None,
                None,
            )
            .await;
        let user_path = storage_root.join(name);
        let _ = git_service.setup_s3_remote(&user_path, &bucket, name);
        info!("Utilisateur prêt avec remote S3 : {}", name);
    }

    let admin_username = "admin_dev";
    if find_user_usecase.execute(admin_username).await?.is_none() {
        let dev_user = DbUser {
            id: uuid::Uuid::new_v4(),
            username: admin_username.to_string(),
            password_hash: AuthService::hash_password("admin123"),
            email: "admin@local".to_string(),
            first_name: Some("Admin".into()),
            last_name: Some("User".into()),
            birth_date: None,
            location: None,
            profile_pic_path: None,
            created_at: chrono::Utc::now(),
            storage_quota_bytes: 1048576,
            is_active: true,
        };
        create_user_usecase.execute(&dev_user).await?;

        let dev_admin = DbAdmin {
            user_id: dev_user.id,
            access_level: "standard".to_string(),
            last_action_at: Some(chrono::Utc::now()),
        };
        db_admin_repo.create(&dev_admin).await?;

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

        info!("Données de développement injectées avec succès.");
    }

    Ok(Services {
        auth_service,
        file_service,
        share_service,
        log_access_usecase,
        db,
    })
}
