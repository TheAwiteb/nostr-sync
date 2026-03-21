use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tracing::Level;

mod config;
mod error;
mod router;
mod state;

use self::config::Config;
use self::error::Error;
use self::state::SharedState;

async fn serve_web_ui(config: Config, state: SharedState) -> Result<(), Error> {
    let listen_add: SocketAddr = config.web.listen_addr;

    // Build router
    let router: Router = router::build(state);

    tracing::debug!("Starting Web UI server...");

    // Bind listener
    let listener: TcpListener = TcpListener::bind(listen_add).await?;

    tracing::info!("Serving Web UI on http://{listen_add}/");

    // Serve web UI
    axum::serve(listener, router).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Init logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    #[cfg(debug_assertions)]
    tracing::warn!("Running in debug mode!");

    // Load the config
    let config: Config = Config::read().await?;

    // Construct state
    let state: SharedState = SharedState::new();

    // Serve the Web UI
    serve_web_ui(config, state).await?;

    Ok(())
}
