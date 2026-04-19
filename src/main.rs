mod python;
mod server;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    server::run().await
}
