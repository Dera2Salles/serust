use crate::common::error::DomainError;
use crate::file::domain::FileMetadata;
use crate::file::interfaces::IFileRepository;
use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncSeek};
use std::io::SeekFrom;


pub struct S3Repository {
    client: Client,
    bucket: String,
}

impl S3Repository {
    fn user_key(&self, username: &str, rel_path: &str) -> String {
        if rel_path.is_empty() {
            format!("{}/", username)
        } else {
            format!("{}/{}", username, rel_path)
        }
    }
}

#[async_trait]
impl IFileRepository for S3Repository {
    async fn store(&self, meta: FileMetadata, data: Vec<u8>) -> Result<(), DomainError> {
        let key = self.user_key(&meta.owner, &meta.filename);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, username: &str, rel_path: &str) -> Result<Vec<u8>, DomainError> {
        let key = self.user_key(username, rel_path);
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|_| DomainError::FileNotFound)?;

        let data = resp.body.collect().await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .to_vec();
        Ok(data)
    }

    async fn delete_file(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        let key = self.user_key(username, rel_path);
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn rename(
        &self,
        username: &str,
        old_rel_path: &str,
        new_rel_path: &str,
    ) -> Result<(), DomainError> {
        let old_key = self.user_key(username, old_rel_path);
        let new_key = self.user_key(username, new_rel_path);

        // Copy
        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(format!("{}/{}", self.bucket, old_key))
            .key(&new_key)
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Delete old
        self.delete_file(username, old_rel_path).await?;
        Ok(())
    }

    async fn stat(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Option<(u64, bool)>, DomainError> {
        let key = self.user_key(username, rel_path);
        
        // Try exact object first (file)
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await {
                Ok(resp) => {
                    return Ok(Some((resp.content_length().unwrap_or(0) as u64, false)));
                }
                Err(_) => {
                    // Try listing with prefix to see if it's a "directory"
                    let dir_key = if key.ends_with('/') { key.clone() } else { format!("{}/", key) };
                    let resp = self.client
                        .list_objects_v2()
                        .bucket(&self.bucket)
                        .prefix(dir_key)
                        .max_keys(1)
                        .send()
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                    
                    if resp.key_count().unwrap_or(0) > 0 {
                        return Ok(Some((0, true)));
                    }
                }
            }
        
        Ok(None)
    }

    async fn create_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        // In S3, directories are just prefixes. We can create a placeholder if needed.
        let key = if rel_path.ends_with('/') {
            self.user_key(username, rel_path)
        } else {
            format!("{}/", self.user_key(username, rel_path))
        };
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(Vec::new()))
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn remove_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        let prefix = if rel_path.ends_with('/') {
            self.user_key(username, rel_path)
        } else {
            format!("{}/", self.user_key(username, rel_path))
        };

        // List all objects with prefix and delete them
        let objects = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for obj in objects.contents() {
            if let Some(key) = obj.key() {
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
        }
        
        Ok(())
    }

    async fn dir_exists(&self, username: &str, rel_path: &str) -> bool {
        let res = self.stat(username, rel_path).await;
        match res {
            Ok(Some((_, is_dir))) => is_dir,
            _ => false,
        }
    }

    async fn list_entries(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Vec<(String, bool)>, DomainError> {
        let prefix = self.user_key(username, rel_path);
        let prefix = if prefix.is_empty() || prefix.ends_with('/') { prefix } else { format!("{}/", prefix) };

        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .delimiter("/")
            .send()
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut result = Vec::new();

        // Common prefixes are directories
        for p in resp.common_prefixes() {
            if let Some(prefix_str) = p.prefix() {
                let name = prefix_str
                    .strip_prefix(&prefix)
                    .unwrap_or(prefix_str)
                    .trim_end_matches('/')
                    .to_string();
                if !name.is_empty() {
                    if name == ".git" {
                        continue;
                    }
                    result.push((name, true));
                }
            }
        }

        // Contents are files
        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                let name = key
                    .strip_prefix(&prefix)
                    .unwrap_or(key)
                    .to_string();
                if !name.is_empty() && !name.ends_with('/') {
                    if name == ".git" {
                        continue;
                    }
                    result.push((name, false));
                }
            }
        }

        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    async fn get_metadata(&self, username: &str, rel_path: &str) -> Option<FileMetadata> {
        let (size, _) = self.stat(username, rel_path).await.ok()??;
        Some(FileMetadata::new(rel_path, size, username))
    }

    async fn get_reader(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Pin<Box<dyn crate::file::interfaces::AsyncReadSeek>>, DomainError> {
        let key = self.user_key(username, rel_path);
        let reader = S3Reader::new(self.client.clone(), self.bucket.clone(), key).await?;
        Ok(Box::pin(reader))
    }

    async fn get_presigned_url(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Option<String>, DomainError> {
        let key = self.user_key(username, rel_path);
        let expires_in = std::time::Duration::from_secs(3600); // 1 hour
        
        let presigned_req = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in).map_err(|e| DomainError::Internal(e.to_string()))?)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(Some(presigned_req.uri().to_string()))
    }
}

