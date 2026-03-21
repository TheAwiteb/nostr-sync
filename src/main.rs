use tracing::Level;

mod config;
mod error;

use self::config::Config;
use self::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Init logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    #[cfg(debug_assertions)]
    tracing::warn!("Running in debug mode!");

    // Load the config
    let config: Config = Config::read().await?;

    // TODO: start the web UI and the services

    Ok(())
}
