use super::protocol::{JsonRpcRequest, JsonRpcResponse, ToolRegistry};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct StdioTransport {
    registry: Arc<ToolRegistry>,
}

impl StdioTransport {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        let mut line = String::new();

        eprintln!("[CodeAtlas MCP] Stdio transport started");

        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(req) => req,
                Err(err) => {
                    eprintln!("[CodeAtlas MCP] Malformed JSON: {}", err);
                    let resp = JsonRpcResponse::error(None, -32700, "Parse error".to_string());
                    let mut bytes = serde_json::to_vec(&resp)?;
                    bytes.push(b'\n');
                    stdout.write_all(&bytes).await?;
                    stdout.flush().await?;
                    line.clear();
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            if let Some(resp) = response {
                let mut bytes = serde_json::to_vec(&resp)?;
                bytes.push(b'\n');
                stdout.write_all(&bytes).await?;
                stdout.flush().await?;
            }

            line.clear();
        }

        Ok(())
    }

    async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "codeatlas-mcp",
                        "version": "3.0.1"
                    }
                });
                Some(JsonRpcResponse::success(req.id, result))
            }
            "notifications/initialized" => {
                eprintln!("[CodeAtlas MCP] Client initialized notification received");
                None
            }
            "ping" => Some(JsonRpcResponse::success(req.id, json!({}))),
            "tools/list" => {
                let tools_json: Vec<Value> = self
                    .registry
                    .list_tools()
                    .iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema,
                        })
                    })
                    .collect();

                Some(JsonRpcResponse::success(
                    req.id,
                    json!({ "tools": tools_json }),
                ))
            }
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.registry.dispatch(tool_name, arguments).await {
                    Ok(text) => {
                        let result = json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": text
                                }
                            ]
                        });
                        Some(JsonRpcResponse::success(req.id, result))
                    }
                    Err(err_msg) => {
                        let result = json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Error: {}", err_msg)
                                }
                            ],
                            "isError": true
                        });
                        Some(JsonRpcResponse::success(req.id, result))
                    }
                }
            }
            _ if req.id.is_none() => None,
            _ => Some(JsonRpcResponse::error(
                req.id,
                -32601,
                format!("Method not found: {}", req.method),
            )),
        }
    }
}
