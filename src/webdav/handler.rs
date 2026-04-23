use crate::common::error::DomainError;
use crate::file::service::FileService;
use crate::user::domain::User;
use crate::user::service::AuthService;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, header, Request, Response, StatusCode};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type WebDavResponse = Response<Full<Bytes>>;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
/// WebDAV path segment encode set
const PATH_SEGMENT: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}').add(b'%');

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, PATH_SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

pub async fn serve_webdav(
    req: Request<Incoming>,
    auth: Arc<AuthService>,
    files: Arc<FileService>,
) -> Result<WebDavResponse, Infallible> {
    match handle_request(req, auth, files).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("WebDAV error: {:?}", e);
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
) -> Result<WebDavResponse, BoxError> {
    let user = match authenticate(&req, auth).await {
        Ok(u) => u,
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Unauthorized")));
            *res.status_mut() = StatusCode::UNAUTHORIZED;
            res.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                "Basic realm=\"WebDAV\"".parse().unwrap(),
            );
            return Ok(res);
        }
    };

    let path = req.uri().path().to_string();
    let method = req.method().clone();

    info!("WebDAV: {} {} for user {}", method, path, user.username);

    match method.as_str() {
        "OPTIONS" => handle_options(),
        "PROPFIND" => handle_propfind(&req, &user, &path, files).await,
        "GET" | "HEAD" => handle_get(&user, &path, files, method.as_str() == "HEAD").await,
        "PUT" => handle_put(req, &user, &path, files).await,
        "MKCOL" => handle_mkcol(&user, &path, files).await,
        "DELETE" => handle_delete(&user, &path, files).await,
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
) -> Result<User, DomainError> {
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
                        let username = parts[0];
                        let password = parts[1];
                        return auth.login(username, password).await;
                    }
                }
            }
        }
    }
    Err(DomainError::InvalidCredentials)
}

fn handle_options() -> Result<WebDavResponse, BoxError> {
    let mut res = Response::new(Full::new(Bytes::new()));
    res.headers_mut().insert(
        header::ALLOW,
        "OPTIONS, PROPFIND, GET, HEAD, PUT, MKCOL, DELETE, COPY, MOVE"
            .parse()
            .unwrap(),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("dav"),
        "1".parse().unwrap(),
    );
    res.headers_mut().insert(
        header::HeaderName::from_static("ms-author-via"),
        "DAV".parse().unwrap(),
    );
    Ok(res)
}

async fn handle_propfind(
    req: &Request<Incoming>,
    user: &User,
    path: &str,
    files: Arc<FileService>,
) -> Result<WebDavResponse, BoxError> {
    let depth = req
        .headers()
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1");

    let is_dir = if path == "/" {
        true
    } else {
        match files.stat(user, "/", path).await {
            Ok(Some((_, is_dir, _))) => is_dir,
            _ => {
                let mut res = Response::new(Full::new(Bytes::from("Not Found")));
                *res.status_mut() = StatusCode::NOT_FOUND;
                return Ok(res);
            }
        }
    };

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n");
    xml.push_str("<D:multistatus xmlns:D=\"DAV:\">\n");

    let build_response = |p: &str, p_is_dir: bool, size: u64| -> String {
        let mut href = encode_path(p);
        if p_is_dir && !href.ends_with('/') {
            href.push('/');
        }

        let displayname = p.trim_end_matches('/').split('/').last().unwrap_or("");

        let mut entry = format!(
            "<D:response>\n  <D:href>{}</D:href>\n  <D:propstat>\n    <D:prop>\n",
            href
        );

        if !displayname.is_empty() {
            entry.push_str(&format!("      <D:displayname>{}</D:displayname>\n", displayname));
        }

        if p_is_dir {
            entry.push_str("      <D:resourcetype><D:collection/></D:resourcetype>\n");
        } else {
            entry.push_str("      <D:resourcetype/>\n");
            entry.push_str(&format!(
                "      <D:getcontentlength>{}</D:getcontentlength>\n",
                size
            ));
            entry.push_str("      <D:getcontenttype>application/octet-stream</D:getcontenttype>\n");
        }
        
        entry.push_str("      <D:getlastmodified>Thu, 01 Jan 1970 00:00:00 GMT</D:getlastmodified>\n");
        entry.push_str("      <D:lockdiscovery/>\n");
        entry.push_str("      <D:supportedlock/>\n");

        entry.push_str("    </D:prop>\n    <D:status>HTTP/1.1 200 OK</D:status>\n  </D:propstat>\n</D:response>\n");
        entry
    };

    if path == "/" {
        xml.push_str(&build_response("/", true, 0));
    } else {
        match files.stat(user, "/", path).await {
            Ok(Some((size, is_dir_stat, _))) => {
                xml.push_str(&build_response(path, is_dir_stat, size));
            }
            _ => {}
        }
    }

    if is_dir && depth != "0" {
        if let Ok(entries) = files.list(user, path).await {
            for (name, entry_is_dir) in entries {
                let entry_path = if path.ends_with('/') {
                    format!("{}{}", path, name)
                } else {
                    format!("{}/{}", path, name)
                };

                let size = if !entry_is_dir {
                    files
                        .stat(user, "/", &entry_path)
                        .await
                        .unwrap_or(Some((0, false, None)))
                        .unwrap_or((0, false, None))
                        .0
                } else {
                    0
                };

                xml.push_str(&build_response(&entry_path, entry_is_dir, size));
            }
        }
    }

    xml.push_str("</D:multistatus>");

    let mut res = Response::new(Full::new(Bytes::from(xml)));
    *res.status_mut() = StatusCode::MULTI_STATUS;
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    Ok(res)
}

