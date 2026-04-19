use crate::database::analytics_repository::AnalyticsRepository;
use crate::database::interfaces::{IFileDatabaseRepository, IShareDatabaseRepository, IUserRepository};
use crate::database::{
    file_repository::FileRepository as DbFileRepository,
    share_repository::ShareRepository as DbShareRepository, Database,
};
use crate::file::service::FileService;
use crate::mcp::registry::McpRegistry;
use crate::mcp::types::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, PromptsCapability, ResourcesCapability,
    ServerCapabilities, ServerInfo, ToolsCapability,
};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info};

pub struct McpServerState {
    pub registry: Arc<McpRegistry>,
    #[allow(dead_code)]
    pub file_service: Arc<FileService>,
    pub db: Database,
}

/// Start the MCP HTTP server on a separate port.
pub async fn run_mcp_server(state: Arc<McpServerState>, addr: &str) -> anyhow::Result<()> {
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(addr).await?;
    info!("MCP server listening on http://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let state = Arc::clone(&state);
                handle_http(req, state)
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                error!("MCP connection error from {}: {}", peer, e);
            }
        });
    }
}

async fn handle_http(
    req: hyper::Request<hyper::body::Incoming>,
    state: Arc<McpServerState>,
) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, Infallible> {
    use http_body_util::{BodyExt, Full};
    use hyper::{Method, StatusCode};

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    let cors_headers = [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        (
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ),
    ];

    if method == Method::OPTIONS {
        let mut resp = hyper::Response::new(Full::new(bytes::Bytes::new()));
        *resp.status_mut() = StatusCode::NO_CONTENT;
        for (k, v) in &cors_headers {
            resp.headers_mut().insert(
                hyper::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                hyper::header::HeaderValue::from_static(v),
            );
        }
        return Ok(resp);
    }

    // ── Route: public file download by share token ────────────────────────────
    if method == Method::GET && path.starts_with("/public/") {
        let token = path.trim_start_matches("/public/");
        return Ok(handle_public_download(token, &state, &cors_headers).await);
    }

    // ── Analytics routes ──────────────────────────────────────────────────────
    if method == Method::GET && path == "/api/storage/stats" {
        let username = extract_query_param(&query, "username").unwrap_or("alice");
        let user = crate::user::domain::User {
            username: username.to_string(),
            password_hash: String::new(),
        };
        let (total_bytes, file_count, dir_count) = state.registry.recursive_stats(&user, "/").await;
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "total_bytes": total_bytes,
                "file_count": file_count,
                "folder_count": dir_count,
            }),
            &cors_headers,
        ));
    }

    if method == Method::GET && path.starts_with("/api/analytics") {
        let username = extract_query_param(&query, "username").unwrap_or("alice");
        let days: i64 = extract_query_param(&query, "days")
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        let limit: i64 = extract_query_param(&query, "limit")
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let analytics = AnalyticsRepository::new(state.db.clone());
        return Ok(match path.as_str() {
            "/api/analytics/summary" => match analytics.get_summary(username).await {
                Ok(s) => json_response(StatusCode::OK, json!(s), &cors_headers),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                    &cors_headers,
                ),
            },
            "/api/analytics/bandwidth" => match analytics.bandwidth_by_day(username, days).await {
                Ok(b) => json_response(StatusCode::OK, json!({"bandwidth": b}), &cors_headers),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                    &cors_headers,
                ),
            },
            "/api/analytics/top_files" => match analytics.top_files(username, limit).await {
                Ok(f) => json_response(StatusCode::OK, json!({"top_files": f}), &cors_headers),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                    &cors_headers,
                ),
            },
            "/api/analytics/recent" => match analytics.recent_activity(username, limit).await {
                Ok(r) => json_response(StatusCode::OK, json!({"recent": r}), &cors_headers),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"error": e.to_string()}),
                    &cors_headers,
                ),
            },
            _ => json_response(
                StatusCode::NOT_FOUND,
                json!({"error": "Not found"}),
                &cors_headers,
            ),
        });
    }

    let response_body = match (method.clone(), path.as_str()) {
        (Method::GET, "/mcp/health") => json_response(
            StatusCode::OK,
            json!({ "status": "ok", "service": "mcp" }),
            &cors_headers,
        ),

        (Method::POST, "/mcp") => {
            let body_bytes = match req.collect().await {
                Ok(b) => b.to_bytes(),
                Err(e) => {
                    error!("Failed to read MCP request body: {}", e);
                    return Ok(json_response(
                        StatusCode::BAD_REQUEST,
                        json!({ "error": "Failed to read body" }),
                        &cors_headers,
                    ));
                }
            };

            match serde_json::from_slice::<JsonRpcRequest>(&body_bytes) {
                Ok(rpc) => {
                    let response = dispatch_rpc(rpc, &state.registry).await;
                    let body = serde_json::to_vec(&response).unwrap_or_default();
                    json_bytes_response(StatusCode::OK, body, &cors_headers)
                }
                Err(e) => {
                    let resp =
                        JsonRpcResponse::err(None, -32700, format!("Parse error: {}", e));
                    let body = serde_json::to_vec(&resp).unwrap_or_default();
                    json_bytes_response(StatusCode::OK, body, &cors_headers)
                }
            }
        }

        _ => json_response(
            StatusCode::NOT_FOUND,
            json!({ "error": "Not found" }),
            &cors_headers,
        ),
    };

    Ok(response_body)
}

