use crate::file::service::FileService;
use crate::share::service::ShareService;
use crate::user::service::AuthService;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, header, Request, Response, StatusCode};
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
) -> Result<S3Response, Infallible> {
    match handle_request(req, auth, files, shares).await {
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
) -> Result<S3Response, BoxError> {
    // 1. Authentication
    let user = match authenticate(&req, auth).await {
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

    let rel_path = segments[1..].join("/");

    match method.as_str() {
        "GET" => {
            if let Some(query) = req.uri().query() {
                if query.contains("list-type=2") {
                    return handle_list_objects(&user, &rel_path, files).await;
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
            handle_get_object(&req, &user, &rel_path, files).await
        }
        "POST" => {
            if let Some(query) = req.uri().query() {
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
                    return handle_share_grant(&user, &rel_path, to, perm, shares).await;
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
        "PUT" => handle_put_object(req, &user, &rel_path, files).await,
        "DELETE" => handle_delete_object(&user, &rel_path, files).await,
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
            let mut res = Response::new(Full::new(Bytes::from(data)));
            res.headers_mut().insert(header::CONTENT_TYPE, "application/octet-stream".parse()?);
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Not Found")));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

async fn handle_put_object(
    req: Request<Incoming>,
    user: &crate::user::domain::User,
    path: &str,
    files: Arc<FileService>,
) -> Result<S3Response, BoxError> {
    let body_bytes = req.into_body().collect().await?.to_bytes();
    let size = body_bytes.len() as u64;

    match files.upload(user, "/", path, size, body_bytes.to_vec()).await {
        Ok(_) => {
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
) -> Result<S3Response, BoxError> {
    match files.delete(user, "/", path).await {
        Ok(_) => {
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
) -> Result<S3Response, BoxError> {
    match files.list(user, path).await {
        Ok(entries) => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<ListBucketResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\n");
            xml.push_str(&format!("  <Name>arosaina-bucket</Name>\n  <Prefix>{}</Prefix>\n", path));
            
            for (name, is_dir) in entries {
                if is_dir {
                    xml.push_str(&format!("  <CommonPrefixes><Prefix>{}{}/</Prefix></CommonPrefixes>\n", path, name));
                } else {
                    xml.push_str("  <Contents>\n");
                    xml.push_str(&format!("    <Key>{}{}</Key>\n", path, name));
                    xml.push_str("    <Size>0</Size>\n");
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
    shares: Arc<ShareService>,
) -> Result<S3Response, BoxError> {
    let can_read = perm.contains('r');
    let can_write = perm.contains('w');
    let can_download = perm.contains('d');
    
    match shares.grant(&user.username, "/", path, to, can_read, can_write, can_download, None, None, None, false, &user.username, None).await {
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

    let mut result = String::new();
    for g in list {
        if is_out {
            result.push_str(&format!("grantee={} path={} perm={}\n", g.grantee, g.path, if g.can_read { "r" } else { "" }));
        } else {
            result.push_str(&format!("owner={} path={} perm={}\n", g.owner, g.path, if g.can_read { "r" } else { "" }));
        }
    }
    let mut res = Response::new(Full::new(Bytes::from(result)));
    res.headers_mut().insert(header::CONTENT_TYPE, "text/plain".parse()?);
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

