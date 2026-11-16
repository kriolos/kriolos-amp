use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

pub struct PortChecker;

impl PortChecker {
    /// Checks if a port is in use by attempting to bind to it
    pub fn is_port_in_use(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        // If binding fails, port is already in use
        if TcpListener::bind(addr).is_err() {
            return true;
        }

        // Also check if we can connect to it (in case SO_REUSEADDR or similar is involved)
        if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
            return true;
        }

        false
    }

    /// Wait until a port becomes responsive or timeout expires
    pub async fn wait_for_port(port: u16, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        while start.elapsed() < timeout {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        false
    }
}
