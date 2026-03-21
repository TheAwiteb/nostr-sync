use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub(super) const DEFAULT_WEB_LISTEN_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 44552));

pub(super) const ENV_WORKDIR: &str = "NOSTR_SYNC_WORKDIR";

pub(super) const ENV_WEB_LISTEN_ADDR: &str = "NOSTR_SYNC_WEB_LISTEN_ADDR";
