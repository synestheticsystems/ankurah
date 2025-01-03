use ankurah_core::{node::Node, storage::SledStorageEngine};
use anyhow::Result;
use std::sync::Arc;
use tracing::Level;

use ankurah_server::ws_server::WebsocketServer;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Initialize storage engine
    let storage = SledStorageEngine::with_homedir_folder(".syncra")?;

    // Create the node
    let node = Arc::new(Node::new(Box::new(storage)));

    // Create and start the websocket server
    let server = WebsocketServer::new(node);
    server.run("0.0.0.0:9797").await?;

    Ok(())
}
