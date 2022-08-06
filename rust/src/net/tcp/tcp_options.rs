use std::net::{Ipv4Addr, SocketAddr};

/// Configuration options which specify in detail how a TCP socket should listen for new connections
/// and process them
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for TcpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}
