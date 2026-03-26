use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use nostr_sdk::prelude::*;
use serde::de::DeserializeOwned;
use tokio::fs;

mod base;
mod constant;

use self::base::BaseConfig;
use self::constant::*;
use crate::error::Error;

#[derive(Debug)]
pub struct WebConfig {
    /// Listening address
    pub listen_addr: SocketAddr,
}

#[derive(Debug)]
pub struct NostrConfig {
    /// Events database path
    pub events_path: PathBuf,
    /// Gossip database path
    pub gossip_path: PathBuf,
    // TODO: move these to the database, so will be configurable from the Web UI?
    /// Discovery relays
    pub discovery_relays: HashSet<RelayUrl>,
}

#[derive(Debug)]
pub struct Config {
    pub web: WebConfig,
    pub nostr: NostrConfig,
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
        override_from_env(&mut base_config)?;

        let web: WebConfig = WebConfig {
            listen_addr: base_config
                .web
                .listen_addr
                .unwrap_or(DEFAULT_WEB_LISTEN_ADDR),
        };

        let nostr: NostrConfig = NostrConfig {
            events_path: workdir.join("events.db"),
            gossip_path: workdir.join("gossip.db"),
            discovery_relays: base_config
                .nostr
                .discovery_relays
                .unwrap_or_else(|| DEFAULT_NOSTR_DISCOVERY_RELAYS.clone()),
        };

        tracing::info!("workdir: {}", workdir.display());
        tracing::info!(addr = ?web.listen_addr, "web:");
        tracing::info!(events_db = ?nostr.events_path, gossip_db = ?nostr.gossip_path, "nostr:");

