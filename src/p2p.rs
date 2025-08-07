use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::error::Error;
use crate::blockchain::{Block, Transaction};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    NewBlock(Block),
    NewTransaction(Transaction),
    GetBlocks(SocketAddr),
    Blocks(Vec<Block>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pMessage {
    pub sender: SocketAddr,
    pub message: Message,
}

pub struct Peer {
    addr: SocketAddr,
    stream: TcpStream,
}

impl Peer {
    pub async fn new(addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Peer { addr, stream })
    }
}

pub struct P2p {
    peers: HashMap<SocketAddr, Peer>,
    listener: TcpListener,
    peer_addrs: Vec<SocketAddr>,
}

impl P2p {
    pub async fn new(port: u16, peer_addrs: Vec<SocketAddr>) -> Result<Self, Box<dyn Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await?;
        Ok(P2p {
            peers: HashMap::new(),
            listener,
            peer_addrs,
        })
    }

    pub async fn run(&mut self, tx: mpsc::Sender<P2pMessage>) {
        info!("P2P network running.");
        loop {
            let (mut stream, addr) = self.listener.accept().await.unwrap();
            info!("New connection from {}", addr);
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0; 1024];
                loop {
                    match stream.read(&mut buffer).await {
                        Ok(0) => {
                            info!("Connection with {} closed.", addr);
                            break;
                        }
                        Ok(n) => {
                            if let Ok(message) = serde_json::from_slice::<P2pMessage>(&buffer[..n]) {
                                info!("Received message: {:?}", message);
                                tx.send(message).await.unwrap();
                            }
                        }
                        Err(e) => {
                            warn!("Failed to read from stream: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }

    pub async fn broadcast_message(&mut self, message: P2pMessage) -> Result<(), Box<dyn Error>> {
        info!("Broadcasting message: {:?}", message);
        let message_bytes = serde_json::to_vec(&message)?;
        for peer in self.peers.values_mut() {
            peer.stream.write_all(&message_bytes).await?;
        }
        Ok(())
    }
}
