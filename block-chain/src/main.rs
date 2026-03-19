use block_chain::{Block, BlockChain};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

mod gui;
mod utils;

struct NodeService {
    blockchain: Arc<Mutex<BlockChain>>,
    node_state: gui::NodeState,
}

impl NodeService {
    pub fn new(node_state: gui::NodeState) -> Self {
        Self {
            blockchain: Arc::new(Mutex::new(BlockChain::new())),
            node_state,
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.blockchain.lock().await.init()?;

        let listener = TcpListener::bind(addr).await?;
        let (tx, mut rx) = mpsc::channel::<Block>(32);

        let bc = self.blockchain.clone();
        tokio::spawn(async move {
            while let Some(block) = rx.recv().await {
                let _ = bc.lock().await.add_block(block);
            }
        });

        let peers = Arc::new(Mutex::new(HashMap::new()));
        loop {
            let (socket, peer_addr) = listener.accept().await?;
            self.handle_connection(socket, peer_addr, tx.clone(), peers.clone()).await;
        }
    }

    async fn handle_connection(
        &self,
        socket: TcpStream,
        addr: SocketAddr,
        tx: mpsc::Sender<Block>,
        peers: Arc<Mutex<HashMap<SocketAddr, tokio::net::tcp::OwnedWriteHalf>>>,
    ) {
        let (reader, writer) = socket.into_split();
        peers.lock().await.insert(addr, writer);
        self.node_state.add_peer(addr);

        tokio::spawn(async move {
            let mut reader = reader;
            let mut buffer = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buffer).await {
                if n == 0 { break; }
                if let Ok(block) = serde_json::from_slice::<Block>(&buffer[..n]) {
                    let _ = tx.send(block).await;
                }
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node_state = gui::NodeState::new();
    let service = NodeService::new(node_state.clone());

    let gui_state = node_state.clone();
    std::thread::spawn(move || {
        if let Err(e) = gui::run_gui(gui_state) {
            eprintln!("GUI Error: {}", e);
        }
    });

    service.run("127.0.0.1:10100").await
}

