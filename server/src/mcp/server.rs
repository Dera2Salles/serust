use crate::database::analytics_repository::AnalyticsRepository;
use crate::database::entities::{access_log, read_counters};
use crate::database::interfaces::{
    IFileDatabaseRepository, IShareDatabaseRepository, IUserRepository,
};
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
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info};

pub struct McpServerState {
    pub registry: Arc<McpRegistry>,
    pub file_service: Arc<FileService>,
    pub auth_service: Arc<crate::user::service::AuthService>,
    pub log_access_usecase: Arc<crate::database::log_usecases::LogAccessUseCase>,
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

    if method == Method::GET && path.starts_with("/public/") {
        let token = path.trim_start_matches("/public/");
        return Ok(handle_public_download(token, &state, &cors_headers, &query).await);
    }

    if method == Method::GET && path == "/api/server/status" {
        let sessions = json!([
            {
                "peer_addr": "192.168.1.15:54321",
                "connected_at": "2026-06-03T10:15:00Z",
                "last_command": "LIST /photos",
                "username": "alice"
            },
            {
                "peer_addr": "10.0.0.5:12345",
                "connected_at": "2026-06-03T10:20:00Z",
                "last_command": "GET /docs/plan.pdf",
                "username": "bob"
            },
            {
                "peer_addr": "172.16.0.2:44332",
                "connected_at": "2026-06-03T10:22:00Z",
                "last_command": "AUTH",
                "username": null
            }
        ]);
        return Ok(json_response(
            StatusCode::OK,
            json!({"status": "ok", "sessions": sessions}),
            &cors_headers,
        ));
    }