// ── Public share download ─────────────────────────────────────────────────────

async fn handle_public_download(
    token: &str,
    state: &McpServerState,
    cors_headers: &[(&str, &str)],
) -> hyper::Response<http_body_util::Full<bytes::Bytes>> {
    use hyper::StatusCode;

    let share_repo = DbShareRepository::new(state.db.clone());
    let user_repo = crate::database::user_repository::UserRepository::new(state.db.clone());
    let file_repo = DbFileRepository::new(state.db.clone());

    // 1. Resolve token
    let link = match share_repo.find_link_by_token(token).await {
        Ok(Some(l)) => l,
        Ok(None) => {
            return json_response(
                StatusCode::NOT_FOUND,
                json!({"error": "Share link not found or expired"}),
                cors_headers,
            )
        }
        Err(e) => {
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": e.to_string()}),
                cors_headers,
            )
        }
    };

    // 2. Check permissions
    if !link.can_read || !link.is_active {
        return json_response(
            StatusCode::FORBIDDEN,
            json!({"error": "This link does not grant read access"}),
            cors_headers,
        );
    }

    // 3. Check expiry
    if let Some(exp) = link.expires_at {
        if exp < chrono::Utc::now() {
            return json_response(
                StatusCode::GONE,
                json!({"error": "Share link has expired"}),
                cors_headers,
            );
        }
    }

    // 4. Resolve the file metadata from DB
    let file_meta = match file_repo.find_by_id(link.file_id).await {
        Ok(Some(f)) => f,
        Ok(None) => {
            return json_response(
                StatusCode::NOT_FOUND,
                json!({"error": "File not found"}),
                cors_headers,
            )
        }
        Err(e) => {
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": e.to_string()}),
                cors_headers,
            )
        }
    };

    // 5. Resolve the owner to get password_hash for decryption
    let owner = match user_repo.find_by_id(file_meta.owner_id).await {
        Ok(Some(u)) => u,
        _ => {
             return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": "Owner not found"}),
                cors_headers,
            )
        }
    };

    // 6. Use DownloadUseCase to get data (includes transparent decryption/decompression)
    // We mock a User object with the owner's credentials to satisfy the UseCase
    let user_domain = crate::user::domain::User {
        username: owner.username.clone(),
        password_hash: owner.password_hash.clone(),
    };
    
    // We need DownloadUseCase. In McpServerState we have file_service.
    // DownloadUseCase is internal to FileService but we can use file_service.download()
    let data = match state.file_service.download(&user_domain, "/", &file_meta.filename).await {
        Ok(d) => d,
        Err(e) => {
            return json_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e.to_string()}), cors_headers);
        }
    };

    // 7. Serve file with content-type and content-disposition
    let mime = file_meta
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".into());
    let filename = file_meta.filename;

    let mut resp =
        hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(data)));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_str(&mime)
            .unwrap_or(hyper::header::HeaderValue::from_static("application/octet-stream")),
    );
    resp.headers_mut().insert(
        hyper::header::HeaderName::from_static("content-disposition"),
        hyper::header::HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            filename
        ))
        .unwrap_or(hyper::header::HeaderValue::from_static("attachment")),
    );
    for (k, v) in cors_headers {
        if let (Ok(name), Ok(val)) = (
            hyper::header::HeaderName::from_bytes(k.as_bytes()),
            hyper::header::HeaderValue::from_str(v),
        ) {
            resp.headers_mut().insert(name, val);
        }
    }
    resp
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|kv| {
        let mut parts = kv.splitn(2, '=');
        if parts.next() == Some(key) {
            parts.next()
        } else {
            None
        }
    })
}

