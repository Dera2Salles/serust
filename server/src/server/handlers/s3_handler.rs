use crate::file::service::FileService;
use crate::share::service::ShareService;
use crate::user::service::AuthService;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, header, Request, Response, StatusCode};
use percent_encoding::percent_decode_str;
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type S3Response = Response<Full<Bytes>>;

pub async fn serve_s3(
    req: Request<Incoming>,
    auth: Arc<AuthService>,
    files: Arc<FileService>,
    shares: Arc<ShareService>,
    logs: Arc<crate::database::log_usecases::LogAccessUseCase>,
) -> Result<S3Response, Infallible> {
    match handle_request(req, auth, files, shares, logs).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("S3 API error: {:?}", e);
            let mut response = Response::new(Full::new(Bytes::from("Internal Server Error")));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(response)
        }
    }
}

async fn handle_request(
    req: Request<Incoming>,
    auth: Arc<AuthService>,
    files: Arc<FileService>,
    shares: Arc<ShareService>,
    logs: Arc<crate::database::log_usecases::LogAccessUseCase>,
) -> Result<S3Response, BoxError> {
    let user = match authenticate(&req, auth.clone()).await {
        Ok(u) => u,
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Unauthorized")));
            *res.status_mut() = StatusCode::UNAUTHORIZED;
            return Ok(res);
        }
    };

    let path = req.uri().path().to_string();
    let method = req.method().clone();

    info!("S3 API: {} {} for user {}", method, path, user.username);

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        let mut res = Response::new(Full::new(Bytes::from("Invalid S3 path")));
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(res);
    }

    let mut rel_path_raw = segments[1..].join("/");
    
    if method == "GET" && segments.len() == 3 && segments[2] == ".profile_pic.jpg" {
        let target_username = segments[1];
        if target_username != user.username {
            if let Ok(Some(other_user)) = auth.get_user_by_username(target_username).await {
                return handle_get_object(&req, &other_user, ".profile_pic.jpg", files, logs.clone()).await;
            }
        }
    }

    if segments.len() > 2 && segments[1] == user.username {
        rel_path_raw = segments[2..].join("/");
    } else if segments.len() == 2 && segments[1] == user.username {
        rel_path_raw = String::new();
    }

    let rel_path = percent_decode_str(&rel_path_raw).decode_utf8_lossy().to_string();
    let is_dir_hint = req.uri().path().ends_with('/');

    match method.as_str() {
        "GET" => {
            if let Some(query) = req.uri().query() {
                if query.contains("list-type=2") {
                    return handle_list_objects(&user, &rel_path, files, shares.clone()).await;
                }
                if query.contains("shares=in") {
                    return handle_list_shares(&req, &user, shares).await;
                }
                if query.contains("git=history") {
                    return handle_git_history(&user, &rel_path, files).await;
                }
                if query.contains("git=diff") {
                    let hash = query.split('&')
                        .find(|s| s.starts_with("hash="))
                        .and_then(|s| s.split('=').nth(1))
                        .unwrap_or("");
                    return handle_git_diff(&user, &rel_path, hash, files).await;
                }
            }
            handle_get_object(&req, &user, &rel_path, files, logs).await
        }
        "POST" => {
            if let Some(query) = req.uri().query() {
                if query.contains("rename=") {
                    let to_raw = query.split('&').find(|s| s.starts_with("rename=")).and_then(|s| s.split('=').nth(1)).unwrap_or("");
                    let to = percent_decode_str(to_raw).decode_utf8_lossy().to_string();
                    return handle_rename(&user, &rel_path, &to, files).await;
                }
                if query.contains("git=restore") {
                    let hash = query.split('&')
                        .find(|s| s.starts_with("hash="))
                        .and_then(|s| s.split('=').nth(1))
                        .unwrap_or("");
                    return handle_git_restore(&user, &rel_path, hash, files).await;
                }
                if query.contains("share=grant") {
                    let to = query.split('&').find(|s| s.starts_with("to=")).and_then(|s| s.split('=').nth(1)).unwrap_or("");
                    let perm = query.split('&').find(|s| s.starts_with("perm=")).and_then(|s| s.split('=').nth(1)).unwrap_or("read");
                    let reshare = query.split('&').find(|s| s.starts_with("reshare=")).and_then(|s| s.split('=').nth(1)).unwrap_or("false") == "true";
                    let expires_at = query.split('&').find(|s| s.starts_with("expires_at=")).and_then(|s| s.split('=').nth(1)).and_then(|s| s.parse::<u64>().ok());
                    return handle_share_grant(&user, &rel_path, to, perm, reshare, expires_at, shares).await;
                }
                if query.contains("compress=") {
                    let format = query.split('&').find(|s| s.starts_with("compress=")).and_then(|s| s.split('=').nth(1)).unwrap_or("zip");
                    return handle_compress(&user, &rel_path, format, files).await;
                }
                if query.contains("decompress=true") {
                    return handle_decompress(&user, &rel_path, files).await;
                }
            }
            let mut res = Response::new(Full::new(Bytes::from("Method Not Allowed")));
            *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(res)
        }
        "PUT" => handle_put_object(req, &user, &rel_path, is_dir_hint, files, logs).await,
        "DELETE" => handle_delete_object(&user, &rel_path, files, logs).await,
        _ => {
            let mut res = Response::new(Full::new(Bytes::from("Method Not Allowed")));
            *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(res)
        }
    }
}