    if method == Method::GET && path == "/api/users/search" {
        let query_val = extract_query_param(&query, "query").unwrap_or("");
        let user_repo = crate::database::user_repository::UserRepository::new(state.db.clone());
        return Ok(match user_repo.search_users(query_val).await {
            Ok(users) => {
                let list: Vec<Value> = users
                    .into_iter()
                    .map(|u| json!({ "username": u.username }))
                    .collect();
                json_response(StatusCode::OK, json!({ "users": list }), &cors_headers)
            }
            Err(e) => json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": e.to_string() }),
                &cors_headers,
            ),
        });
    }

    if method == Method::GET && path == "/api/storage/stats" {
        let username = extract_query_param(&query, "username").unwrap_or("alice");
        let user_repo = crate::database::user_repository::UserRepository::new(state.db.clone());
        let db_user = user_repo.find_by_username(username).await.unwrap_or(None);
        let user = crate::user::domain::User {
            id: db_user
                .as_ref()
                .map(|u| u.id)
                .unwrap_or_else(uuid::Uuid::nil),
            username: username.to_string(),
            password_hash: String::new(),
            email: db_user
                .as_ref()
                .map(|u| u.email.clone())
                .unwrap_or_default(),
            first_name: None,
            last_name: None,
            birth_date: None,
            location: None,
            profile_pic_path: db_user.and_then(|u| u.profile_pic_path),
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

    if method == Method::POST && path == "/api/auth/register" {
        let body_bytes = match req.collect().await {
            Ok(b) => b.to_bytes(),
            Err(e) => {
                error!("Failed to read registration body: {}", e);
                return Ok(json_response(
                    StatusCode::BAD_REQUEST,
                    json!({ "error": "Failed to read body" }),
                    &cors_headers,
                ));
            }
        };

        let reg_data: Value = match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(e) => {
                return Ok(json_response(
                    StatusCode::BAD_REQUEST,
                    json!({ "error": format!("Invalid JSON: {}", e) }),
                    &cors_headers,
                ));
            }
        };

        let username = reg_data["username"].as_str().unwrap_or("");
        let email = reg_data["email"].as_str().unwrap_or("");
        let password = reg_data["password"].as_str().unwrap_or("");
        let first_name = reg_data["first_name"].as_str().map(|s| s.to_string());
        let last_name = reg_data["last_name"].as_str().map(|s| s.to_string());
        let birth_date = reg_data["birth_date"].as_str().map(|s| s.to_string());
        let location = reg_data["location"].as_str().map(|s| s.to_string());

        if username.is_empty() || email.is_empty() || password.is_empty() {
            return Ok(json_response(
                StatusCode::BAD_REQUEST,
                json!({ "error": "Username, email, and password are required" }),
                &cors_headers,
            ));
        }

        return Ok(
            match state
                .auth_service
                .register(
                    username, email, password, first_name, last_name, birth_date, location,
                )
                .await
            {
                Ok(_) => json_response(
                    StatusCode::CREATED,
                    json!({ "status": "ok" }),
                    &cors_headers,
                ),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": e.to_string() }),
                    &cors_headers,
                ),
            },
        );
    }

    if method == Method::POST && path == "/api/auth/login" {
        let body_bytes = match req.collect().await {
            Ok(b) => b.to_bytes(),
            Err(e) => {
                error!("Failed to read login body: {}", e);
                return Ok(json_response(
                    StatusCode::BAD_REQUEST,
                    json!({ "error": "Failed to read body" }),
                    &cors_headers,
                ));
            }
        };

        let login_data: Value = match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(_) => {
                return Ok(json_response(
                    StatusCode::BAD_REQUEST,
                    json!({ "error": "Invalid JSON" }),
                    &cors_headers,
                ));
            }
        };

        let email = login_data["email"].as_str().unwrap_or("");
        let password = login_data["password"].as_str().unwrap_or("");

        return Ok(match state.auth_service.login(email, password).await {
            Ok(user) => json_response(StatusCode::OK, json!(user), &cors_headers),
            Err(e) => {
                let status = match e {
                    crate::common::error::DomainError::PendingApproval => StatusCode::FORBIDDEN,
                    _ => StatusCode::UNAUTHORIZED,
                };
                json_response(status, json!({ "error": e.to_string() }), &cors_headers)
            }
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
                    let resp = JsonRpcResponse::err(None, -32700, format!("Parse error: {}", e));
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

async fn handle_public_download(
    token: &str,
    state: &McpServerState,
    cors_headers: &[(&str, &str)],
    query_params: &str,
) -> hyper::Response<http_body_util::Full<bytes::Bytes>> {
    use hyper::StatusCode;

    let share_repo = DbShareRepository::new(state.db.clone());
    let user_repo = crate::database::user_repository::UserRepository::new(state.db.clone());
    let file_repo = DbFileRepository::new(state.db.clone());

    let link = match share_repo.find_link_by_token(token).await {
        Ok(Some(l)) => l,
        Ok(None) => {
            return json_response(
                StatusCode::NOT_FOUND,
                json!({"error": "Share link not found or inactive"}),
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

    if !link.can_read || !link.is_active {
        return json_response(
            StatusCode::FORBIDDEN,
            json!({"error": "This link is disabled"}),
            cors_headers,
        );
    }

    if let Some(exp) = link.expires_at {
        if exp < chrono::Utc::now() {
            return json_response(
                StatusCode::GONE,
                json!({"error": "Share link has expired"}),
                cors_headers,
            );
        }
    }

    if let Some(ref hash) = link.password_hash {
        let password = extract_query_param(query_params, "password").unwrap_or("");
        if crate::user::service::AuthService::hash_password(password) != *hash {
            return json_response(
                StatusCode::UNAUTHORIZED,
                json!({"error": "Invalid password", "requires_password": true}),
                cors_headers,
            );
        }
    }

    let read_count_model = read_counters::Entity::find()
        .filter(read_counters::Column::ShareLinkId.eq(link.id.to_string()))
        .one(&state.db.connection)
        .await
        .unwrap_or(None);

    if let (Some(max), Some(model)) = (link.max_reads, read_count_model) {
        if model.read_count >= max {
            return json_response(
                StatusCode::GONE,
                json!({"error": "Maximum read count reached"}),
                cors_headers,
            );
        }
    }

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

    let user_domain = crate::user::domain::User {
        id: owner.id,
        username: owner.username.clone(),
        password_hash: owner.password_hash.clone(),
        email: owner.email.clone(),
        first_name: owner.first_name.clone(),
        last_name: owner.last_name.clone(),
        birth_date: owner.birth_date.clone(),
        location: owner.location.clone(),
        profile_pic_path: owner.profile_pic_path.clone(),
    };

    let data = match state
        .file_service
        .download(&user_domain, "/", &file_meta.filename)
        .await
    {
        Ok(d) => {
            let _ = state.log_access_usecase.execute(&crate::database::domain::DbAccessLog {
                id: 0,
                file_id: file_meta.id,
                accessed_by: None,
                share_link_id: Some(link.id),
                grant_id: None,
                action: "read".into(),
                accessed_at: chrono::Utc::now(),
                ip_address: None,
                user_agent: None,
                bytes_transferred: Some(d.len() as i64),
            }).await;

            d
        }
        Err(e) => {
            return json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"error": e.to_string()}),
                cors_headers,
            );
        }
    };

    let mime = file_meta
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".into());
    let filename = file_meta.filename;

    let mut resp = hyper::Response::new(http_body_util::Full::new(bytes::Bytes::from(data)));
    *resp.status_mut() = StatusCode::OK;
    resp.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_str(&mime).unwrap_or(
            hyper::header::HeaderValue::from_static("application/octet-stream"),
        ),
    );
    resp.headers_mut().insert(
        hyper::header::HeaderName::from_static("content-disposition"),
        hyper::header::HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
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
