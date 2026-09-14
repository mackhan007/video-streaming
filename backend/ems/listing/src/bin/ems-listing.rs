#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = dotenvy::dotenv();
    shared::logging::init();
    ems_listing::run().await
}
