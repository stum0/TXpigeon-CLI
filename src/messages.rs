use bitcoin::{
    network::message::NetworkMessage,
    network::{
        constants::{self, ServiceFlags},
        message::{self},
        message_blockdata::Inventory,
        message_network::VersionMessage,
        Address,
    },
    Transaction,
};

use log::{error, info};

use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, time::timeout};

use std::{
    net::{IpAddr, Ipv4Addr},
    process,
    time::Duration,
};

use crate::{
    network::create_connection,
    peers::{PeerManager, SendToPeer},
};

pub async fn handle_messages(addr: SocketAddr, peer_manager: Arc<PeerManager>) {
    let stream = timeout(Duration::from_secs(3), TcpStream::connect(addr)).await;

    if let Ok(Ok(stream)) = stream {
        let _ = stream.set_nodelay(true);

        let (write_handle, mut message_rx) =
            create_connection(addr, stream, peer_manager.network, peer_manager.clone()).await;

        write_handle
            .write_message(NetworkMessage::Version(version_msg(addr).await))
            .await;

        let new_peer = SendToPeer::new(write_handle.clone());
        let new_peer = Arc::new(new_peer);

        peer_manager
            .peer
            .lock()
            .await
            .peers
            .insert(addr, new_peer);

        while let Some(msg) = message_rx.recv().await {
            match msg {
                message::NetworkMessage::Version(_) => {
                    write_handle.write_message(NetworkMessage::Verack).await;
                    info!("connected to {:?}", addr);
                }
                message::NetworkMessage::Inv(inv) => {
                    if inv.contains(&Inventory::Transaction(peer_manager.tx.txid())) {
                        info!("Transaction has reached the mempool {:?}", addr);

                        process::exit(1);
                    }
                }
                message::NetworkMessage::GetData(_) => {
                    if msg
                        == message::NetworkMessage::GetData(
                            inv_message(&peer_manager.tx).await,
                        )
                    {
                        write_handle
                            .write_message(NetworkMessage::Tx(peer_manager.tx.clone())).await;
                    }
                    info!("{:?} Broadcast to {:?}", peer_manager.tx, addr);
                    break;
                }

                _ => {}
            }
        }
    } else {
        error!("connection failed to {:?}", addr);
    }
}

async fn version_msg(addr: SocketAddr) -> VersionMessage {
    let my_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

    VersionMessage {
        version: 70016,
        services: ServiceFlags::NONE,
        timestamp: 0,
        receiver: Address::new(&addr, constants::ServiceFlags::NONE),
        sender: Address::new(&my_address, constants::ServiceFlags::NONE),
        nonce: 0,
        user_agent: "/Satoshi:23.0.0/".to_string(),
        start_height: 0,
        relay: true,
    }
}

pub async fn inv_message(tx: &Transaction) -> Vec<Inventory> {
    let txid = Inventory::Transaction(tx.txid());
    vec![txid]
}