// ── JSON-RPC dispatch ─────────────────────────────────────────────────────────

async fn dispatch_rpc(req: JsonRpcRequest, registry: &Arc<McpRegistry>) -> JsonRpcResponse {
    let id = req.id.clone();

    if req.jsonrpc != "2.0" {
        return JsonRpcResponse::err(id, -32600, "Invalid Request: expected jsonrpc 2.0");
    }

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".into(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability {
                        list_changed: false,
                    },
                    resources: ResourcesCapability {
                        subscribe: false,
                        list_changed: false,
                    },
                    prompts: PromptsCapability {
                        list_changed: false,
                    },
                },
                server_info: ServerInfo {
                    name: "tcp-framework-mcp".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                },
            };
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        "notifications/initialized" => JsonRpcResponse::ok(id, json!({})),

        "tools/list" => {
            let tools = registry.list_tools();
            JsonRpcResponse::ok(id, json!({ "tools": tools }))
        }

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Object(serde_json::Map::new()));
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            let result = registry.call_tool(name, &args).await;
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        "resources/list" => {
            let params = req.params.unwrap_or(Value::Object(serde_json::Map::new()));
            let username = params
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("alice");
            let resources = registry.list_resources(username).await;
            JsonRpcResponse::ok(id, json!({ "resources": resources }))
        }

        "resources/read" => {
            let params = req.params.unwrap_or(Value::Object(serde_json::Map::new()));
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            match registry.read_resource(uri).await {
                Ok(content) => JsonRpcResponse::ok(id, json!({ "contents": [content] })),
                Err(e) => JsonRpcResponse::err(id, -32000, e),
            }
        }

        "prompts/list" => {
            let prompts = registry.list_prompts();
            JsonRpcResponse::ok(id, json!({ "prompts": prompts }))
        }

        "prompts/get" => {
            let params = req.params.unwrap_or(Value::Object(serde_json::Map::new()));
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            match registry.get_prompt(name, &args).await {
                Ok(text) => JsonRpcResponse::ok(
                    id,
                    json!({
                        "description": "System prompt generated",
                        "messages": [
                            { "role": "user", "content": { "type": "text", "text": text } }
                        ]
                    }),
                ),
                Err(e) => JsonRpcResponse::err(id, -32000, e),
            }
        }

        "ping" => JsonRpcResponse::ok(id, json!({})),

        method => JsonRpcResponse::err(id, -32601, format!("Method not found: {}", method)),
    }
}

fn json_response(
    status: hyper::StatusCode,
    body: Value,
    cors: &[(&str, &str)],
) -> hyper::Response<http_body_util::Full<bytes::Bytes>> {
    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    json_bytes_response(status, bytes, cors)
}

fn json_bytes_response(
    status: hyper::StatusCode,
    body: Vec<u8>,
    cors: &[(&str, &str)],
) -> hyper::Response<http_body_util::Full<bytes::Bytes>> {
    let mut resp = hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(body)));
    *resp.status_mut() = status;
    resp.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    for (k, v) in cors {
        resp.headers_mut().insert(
            hyper::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
            hyper::header::HeaderValue::from_str(v).unwrap(),
        );
    }
    resp
}
