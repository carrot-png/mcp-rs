use axum::Router;
use rmcp::{
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;

#[cfg(feature = "searxng")]
use crate::tools::search::WebSearch;

#[cfg(feature = "fetch")]
use crate::tools::fetch::WebFetch;

use crate::tools::{self, python::PythonScript};

const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[derive(Clone)]
struct McpServer {
    #[cfg(feature = "searxng")]
    search_config: tools::search::SearchConfig,

    #[cfg(feature = "fetch")]
    fetch_client: reqwest::Client,
}

impl McpServer {
    fn new() -> Self {
        Self {
            #[cfg(feature = "searxng")]
            search_config: tools::search::SearchConfig::new(),

            #[cfg(feature = "fetch")]
            fetch_client: tools::fetch::get_client(),
        }
    }
}

impl McpServer {
    // Manually implement #[tool_router]. rmcp crate macro does not respect cfg feature flags
    fn tool_router() -> ToolRouter<Self> {
        let router = ToolRouter::<Self>::new()
            .with_route((Self::run_python_tool_attr(), Self::run_python))
            .with_route((Self::datetime_tool_attr(), Self::datetime));

        #[cfg(feature = "searxng")]
        let router = router.with_route((Self::web_search_tool_attr(), Self::web_search));
        #[cfg(feature = "fetch")]
        let router = router.with_route((Self::web_fetch_tool_attr(), Self::web_fetch));

        router
    }

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
        tools::search::query(&self.search_config, web_search.0).await
    }

    #[cfg(feature = "fetch")]
    #[tool(description = "Get the text content from a web URL")]
    pub async fn web_fetch(&self, web_fetch: Parameters<WebFetch>) -> CallToolResult {
        tools::fetch::run(self.fetch_client.clone(), web_fetch.0).await
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        let implementation = Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .with_website_url(env!("CARGO_PKG_REPOSITORY"))
            .with_description(env!("CARGO_PKG_DESCRIPTION"));

        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(implementation)
    }
}

pub async fn run() -> anyhow::Result<()> {
    let server = McpServer::new();
    let ct = CancellationToken::new();

    let config = StreamableHttpServerConfig::default()
        .with_cancellation_token(ct.clone())
        .with_allowed_hosts(get_allowed_hosts());

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

fn get_allowed_hosts() -> Vec<String> {
    let mut hosts = vec!["localhost".into(), "127.0.0.1".into(), "::1".into()];
    hosts.extend(std::env::var("BASE_URL"));
    hosts
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
