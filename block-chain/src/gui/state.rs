use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
struct SharedNodeStats {
    peer_count: usize,
    peer_addresses: Vec<SocketAddr>,
}

#[derive(Debug, Clone, Default)]
pub struct NodeState {
    inner: Arc<Mutex<SharedNodeStats>>,
}

impl NodeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(&self, addr: SocketAddr) {
        if let Ok(mut stats) = self.inner.lock() {
            stats.peer_count += 1;
            stats.peer_addresses.push(addr);
        }
    }

    pub fn get_summary(&self) -> (usize, Vec<SocketAddr>) {
        self.inner
            .lock()
            .map(|s| (s.peer_count, s.peer_addresses.clone()))
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub enum NavigationPage {
    Dashboard,
    NetworkExplorer,
    PeerDetails(SocketAddr),
}