async fn authenticate(
    req: &Request<Incoming>,
    auth: Arc<AuthService>,
) -> Result<crate::user::domain::User, crate::common::error::DomainError> {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    if let Some(auth_val) = auth_header {
        let auth_str = auth_val.to_str().unwrap_or("");
        if auth_str.starts_with("Basic ") {
            let encoded = &auth_str[6..];
            use base64::{engine::general_purpose, Engine as _};
            if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        return auth.login(parts[0], parts[1]).await;
                    }
                }
            }
        }
    }
    Err(crate::common::error::DomainError::InvalidCredentials)
}

async fn handle_get_object(
    req: &Request<Incoming>,
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
    logs: Arc<crate::database::log_usecases::LogAccessUseCase>,
) -> Result<S3Response, BoxError> {
    if req.uri().query().map(|q| q.contains("presign=true")).unwrap_or(false) {
        if let Ok(Some(url)) = files.get_presigned_url(user, "/", path).await {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::TEMPORARY_REDIRECT;
            res.headers_mut().insert(header::LOCATION, url.parse()?);
            return Ok(res);
        }
    }

    match files.download(user, "/", path).await {
        Ok(data) => {
            if let Ok(Some(meta)) = files.find_db_file_by_path(user.id, path).await {
                let _ = logs.execute(&crate::database::domain::DbAccessLog {
                    id: 0,
                    file_id: meta.id,
                    accessed_by: Some(user.id),
                    share_link_id: None,
                    grant_id: None,
                    action: "read".into(),
                    accessed_at: chrono::Utc::now(),
                    ip_address: None,
                    user_agent: None,
                    bytes_transferred: Some(data.len() as i64),
                }).await;
            }

            let mut res = Response::new(Full::new(Bytes::from(data)));
            res.headers_mut().insert(header::CONTENT_TYPE, "application/octet-stream".parse()?);
            Ok(res)
        }
        Err(e) => {
            use crate::common::error::DomainError;
            let (status, msg) = match e {
                DomainError::PermissionDenied => (StatusCode::FORBIDDEN, "Permission Denied"),
                DomainError::FileNotFound => (StatusCode::NOT_FOUND, "Not Found"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error"),
            };
            let mut res = Response::new(Full::new(Bytes::from(msg)));
            *res.status_mut() = status;
            Ok(res)
        }
    }
}

async fn handle_rename(
    user: &crate::user::domain::User,
    from: &str,
    to: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    match files.rename(user, "/", from, to).await {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(e) => {
            error!("S3 Rename failed: {:?}", e);
            let mut res = Response::new(Full::new(Bytes::from(format!("Rename failed: {}", e))));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_put_object(
    req: Request<Incoming>,
    user: &crate::user::domain::User,
    path: &str,
    is_dir: bool,
    files: Arc<FileService>,
    logs: Arc<crate::database::log_usecases::LogAccessUseCase>,
) -> Result<S3Response, BoxError> {
    if is_dir {
        match files.mkdir(user, "/", path).await {
            Ok(_) => {
                let mut res = Response::new(Full::new(Bytes::new()));
                *res.status_mut() = StatusCode::CREATED;
                return Ok(res);
            }
            Err(e) => {
                error!("S3 MKDIR failed: {:?}", e);
                let mut res = Response::new(Full::new(Bytes::from("Internal Error")));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return Ok(res);
            }
        }
    }

    let body_bytes = req.into_body().collect().await?.to_bytes();
    let size = body_bytes.len() as u64;

    match files.upload(user, "/", path, size, body_bytes.to_vec()).await {
        Ok(_) => {
            if let Ok(Some(meta)) = files.find_db_file_by_path(user.id, path).await {
                let _ = logs.execute(&crate::database::domain::DbAccessLog {
                    id: 0,
                    file_id: meta.id,
                    accessed_by: Some(user.id),
                    share_link_id: None,
                    grant_id: None,
                    action: "upload".into(),
                    accessed_at: chrono::Utc::now(),
                    ip_address: None,
                    user_agent: None,
                    bytes_transferred: Some(size as i64),
                }).await;
            }

            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(e) => {
            error!("S3 PUT failed: {:?}", e);
            let mut res = Response::new(Full::new(Bytes::from("Internal Error")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_delete_object(
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
    logs: Arc<crate::database::log_usecases::LogAccessUseCase>,
) -> Result<S3Response, BoxError> {
    let file_meta = files.find_db_file_by_path(user.id, path).await.ok().flatten();

    match files.delete(user, "/", path).await {
        Ok(_) => {
            if let Some(meta) = file_meta {
                let _ = logs.execute(&crate::database::domain::DbAccessLog {
                    id: 0,
                    file_id: meta.id,
                    accessed_by: Some(user.id),
                    share_link_id: None,
                    grant_id: None,
                    action: "delete".into(),
                    accessed_at: chrono::Utc::now(),
                    ip_address: None,
                    user_agent: None,
                    bytes_transferred: None,
                }).await;
            }

            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::NO_CONTENT;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Not Found")));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

async fn handle_list_objects(
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
    shares: Arc<ShareService>,
) -> Result<S3Response, BoxError> {
    match files.list(user, path).await {
        Ok(entries) => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<ListBucketResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\n");
            
            let path_prefix = if path.is_empty() { 
                "".to_string() 
            } else if path.ends_with('/') { 
                path.to_string() 
            } else { 
                format!("{}/", path) 
            };

            xml.push_str(&format!("  <Name>arosaina-bucket</Name>\n  <Prefix>{}</Prefix>\n", path_prefix));
            
            for (name, is_dir) in entries {
                let mut can_read = true;
                let mut can_write = true;
                let mut can_download = true;
                let mut can_reshare = true;

                let item_path = if path == "/" {
                    name.clone()
                } else {
                    format!("{}/{}", path, name)
                };

                let grants = shares.list_incoming(&user.username).await;
                for grant in grants {
                    let share_path = if grant.path.starts_with('/') { &grant.path[1..] } else { &grant.path };
                    
                    let expected_prefix = format!("shared/{}/", grant.owner);
                    if item_path.starts_with(&expected_prefix) {
                        let inner_item_path = &item_path[expected_prefix.len()..];
                        if inner_item_path.starts_with(share_path) || share_path.starts_with(inner_item_path) {
                            can_read = grant.can_read;
                            can_write = grant.can_write;
                            can_download = grant.can_download;
                            can_reshare = grant.can_reshare;
                            break;
                        }
                    } else if item_path.starts_with("shared/") && item_path.split('/').nth(1).unwrap_or("") == grant.owner {
                         can_read = true; 
                    }
                }

                if is_dir {
                    xml.push_str(&format!("  <CommonPrefixes>\n    <Prefix>{}{}/</Prefix>\n    <CanRead>{}</CanRead>\n    <CanWrite>{}</CanWrite>\n    <CanDownload>{}</CanDownload>\n    <CanReshare>{}</CanReshare>\n  </CommonPrefixes>\n", 
                        path_prefix, name, can_read, can_write, can_download, can_reshare));
                } else {
                    let mut size = 0;
                    if let Ok(Some((s, _, _))) = files.stat(user, path, &name).await {
                        size = s;
                    }
                    xml.push_str("  <Contents>\n");
                    xml.push_str(&format!("    <Key>{}{}</Key>\n", path_prefix, name));
                    xml.push_str(&format!("    <Size>{}</Size>\n", size));
                    xml.push_str(&format!("    <CanRead>{}</CanRead>\n", can_read));
                    xml.push_str(&format!("    <CanWrite>{}</CanWrite>\n", can_write));
                    xml.push_str(&format!("    <CanDownload>{}</CanDownload>\n", can_download));
                    xml.push_str(&format!("    <CanReshare>{}</CanReshare>\n", can_reshare));
                    xml.push_str("  </Contents>\n");
                }
            }
            xml.push_str("</ListBucketResult>");
            
            let mut res = Response::new(Full::new(Bytes::from(xml)));
            res.headers_mut().insert(header::CONTENT_TYPE, "application/xml".parse()?);
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Internal Error")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_git_history(
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let filename = path.split('/').last().unwrap_or(path);
    let cwd = if path.contains('/') { path.rsplitn(2, '/').nth(1).unwrap_or("/") } else { "/" };

    match files.git_history(user, cwd, filename).await {
        Ok(history) => {
            let mut result = String::new();
            for (hash, time, msg) in history {
                result.push_str(&format!("{}|{}|{}\n", hash, time, msg));
            }
            let mut res = Response::new(Full::new(Bytes::from(result)));
            res.headers_mut().insert(header::CONTENT_TYPE, "text/plain".parse()?);
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error fetching history")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_git_diff(
    user: &crate::user::domain::User,
    path: &str,
    hash: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let filename = path.split('/').last().unwrap_or(path);
    let cwd = if path.contains('/') { path.rsplitn(2, '/').nth(1).unwrap_or("/") } else { "/" };

    match files.git_diff(user, cwd, filename, hash).await {
        Ok(diff) => {
            let mut res = Response::new(Full::new(Bytes::from(diff)));
            res.headers_mut().insert(header::CONTENT_TYPE, "text/plain".parse()?);
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error fetching diff")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_git_restore(
    user: &crate::user::domain::User,
    path: &str,
    hash: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let filename = path.split('/').last().unwrap_or(path);
    let cwd = if path.contains('/') { path.rsplitn(2, '/').nth(1).unwrap_or("/") } else { "/" };

    match files.git_restore(user, cwd, filename, hash).await {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error restoring version")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_share_grant(
    user: &crate::user::domain::User,
    path: &str,
    to: &str,
    perm: &str,
    can_reshare: bool,
    expires_at: Option<u64>,
    shares: Arc<ShareService>,
) -> Result<S3Response, BoxError> {
    let can_read = perm.contains('r') || perm.contains('R');
    let can_write = perm.contains('w') || perm.contains('W');
    let can_download = perm.contains('d') || perm.contains('D');
    
    match shares.grant(&user.username, "/", path, to, can_read, can_write, can_download, None, None, None, can_reshare, &user.username, expires_at).await {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error granting share")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_list_shares(
    req: &Request<Incoming>,
    user: &crate::user::domain::User,
    shares: Arc<ShareService>,
) -> Result<S3Response, BoxError> {
    let query = req.uri().query().unwrap_or("");
    let is_out = query.contains("type=out");
    
    let list = if is_out {
        shares.list_outgoing(&user.username).await
    } else {
        shares.list_incoming(&user.username).await
    };

    let result = serde_json::to_string(&list)?;
    
    let mut res = Response::new(Full::new(Bytes::from(result)));
    res.headers_mut().insert(header::CONTENT_TYPE, "application/json".parse()?);
    Ok(res)
}

async fn handle_compress(
    user: &crate::user::domain::User,
    path: &str,
    format: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let filename = path.split('/').last().unwrap_or(path);
    let cwd = if path.contains('/') { path.rsplitn(2, '/').nth(1).unwrap_or("/") } else { "/" };

    match files.compress(user, cwd, filename, format).await {
        Ok(out) => {
            let mut res = Response::new(Full::new(Bytes::from(out)));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error compressing")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_decompress(
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let filename = path.split('/').last().unwrap_or(path);
    let cwd = if path.contains('/') { path.rsplitn(2, '/').nth(1).unwrap_or("/") } else { "/" };

    match files.decompress(user, cwd, filename).await {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::OK;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Error decompressing")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}
