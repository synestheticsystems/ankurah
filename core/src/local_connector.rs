use ankurah_proto as proto;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::node::{Node, PeerConnection};

pub struct LocalPeerConnection {
    sender: mpsc::Sender<proto::PeerMessage>,
}

impl PeerConnection for LocalPeerConnection {
    async fn send_message(&self, message: proto::PeerMessage) -> anyhow::Result<()> {
        self.sender
            .send(message)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send message"))?;
        Ok(())
    }
}

/// Manages a local in-memory connection between two nodes
pub struct LocalConnector {
    node1: Arc<Node>,
    node2: Arc<Node>,
}

impl LocalConnector {
    /// Create a new LocalConnector and establish connection between the nodes
    pub fn new(node1: Arc<Node>, node2: Arc<Node>) -> anyhow::Result<Self> {
        let (node1_tx, node1_rx) = mpsc::channel(100);
        let (node2_tx, node2_rx) = mpsc::channel(100);

        Self::setup_receiver(&node1, node1_rx);
        Self::setup_receiver(&node2, node2_rx);

        Ok(Self { node1, node2 })
    }

    fn setup_receiver(node: &Arc<Node>, rx: mpsc::Receiver<proto::PeerMessage>) {
        let me = node.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                println!("Received message: {:?}", message);
                match message {
                    proto::PeerMessage::Request(request) => {
                        // double check to make sure we have a connection to the peer based on the node id
                        let conn = {
                            let peer_connections = me.peer_connections.read().unwrap();
                            peer_connections.get(&request.from).cloned()
                        };

                        if let Some(tx) = conn {
                            let me = me.clone();
                            tokio::spawn(async move {
                                let from = request.from.clone();
                                let to = request.to.clone();
                                let id = request.id.clone();
                                let result = me.handle_request(request).await;
                                tx.send_message(PeerMessage::Response(NodeResponse {
                                    request_id: id,
                                    from: from,
                                    to: to,
                                    result,
                                }))
                                .await
                                .unwrap();
                            });
                        }
                    }
                    PeerMessage::Response(response) => {
                        if let Some(tx) = me
                            .pending_requests
                            .write()
                            .unwrap()
                            .remove(&response.request_id)
                        {
                            tx.send(response.result).unwrap();
                        }
                    }
                }
            }
        });
    }
}

impl Drop for LocalConnector {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

// pub fn local_connect(self: &Arc<Self>, peer: &Arc<Node>) {
//     let (my_tx, my_rx) = mpsc::channel(100);
//     let (peer_tx, peer_rx) = mpsc::channel(100); // Buffer size of 100

//     self.setup_local_handler(my_rx);
//     peer.setup_local_handler(peer_rx);
//     // Store the peer connection for the client
//     self.peer_connections
//         .write()
//         .unwrap()
//         .insert(peer.id.clone(), PeerConnection::Local(peer_tx));
//     // Store the peer connection for the server
//     peer.peer_connections
//         .write()
//         .unwrap()
//         .insert(self.id.clone(), PeerConnection::Local(my_tx));
// }
