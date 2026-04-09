use crate::database::user_usecases::{CreateUserUseCase, FindUserUseCase};
use crate::database::file_usecases::{CreateFileUseCase, FindFileUseCase};
use crate::database::share_usecases::{CreateLinkUseCase, CreateGrantUseCase};
use crate::database::log_usecases::LogAccessUseCase;
use std::sync::Arc;

pub struct DatabaseService {
    pub create_user: Arc<CreateUserUseCase>,
    pub find_user: Arc<FindUserUseCase>,
    pub create_file: Arc<CreateFileUseCase>,
    pub find_file: Arc<FindFileUseCase>,
    pub create_link: Arc<CreateLinkUseCase>,
    pub create_grant: Arc<CreateGrantUseCase>,
    pub log_access: Arc<LogAccessUseCase>,
}

impl DatabaseService {
    pub fn new(
        create_user: Arc<CreateUserUseCase>,
        find_user: Arc<FindUserUseCase>,
        create_file: Arc<CreateFileUseCase>,
        find_file: Arc<FindFileUseCase>,
        create_link: Arc<CreateLinkUseCase>,
        create_grant: Arc<CreateGrantUseCase>,
        log_access: Arc<LogAccessUseCase>,
    ) -> Self {
        Self {
            create_user,
            find_user,
            create_file,
            find_file,
            create_link,
            create_grant,
            log_access,
        }
    }
}
