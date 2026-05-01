mod server;
mod tools;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}
