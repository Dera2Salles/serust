
use crate::mcp::registry::McpRegistry;
use crate::mcp::types::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse,
    PromptsCapability, ResourcesCapability,
    ServerCapabilities, ServerInfo, ToolsCapability,
};
use serde_json::{json, Value};
use std::convert::Infallible;

use std::sync::Arc;
use tracing::{error, info};


/// Start the MCP HTTP server on a separate port.
pub async fn run_mcp_server(
    registry: Arc<McpRegistry>,
    addr: &str,
) -> anyhow::Result<()> {
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(addr).await?;
    info!("MCP server listening on http://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let registry = Arc::clone(&registry);

        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let registry = Arc::clone(&registry);
                handle_http(req, registry)
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                error!("MCP connection error from {}: {}", peer, e);
            }
        });
    }
}


async fn handle_http(
    req: hyper::Request<hyper::body::Incoming>,
    registry: Arc<McpRegistry>,
) -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, Infallible> {
    use http_body_util::{BodyExt, Full};
    use hyper::{Method, StatusCode};

    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let cors_headers = [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
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

    let response_body = match (method.clone(), path.as_str()) {
        (Method::GET, "/mcp/health") => {
            json_response(StatusCode::OK, json!({ "status": "ok", "service": "mcp" }), &cors_headers)
        }

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
                    let response = dispatch_rpc(rpc, &registry).await;
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

        _ => json_response(StatusCode::NOT_FOUND, json!({ "error": "Not found" }), &cors_headers),
    };

    Ok(response_body)
}


async fn dispatch_rpc(
    req: JsonRpcRequest,
    registry: &Arc<McpRegistry>,
) -> JsonRpcResponse {
    let id = req.id.clone();

    if req.jsonrpc != "2.0" {
        return JsonRpcResponse::err(id, -32600, "Invalid Request: expected jsonrpc 2.0");
    }

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".into(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability { list_changed: false },
                    resources: ResourcesCapability { subscribe: false, list_changed: false },
                    prompts: PromptsCapability { list_changed: false },
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
            let args = params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
            
            let result = registry.call_tool(name, &args).await;
            JsonRpcResponse::ok(id, serde_json::to_value(result).unwrap())
        }

        "resources/list" => {
            let params = req.params.unwrap_or(Value::Object(serde_json::Map::new()));
            let username = params.get("username").and_then(|v| v.as_str()).unwrap_or("alice");
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
            let args = params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
            
            match registry.get_prompt(name, &args).await {
                Ok(text) => JsonRpcResponse::ok(id, json!({ 
                    "description": "System prompt generated",
                    "messages": [
                        { "role": "user", "content": { "type": "text", "text": text } }
                    ]
                })),
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
