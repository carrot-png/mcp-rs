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
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;

use crate::tools::{self, python::PythonScript, search::WebSearch};

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[derive(Clone)]
struct McpServer;

#[tool_router]
impl McpServer {
    #[tool(description = r#"
        Run Python code using a lightweight sandboxed Monty interpreter.
        Supports core Python syntax and a very small subset of the standard library: math, re, json
        Does NOT support:
        - itertools, file I/O, classes, or random
        - external libraries such as numpy, pandas, or matplotlib.
    "#)]
    pub async fn run_python(&self, params: Parameters<PythonScript>) -> CallToolResult {
        tools::python::run_python(params.0.code)
    }

    #[tool(description = "Get the current date and time.")]
    pub async fn datetime(&self) -> CallToolResult {
        tools::datetime::datetime()
    }

    #[cfg(feature = "searxng")]
    #[tool(description = "Web search")]
    pub async fn web_search(&self, web_search: Parameters<WebSearch>) -> CallToolResult {
        tools::search::query(web_search.0).await
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
    let ct = CancellationToken::new();

    let mut config = StreamableHttpServerConfig::default().with_cancellation_token(ct.clone());

    if let Ok(host) = std::env::var("BASE_URL") {
        config = config.with_allowed_hosts(vec![host]);
    }

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
        .with_graceful_shutdown(shutdown_signal(ct.clone()))
        .await?;
    Ok(())
}

async fn shutdown_signal(ct: CancellationToken) {
    let sigint = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    tokio::select! {
        _ = sigint => println!("Received SIGINT"),
        _ = sigterm => println!("Received SIGTERM"),
    }
    println!("Shutting down...");
    ct.cancel();
}
