#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    shared::logging::init();
    ems_server::run().await
}
