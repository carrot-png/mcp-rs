use axum::Router;
use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[derive(Clone)]
struct McpServer;

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct PythonScript {
    code: String,
}

#[tool_router]
impl McpServer {
    #[tool(description = "Run Python code in a sandbox")]
    pub async fn run_python(&self, params: Parameters<PythonScript>) -> CallToolResult {
        crate::python::run_python(params.0.code)
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("A simple MCP tool server")
    }
}

pub async fn run() -> anyhow::Result<()> {
    let server = McpServer;
    let ct = tokio_util::sync::CancellationToken::new();

    let config = StreamableHttpServerConfig::default().with_cancellation_token(ct.clone());

    let mcp_service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        config,
    );

    let app = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(BIND_ADDRESS).await?;
    println!("MCP endpoint:  http://{}/mcp", BIND_ADDRESS);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await?;
    Ok(())
}