async fn handle_get(
    user: &User,
    path: &str,
    files: Arc<FileService>,
    is_head: bool,
) -> Result<WebDavResponse, BoxError> {
    match files.stat(user, "/", path).await {
        Ok(Some((size, is_dir, _))) => {
            if is_dir {
                // For directories, we could return a simple HTML listing or just 200 OK
                let mut res = Response::new(Full::new(Bytes::from("Directory")));
                if is_head {
                    *res.body_mut() = Full::new(Bytes::new());
                }
                return Ok(res);
            }

            match files.download(user, "/", path).await {
                Ok(data) => {
                    let mut res = Response::new(Full::new(Bytes::from(if is_head {
                        Bytes::new()
                    } else {
                        Bytes::from(data)
                    })));
                    res.headers_mut().insert(
                        header::CONTENT_LENGTH,
                        header::HeaderValue::from(size),
                    );
                    // Add content type if possible
                    Ok(res)
                }
                Err(DomainError::FileNotFound) => {
                    let mut res = Response::new(Full::new(Bytes::from("Not Found")));
                    *res.status_mut() = StatusCode::NOT_FOUND;
                    Ok(res)
                }
                Err(_) => {
                    let mut res = Response::new(Full::new(Bytes::from("Internal Server Error")));
                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(res)
                }
            }
        }
        _ => {
            let mut res = Response::new(Full::new(Bytes::from("Not Found")));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

async fn handle_put(
    req: Request<Incoming>,
    user: &User,
    path: &str,
    files: Arc<FileService>,
) -> Result<WebDavResponse, BoxError> {
    let body_bytes = req.into_body().collect().await?.to_bytes();
    let size = body_bytes.len() as u64;

    match files
        .upload(user, "/", path, size, body_bytes.to_vec())
        .await
    {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::CREATED;
            Ok(res)
        }
        Err(e) => {
            error!("PUT upload failed: {:?}", e);
            let mut res = Response::new(Full::new(Bytes::from("Internal Server Error")));
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(res)
        }
    }
}

async fn handle_mkcol(
    user: &User,
    path: &str,
    files: Arc<FileService>,
) -> Result<WebDavResponse, BoxError> {
    match files.mkdir(user, "/", path).await {
        Ok(_) => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::CREATED;
            Ok(res)
        }
        Err(_) => {
            let mut res = Response::new(Full::new(Bytes::from("Conflict or Server Error")));
            *res.status_mut() = StatusCode::CONFLICT;
            Ok(res)
        }
    }
}

async fn handle_delete(
    user: &User,
    path: &str,
    files: Arc<FileService>,
) -> Result<WebDavResponse, BoxError> {
    let stat = files.stat(user, "/", path).await;
    match stat {
        Ok(Some((_, is_dir, _))) => {
            let result = if is_dir {
                files.rmdir(user, "/", path).await
            } else {
                files.delete(user, "/", path).await
            };

            match result {
                Ok(_) => {
                    let mut res = Response::new(Full::new(Bytes::new()));
                    *res.status_mut() = StatusCode::NO_CONTENT;
                    Ok(res)
                }
                Err(_) => {
                    let mut res = Response::new(Full::new(Bytes::from("Internal Server Error")));
                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(res)
                }
            }
        }
        _ => {
            let mut res = Response::new(Full::new(Bytes::from("Not Found")));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}
