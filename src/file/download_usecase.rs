use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use aes::Aes256;
use cbc::Decryptor;
use flate2::read::GzDecoder;
use std::io::Read;
use std::sync::Arc;

type Aes256CbcDec = Decryptor<Aes256>;

use crate::user::service::AuthService;

pub struct DownloadUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    auth: Arc<AuthService>,
}

impl DownloadUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        auth: Arc<AuthService>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            auth,
        }
    }

    fn decrypt_if_needed(data: Vec<u8>, password_hash: &str) -> Vec<u8> {
        let magic = b"AROS-E2EE:";
        if data.len() < magic.len() + 16 {
            return data;
        }

        if &data[..magic.len()] != magic {
            return data;
        }

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password_hash.as_bytes());
        let key = hasher.finalize();

        let iv = &data[magic.len()..magic.len() + 16];
        let encrypted_content = &data[magic.len() + 16..];

        let decryptor = Aes256CbcDec::new(&key.into(), iv.into());

        let mut buffer = encrypted_content.to_vec();
        match decryptor.decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buffer) {
            Ok(decrypted) => decrypted.to_vec(),
            Err(_) => data,
        }
    }

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<Vec<u8>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        let mut password_hash = user.password_hash.clone();

        let data = if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            if !self
                .shares
                .can_download(&user.username, &owner, &inner)
                .await
            {
                return Err(DomainError::PermissionDenied);
            }

            // If it's a shared file, we must use the owner's password hash to decrypt it
            if let Ok(Some(owner_user)) = self.auth.get_user_by_username(&owner).await {
                password_hash = owner_user.password_hash;
            }

            self.file_repo.load(&owner, &inner).await?
        } else {
            let _storage_path = format!("/{}", resolved);

            let meta = self
                .file_repo
                .get_metadata(&user.username, &resolved)
                .await
                .ok_or(DomainError::FileNotFound)?;

            if !PermissionChecker::can_access(user, &meta.owner, &Permission::Read) {
                return Err(DomainError::PermissionDenied);
            }

            self.file_repo.load(&user.username, &resolved).await?
        };

        let data = Self::decrypt_if_needed(data, &password_hash);

        let data = if data.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                decompressed
            } else {
                data
            }
        } else {
            data
        };

        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            self.shares
                .consume_download(&user.username, &owner, &inner)
                .await?;
        }

        let log_event =
            crate::log::domain::AccessLog::new_download_event(uuid::Uuid::new_v4(), None);
        tracing::debug!("Domain access log recorded: {:?}", log_event);

        Ok(data)
    }
}
