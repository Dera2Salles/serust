export async function callMcpTool(name: string, args: any = {}) {
  const response = await fetch('http://localhost:8081/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: Math.random().toString(36).substring(7),
      method: 'tools/call',
      params: {
        name,
        arguments: args
      }
    }),
  });

  const json = await response.json();
  if (json.error) {
    throw new Error(json.error.message);
  }
  
  // result.content is an array of objects with { type, text }
  return json.result;
}
