use std::net::{Ipv4Addr, SocketAddr};

/// Configuration options which specify in detail how Websocket server should listen for new connections
/// and process them
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WsOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for WsOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}