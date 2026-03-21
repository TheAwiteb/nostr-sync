use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::DeserializeOwned;
use tokio::fs;

mod base;
mod constant;

use self::base::BaseConfig;
use self::constant::*;
use crate::error::Error;

#[derive(Clone)]
pub struct WebConfig {
    /// Listening address
    pub listen_addr: SocketAddr,
}

#[derive(Clone)]
pub struct Config {
    pub web: WebConfig,
}

impl Config {
    pub async fn read() -> Result<Self, Error> {
        // Get the workdir
        let workdir: PathBuf = get_workdir().await?;

        // Get the path for the base config file
        let config_file_path: PathBuf = workdir.join("config.toml");

        // Read base config
        let mut base_config: BaseConfig = read_toml_config(config_file_path)
            .await?
            .unwrap_or_default();

        // Overwrite config file or default values with ENV variables
        override_from_env(&mut base_config);

        let web: WebConfig = WebConfig {
            listen_addr: base_config
                .web
                .listen_addr
                .unwrap_or(DEFAULT_WEB_LISTEN_ADDR),
        };

        tracing::info!("workdir: {}", workdir.display());
        tracing::info!(addr = ?web.listen_addr, "web:");

        // Construct configs
        Ok(Self { web })
    }
}

async fn get_workdir() -> Result<PathBuf, Error> {
    // Construct the path
    let path: PathBuf = match env::var(ENV_WORKDIR) {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            // Get data dir
            let data_dir: PathBuf = dirs::data_dir().expect("Can't find data directory");

            // Construct path
            // TODO: use rust-nostr/sync path?
            data_dir.join("nostr/sync")
        }
    };

    // If the path doesn't exists, create it
    if !path.exists() {
        fs::create_dir_all(&path).await?;
    }

    // Ensure is a directory
    if !path.is_dir() {
        return Err(Error::NotADirectory);
    }

    Ok(path)
}

async fn read_toml_config<P, T>(path: P) -> Result<Option<T>, Error>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let path: &Path = path.as_ref();

    if !path.exists() {
        tracing::warn!("Config file not found: {}", path.display());
        return Ok(None);
    }

    let content: String = fs::read_to_string(path).await?;
    Ok(Some(toml::from_str(&content)?))
}

fn override_from_env(base_config: &mut BaseConfig) {
    // Override listen addr from env, if any
    if let Ok(addr) = env::var(ENV_WEB_LISTEN_ADDR) {
        base_config.web.listen_addr = Some(SocketAddr::from_str(&addr).unwrap());
    }
}
