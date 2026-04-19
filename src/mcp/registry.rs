use crate::file::service::FileService;
use crate::mcp::types::{
    McpPrompt, McpPromptArgument, McpResource, McpResourceContent, McpTool, McpToolResult,
};
use crate::user::domain::User;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::info;

pub struct McpRegistry {
    file_service: Arc<FileService>,
}

impl McpRegistry {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }

    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "list_directory".into(),
                description: "List files and folders in a directory on the FTP server.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path": { "type": "string", "description": "Absolute path to list" }
                    },
                    "required": ["username", "path"]
                }),
            },
            McpTool {
                name: "get_storage_stats".into(),
                description: "Get storage statistics for a user.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" }
                    },
                    "required": ["username"]
                }),
            },
            McpTool {
                name: "create_folder".into(),
                description: "Create a new folder on the FTP server.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path": { "type": "string", "description": "Parent directory path" },
                        "name": { "type": "string", "description": "Name of the new folder" }
                    },
                    "required": ["username", "path", "name"]
                }),
            },
            McpTool {
                name: "create_file".into(),
                description: "Create a new text file on the FTP server.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path": { "type": "string", "description": "Directory where the file will be created" },
                        "filename": { "type": "string", "description": "Name of the new file" },
                        "content": { "type": "string", "description": "Text content of the file" }
                    },
                    "required": ["username", "path", "filename", "content"]
                }),
            },
            McpTool {
                name: "delete_file".into(),
                description: "Delete a file from the FTP server.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path": { "type": "string", "description": "Directory containing the file" },
                        "filename": { "type": "string", "description": "Name of the file to delete" }
                    },
                    "required": ["username", "path", "filename"]
                }),
            },
            McpTool {
                name: "search_files".into(),
                description: "Search for files matching a pattern across the entire storage."
                    .into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "query": { "type": "string", "description": "Search term" },
                        "path": { "type": "string", "description": "Root path to search from", "default": "/" }
                    },
                    "required": ["username", "query"]
                }),
            },
            McpTool {
                name: "rename_file".into(),
                description: "Rename a file or folder on the FTP server.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path": { "type": "string", "description": "Directory containing the file or folder" },
                        "old_name": { "type": "string", "description": "Current name of the file or folder" },
                        "new_name": { "type": "string", "description": "New name of the file or folder" }
                    },
                    "required": ["username", "path", "old_name", "new_name"]
                }),
            },
            McpTool {
                name: "move_file".into(),
                description: "Move a file or folder from one location to another.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "source_path": { "type": "string", "description": "Current path of the file or folder" },
                        "destination_path": { "type": "string", "description": "New destination path" }
                    },
                    "required": ["username", "source_path", "destination_path"]
                }),
            },
            McpTool {
                name: "read_file".into(),
                description: "Read the text content of a file on the FTP server. Use this to answer questions about file contents, summarize documents, or analyze code.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "username": { "type": "string", "description": "Authenticated username" },
                        "path":     { "type": "string", "description": "Directory containing the file" },
                        "filename": { "type": "string", "description": "Name of the file to read" }
                    },
                    "required": ["username", "path", "filename"]
                }),
            },
        ]
    }

    pub async fn call_tool(&self, name: &str, args: &Value) -> McpToolResult {
        info!("MCP tool call: {} args={}", name, args);
        match name {
            "list_directory" => self.tool_list_directory(args).await,
            "get_storage_stats" => self.tool_get_storage_stats(args).await,
            "create_folder" => self.tool_create_folder(args).await,
            "create_file" => self.tool_create_file(args).await,
            "delete_file" => self.tool_delete_file(args).await,
            "search_files" => self.tool_search_files(args).await,
            "rename_file" => self.tool_rename_file(args).await,
            "move_file" => self.tool_move_file(args).await,
            "read_file" => self.tool_read_file(args).await,
            _ => McpToolResult::error(format!("Unknown tool: {}", name)),
        }
    }

    pub async fn list_resources(&self, username: &str) -> Vec<McpResource> {
        let user = Self::make_user(username);
        let mut resources = Vec::new();
        Box::pin(self.collect_resources(username, &user, "/", 0, &mut resources)).await;
        resources
    }

    async fn collect_resources(
        &self,
        username: &str,
        user: &User,
        path: &str,
        depth: usize,
        out: &mut Vec<McpResource>,
    ) {
        const MAX_DEPTH: usize = 4;
        if depth > MAX_DEPTH {
            return;
        }
        let entries = match self.file_service.list(user, path).await {
            Ok(e) => e,
            Err(_) => return,
        };
        for (name, is_dir) in entries {
            let full_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            if is_dir {
                Box::pin(self.collect_resources(username, user, &full_path, depth + 1, out)).await;
            } else {
                // Only expose text-readable files to the AI
                if Self::is_text_readable(&name) {
                    let uri = format!("ftp://{}{}", username, full_path);
                    out.push(McpResource {
                        uri: uri.clone(),
                        name: name.clone(),
                        description: Some(format!("File at '{}'", full_path)),
                        mime_type: Some(Self::guess_mime(&name).into()),
                    });
                }
            }
        }
    }

    fn is_text_readable(name: &str) -> bool {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        matches!(
            ext.as_str(),
            "txt" | "md" | "json" | "yaml" | "yml" | "toml" | "xml" | "html" | "htm"
                | "css" | "js" | "ts" | "rs" | "dart" | "py" | "go" | "java" | "kt"
                | "swift" | "c" | "cpp" | "h" | "sh" | "bash" | "env" | "csv"
                | "log" | "conf" | "ini" | "sql" | "graphql" | "proto"
        )
    }

    fn guess_mime(name: &str) -> &'static str {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "json" => "application/json",
            "xml" => "application/xml",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" | "ts" => "text/javascript",
            "csv" => "text/csv",
            "sql" => "application/sql",
            _ => "text/plain",
        }
    }

    pub async fn read_resource(&self, uri: &str) -> Result<McpResourceContent, String> {
        let rest = uri
            .strip_prefix("ftp://")
            .ok_or_else(|| format!("Invalid URI scheme: {}", uri))?;

        let mut parts = rest.splitn(2, '/');
        let username = parts.next().ok_or("Missing username in URI")?;
        let path = parts.next().unwrap_or("");

        let user = Self::make_user(username);

        let (dir, filename) = if path.contains('/') {
            path.rsplit_once('/').unwrap()
        } else {
            ("/", path)
        };

        match self.file_service.download(&user, dir, filename).await {
            Ok(data) => {
                let text = String::from_utf8(data)
                    .unwrap_or_else(|_| "Binary data (cannot display as text)".into());
                Ok(McpResourceContent {
                    uri: uri.to_string(),
                    mime_type: Some("text/plain".into()),
                    text,
                })
            }
            Err(e) => Err(format!("Failed to read resource {}: {}", uri, e)),
        }
    }

    pub fn list_prompts(&self) -> Vec<McpPrompt> {
        vec![McpPrompt {
            name: "analyze_storage".into(),
            description: Some(
                "Provides a summary of the user's storage and suggests organization.".into(),
            ),
            arguments: Some(vec![McpPromptArgument {
                name: "username".into(),
                description: Some("Target username".into()),
                required: true,
            }]),
        }]
    }

    pub async fn get_prompt(&self, name: &str, args: &Value) -> Result<String, String> {
        match name {
            "analyze_storage" => {
                let username = Self::get_str(args, "username")?;
                let user = Self::make_user(username);
                let (total_bytes, file_count, dir_count) = self.recursive_stats(&user, "/").await;

                Ok(format!(
                    "User '{}' has {} files and {} folders using {}.\n\
                     Please analyze the content and suggest if any organization is needed.",
                    username,
                    file_count,
                    dir_count,
                    Self::format_size(total_bytes)
                ))
            }
            _ => Err(format!("Unknown prompt: {}", name)),
        }
    }

    fn make_user(username: &str) -> User {
        User {
            username: username.to_string(),
            password_hash: String::new(),
        }
    }

    fn get_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
        args.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing required argument: {}", key))
    }

    fn format_size(bytes: u64) -> String {
        if bytes < 1_024 {
            format!("{} B", bytes)
        } else if bytes < 1_024 * 1_024 {
            format!("{:.1} KB", bytes as f64 / 1_024.0)
        } else if bytes < 1_024 * 1_024 * 1_024 {
            format!("{:.1} MB", bytes as f64 / (1_024.0 * 1_024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1_024.0 * 1_024.0 * 1_024.0))
        }
    }

    async fn tool_list_directory(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        match self.file_service.list(&user, path).await {
            Ok(entries) => {
                let mut lines = vec![format!("Contents of '{}':", path)];
                for (name, is_dir) in &entries {
                    let icon = if *is_dir { "📁" } else { "📄" };
                    let mut line = format!("  {} {}", icon, name);
                    
                    if !is_dir {
                        if let Ok(Some((size, _, Some(checksum)))) = self.file_service.stat(&user, path, name).await {
                            line.push_str(&format!(" (size: {}, checksum: {})", size, checksum));
                        }
                    }
                    lines.push(line);
                }
                McpToolResult::success(lines.join("\n"))
            }
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    async fn tool_get_storage_stats(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let user = Self::make_user(username);
        let (total_bytes, file_count, dir_count) = self.recursive_stats(&user, "/").await;

        McpToolResult::success(format!(
            "Storage for '{}': {} used, {} files, {} folders.",
            username,
            Self::format_size(total_bytes),
            file_count,
            dir_count
        ))
    }

    async fn tool_create_folder(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let name = match Self::get_str(args, "name") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        match self.file_service.mkdir(&user, path, name).await {
            Ok(_) => McpToolResult::success(format!("Folder '{}' created.", name)),
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    async fn tool_create_file(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let filename = match Self::get_str(args, "filename") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let content = match Self::get_str(args, "content") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        let data = content.as_bytes().to_vec();
        let size = data.len() as u64;

        match self.file_service.upload(&user, path, filename, size, data).await {
            Ok(_) => McpToolResult::success(format!("File '{}' created.", filename)),
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    async fn tool_delete_file(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let filename = match Self::get_str(args, "filename") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        match self.file_service.delete(&user, path, filename).await {
            Ok(_) => McpToolResult::success(format!("File '{}' deleted.", filename)),
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    async fn tool_search_files(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let query = match Self::get_str(args, "query") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or("/");

        let user = Self::make_user(username);
        let matches = self
            .recursive_search(&user, root, &query.to_lowercase())
            .await;

        if matches.is_empty() {
            McpToolResult::success("No matches found.")
        } else {
            McpToolResult::success(format!("Matches:\n{}", matches.join("\n")))
        }
    }

    async fn tool_rename_file(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let old_name = match Self::get_str(args, "old_name") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let new_name = match Self::get_str(args, "new_name") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        match self.file_service.rename(&user, path, old_name, new_name).await {
            Ok(_) => McpToolResult::success(format!("Renamed '{}' to '{}'.", old_name, new_name)),
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    async fn tool_read_file(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let path = match Self::get_str(args, "path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let filename = match Self::get_str(args, "filename") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        const MAX_PREVIEW_BYTES: usize = 128 * 1024; // 128 KB cap for AI context
        match self.file_service.download(&user, path, filename).await {
            Ok(data) => {
                let truncated = data.len() > MAX_PREVIEW_BYTES;
                let slice = &data[..data.len().min(MAX_PREVIEW_BYTES)];
                let text = String::from_utf8_lossy(slice).to_string();
                let content = if truncated {
                    format!("{}\n\n[... file truncated at 128 KB ...]", text)
                } else {
                    text
                };
                McpToolResult::success(content)
            }
            Err(e) => McpToolResult::error(format!("Cannot read '{}': {}", filename, e)),
        }
    }

    async fn tool_move_file(&self, args: &Value) -> McpToolResult {
        let username = match Self::get_str(args, "username") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let source_path = match Self::get_str(args, "source_path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };
        let destination_path = match Self::get_str(args, "destination_path") {
            Ok(v) => v,
            Err(e) => return McpToolResult::error(e),
        };

        let user = Self::make_user(username);
        match self.file_service.rename(&user, "/", source_path, destination_path).await {
            Ok(_) => McpToolResult::success(format!("Moved '{}' to '{}'.", source_path, destination_path)),
            Err(e) => McpToolResult::error(format!("Error: {}", e)),
        }
    }

    pub async fn recursive_stats(&self, user: &User, path: &str) -> (u64, u64, u64) {
        let entries = match self.file_service.list(user, path).await {
            Ok(e) => e,
            Err(_) => return (0, 0, 0),
        };
        let mut tb = 0;
        let mut fc = 0;
        let mut dc = 0;
        for (name, is_dir) in &entries {
            if *is_dir {
                dc += 1;
                let sub = if path == "/" {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", path, name)
                };
                let (b, f, d) = Box::pin(self.recursive_stats(user, &sub)).await;
                tb += b;
                fc += f;
                dc += d;
            } else {
                fc += 1;
                if let Ok(Some((s, _, _))) = self.file_service.stat(user, path, name).await {
                    tb += s;
                }
            }
        }
        (tb, fc, dc)
    }

    async fn recursive_search(&self, user: &User, path: &str, query: &str) -> Vec<String> {
        let entries = match self.file_service.list(user, path).await {
            Ok(e) => e,
            Err(_) => return vec![],
        };
        let mut results = Vec::new();
        for (name, is_dir) in &entries {
            let full = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };
            if name.to_lowercase().contains(query) {
                results.push(format!("{} {}", if *is_dir { "📁" } else { "📄" }, full));
            }
            if *is_dir {
                results.extend(Box::pin(self.recursive_search(user, &full, query)).await);
            }
        }
        results
    }
}
