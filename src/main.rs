use tracing::Level;

mod config;
mod error;
mod router;
mod state;
mod syncer;

use self::config::Config;
use self::error::Error;
use self::state::SharedState;

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

    tokio::select! {
        res = syncer::run(&config) => {
            // Propagate error, if any
            let _: () = res?;
        }
        // Serve Web UI
        res = router::serve_web_ui(&config, state) => {
            // Propagate error, if any
            let _: () = res?;
        }
    }

    Ok(())
}
