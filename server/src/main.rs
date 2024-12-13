use ankurah_core::storage::SledStorageEngine;
use anyhow::Result;
use std::path::PathBuf;
use tracing::Level;

mod server;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Initialize storage engine
    let storage_path = std::env::var("STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".syncra"));

    let storage = SledStorageEngine::with_path(storage_path)?;

    // Create and start the server
    let server = server::Server::builder()
        .bind_address("0.0.0.0:8080") // Cloud Run requires port 8080
        .with_storage(storage)
        .build()?;

    server.run().await?;

    Ok(())
}
