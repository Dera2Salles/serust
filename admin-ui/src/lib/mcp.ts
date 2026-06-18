/**
 * Helper to call MCP tools via the JSON-RPC bridge on port 8081
 */
export async function callMcpTool(method: string, params: any = {}) {
  const user = JSON.parse(sessionStorage.getItem("admin_user") || "{}");
  const token = user?.token;

  const response = await fetch("http://localhost:8081/mcp", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: Date.now(),
      method: "call_tool",
      params: {
        name: method,
        arguments: params,
      },
    }),
  });

  if (!response.ok) {
    throw new Error(`MCP error: ${response.statusText}`);
  }

  const data = await response.json();

  if (data.error) {
    throw new Error(data.error.message || "Unknown MCP error");
  }

  // MCP 'call_tool' returns { content: [{ type: 'text', text: '...' }] }
  return data.result;
}