        // Construct configs
        Ok(Self { web, nostr })
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
            data_dir.join("rust-nostr/sync")
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

fn override_from_env(base_config: &mut BaseConfig) -> Result<(), Error> {
    // Override listen addr from env, if any
    if let Ok(addr) = env::var(ENV_WEB_LISTEN_ADDR) {
        base_config.web.listen_addr = Some(SocketAddr::from_str(&addr).unwrap());
    }

    // Override discovery relays, if any
    if let Ok(discovery_relays) = env::var(ENV_NOSTR_DISCOVERY_RELAYS) {
        let relays: HashSet<RelayUrl> = dbg!(discovery_relays)
            .split(',')
            .map(str::trim)
            .filter_map(|relay_url| {
                if !relay_url.is_empty() {
                    Some(
                        RelayUrl::from_str(relay_url)
                            .map_err(|err| Error::InvalidRelayUrl(relay_url.to_owned(), err)),
                    )
                } else {
                    None
                }
            })
            .collect::<Result<_, _>>()?;

        if !relays.is_empty() {
            base_config.nostr.discovery_relays = Some(relays);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::net::SocketAddr;
    use std::sync::LazyLock;

    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use super::*;

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    // Helper to create a temp directory and set it as workdir
    fn setup_temp_env() -> tempfile::TempDir {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        unsafe {
            env::set_var(ENV_WORKDIR, temp_dir.path());
        }
        temp_dir
    }

    // Helper to clean up env vars
    fn cleanup_env() {
        unsafe {
            env::remove_var(ENV_WORKDIR);
            env::remove_var(ENV_WEB_LISTEN_ADDR);
            env::remove_var(ENV_NOSTR_DISCOVERY_RELAYS);
        }
    }

    // Helper function to set env var
    fn set_env(key: &str, val: &str) {
        unsafe {
            env::set_var(key, val);
        }
    }

    #[tokio::test]
    async fn test_config_with_env_relays_valid() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set valid relay URLs via ENV
        set_env(
            ENV_NOSTR_DISCOVERY_RELAYS,
            "wss://relay.example1,wss://example2.wine,wss://relay.example3.social",
        );

        let config = Config::read().await.expect("Failed to read config");

        assert_eq!(config.nostr.discovery_relays.len(), 3);
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("example1"))
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("example2.wine"))
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("example3.social"))
        );
    }

    #[tokio::test]
    async fn test_config_with_env_relays_single() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set single relay URL
        set_env(ENV_NOSTR_DISCOVERY_RELAYS, "wss://relay.example1");

        let config = Config::read().await.expect("Failed to read config");

        assert_eq!(config.nostr.discovery_relays.len(), 1);
        assert!(
            config
                .nostr
                .discovery_relays
                .contains(&RelayUrl::from_str("wss://relay.example1").unwrap()),
        );
    }

    #[tokio::test]
    async fn test_config_with_env_relays_empty_string() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set empty relay list
        set_env(ENV_NOSTR_DISCOVERY_RELAYS, "");

        let config = Config::read().await.expect("Failed to read config");

        // Should use defaults when empty string
        assert!(!config.nostr.discovery_relays.is_empty());
        assert_eq!(
            config.nostr.discovery_relays,
            DEFAULT_NOSTR_DISCOVERY_RELAYS.clone()
        );
    }

    #[tokio::test]
    async fn test_config_with_env_relays_invalid_url() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set invalid relay URL
        set_env(ENV_NOSTR_DISCOVERY_RELAYS, "invalid-url, wss://valid.relay");

        let result = Config::read().await;
        assert!(result.is_err());

        // Check error type
        match result.unwrap_err() {
            Error::InvalidRelayUrl(url, _) => {
                assert_eq!(url, "invalid-url");
            }
            _ => panic!("Expected InvalidRelayUrl error"),
        }
    }

    #[tokio::test]
    async fn test_config_with_env_relays_malformed() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set malformed relay URL (missing scheme)
        set_env(ENV_NOSTR_DISCOVERY_RELAYS, "relay.example1");

        let result = Config::read().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::InvalidRelayUrl(url, _) => {
                assert_eq!(url, "relay.example1");
            }
            _ => panic!("Expected InvalidRelayUrl error"),
        }
    }

    #[tokio::test]
    async fn test_config_with_env_relays_trailing_comma() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Test with trailing comma
        set_env(ENV_NOSTR_DISCOVERY_RELAYS, "wss://relay.example1,");

        let config = Config::read().await.expect("Failed to read config");

        // Should parse single relay ignoring trailing comma
        assert_eq!(config.nostr.discovery_relays.len(), 1);
        assert!(
            config
                .nostr
                .discovery_relays
                .contains(&RelayUrl::from_str("wss://relay.example1").unwrap())
        );
    }

    #[tokio::test]
    async fn test_config_with_env_relays_extra_spaces() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Test with extra spaces around commas
        set_env(
            ENV_NOSTR_DISCOVERY_RELAYS,
            "  wss://relay.example1  ,  wss://example2.wine  ,  wss://relay.example3.social  ",
        );

        let config = Config::read().await.expect("Failed to read config");

        assert_eq!(config.nostr.discovery_relays.len(), 3);
        // URLs should be trimmed and normalized
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str() == "wss://relay.example1")
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str() == "wss://example2.wine")
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str() == "wss://relay.example3.social")
        );
    }

    #[tokio::test]
    async fn test_config_with_env_web_addr() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set web listen address via ENV
        set_env(ENV_WEB_LISTEN_ADDR, "127.0.0.1:8080");

        let config = Config::read().await.expect("Failed to read config");

        assert_eq!(
            config.web.listen_addr,
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap()
        );
    }

    #[tokio::test]
    async fn test_config_with_env_web_addr_and_relays() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Set both ENV variables
        set_env(ENV_WEB_LISTEN_ADDR, "0.0.0.0:3000");
        set_env(
            ENV_NOSTR_DISCOVERY_RELAYS,
            "wss://relay.example1,wss://example2.wine",
        );

        let config = Config::read().await.expect("Failed to read config");

        assert_eq!(
            config.web.listen_addr,
            "0.0.0.0:3000".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.nostr.discovery_relays.len(), 2);
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("example1"))
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("example2.wine"))
        );
    }

    #[tokio::test]
    async fn test_config_without_env_uses_defaults() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let _temp_dir = setup_temp_env();

        // Don't set any ENV variables
        let config = Config::read().await.expect("Failed to read config");

        // Should use default web listen addr
        assert_eq!(config.web.listen_addr, DEFAULT_WEB_LISTEN_ADDR);

        // Should use default discovery relays
        assert_eq!(
            config.nostr.discovery_relays,
            DEFAULT_NOSTR_DISCOVERY_RELAYS.clone()
        );
    }

    #[tokio::test]
    async fn test_config_env_relays_overrides_file() {
        let _guard = ENV_MUTEX.lock().await;

        cleanup_env();
        let temp_dir = setup_temp_env();

        // Create a config file with some relays
        let config_content = r#"
            [web]
            listen_addr = "127.0.0.1:8080"

            [nostr]
            discovery_relays = [
                "wss://old.relay1",
                "wss://old.relay2"
            ]
        "#;

        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write config file");

        // Set ENV variable to override
        set_env(
            ENV_NOSTR_DISCOVERY_RELAYS,
            "wss://new.relay1,wss://new.relay2",
        );

        let config = Config::read().await.expect("Failed to read config");

        // Should use ENV values, not file values
        assert_eq!(config.nostr.discovery_relays.len(), 2);
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("new.relay1"))
        );
        assert!(
            config
                .nostr
                .discovery_relays
                .iter()
                .any(|r| r.as_str().contains("new.relay2"))
        );
    }
}
