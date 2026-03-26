use std::process::ExitCode;

use tracing::Level;

mod config;
mod error;
mod router;
mod state;
mod syncer;
mod util;

use self::config::Config;
use self::error::Error;
use self::state::SharedState;

async fn try_main() -> Result<(), Error> {
    // Load the config
    let config: Config = Config::read().await?;

    // Construct state
    let state: SharedState = SharedState::new();

    tokio::select! {
        res = syncer::run(&config, state.clone()) => {
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

#[tokio::main]
async fn main() -> ExitCode {
    // Init logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    #[cfg(debug_assertions)]
    tracing::warn!("Running in debug mode!");

    if let Err(err) = try_main().await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
