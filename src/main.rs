use docker_mcp::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}
