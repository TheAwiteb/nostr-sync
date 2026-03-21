use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct BaseWebConfig {
    pub listen_addr: Option<SocketAddr>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BaseConfig {
    pub web: BaseWebConfig,
}