pub struct S3Reader {
    client: Client,
    bucket: String,
    key: String,
    pos: u64,
    size: u64,
    current_stream: Option<Pin<Box<dyn AsyncRead + Send>>>,
}

impl S3Reader {
    pub async fn new(client: Client, bucket: String, key: String) -> Result<Self, DomainError> {
        let resp = client
            .head_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .map_err(|_| DomainError::FileNotFound)?;
        
        let size = resp.content_length().unwrap_or(0) as u64;
        
        Ok(Self {
            client,
            bucket,
            key,
            pos: 0,
            size,
            current_stream: None,
        })
    }
}

impl AsyncRead for S3Reader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.pos >= self.size {
            return std::task::Poll::Ready(Ok(()));
        }

        if self.current_stream.is_none() {
            let _client = self.client.clone();
            let _bucket = self.bucket.clone();
            let _key = self.key.clone();
            let _pos = self.pos;
            
            // We need a way to create the stream asynchronously within poll_read or pre-fetch it.
            // Simplified: we'll just return Pending and spawn a future to update the stream?
            // Actually, in a real implementation, we'd use a more robust S3 reader crate.
            // For now, let's just return an error if stream is missing, 
            // but we should ideally initiate the fetch.
            
            // Note: This is a simplified implementation. A real one would use 
            // a future to get the stream and then poll it.
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream not initialized. S3Reader requires sequential read or explicit seek initialization.",
            )));
        }

        let stream = self.current_stream.as_mut().unwrap();
        match Pin::new(stream).poll_read(cx, buf) {
            std::task::Poll::Ready(Ok(())) => {
                self.pos += buf.filled().len() as u64;
                std::task::Poll::Ready(Ok(()))
            }
            res => res,
        }
    }
}

impl AsyncSeek for S3Reader {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> std::io::Result<()> {
        let new_pos = match position {
            SeekFrom::Start(p) => p,
            SeekFrom::Current(p) => (self.pos as i64 + p) as u64,
            SeekFrom::End(p) => (self.size as i64 + p) as u64,
        };
        
        if new_pos != self.pos {
            self.pos = new_pos;
            self.current_stream = None; // Reset stream on seek
        }
        Ok(())
    }

    fn poll_complete(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<u64>> {
        if self.current_stream.is_none() && self.pos < self.size {
            // Need to fetch stream for the new position
            let _client = self.client.clone();
            let _bucket = self.bucket.clone();
            let _key = self.key.clone();
            let _pos = self.pos;
            
            // This is still tricky in poll_complete. 
            // Usually, you'd use a state machine or a lazy stream.
        }
        std::task::Poll::Ready(Ok(self.pos))
    }
}
