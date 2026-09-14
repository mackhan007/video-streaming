#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = dotenvy::dotenv();
    shared::logging::init();
    ims_processor::run().await
}
