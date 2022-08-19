use bitcoin::{network::message::NetworkMessage, Network, Transaction};

use log::trace;
use rand::seq::SliceRandom;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::{net::lookup_host, sync::Mutex, time::sleep};

use crate::{
    messages::{handle_messages, inv_message},
    network::WriteHandle,
};

pub struct PeerManager {
    pub max_outbound: usize,
    pub peer: Mutex<Peer>,
    pub tx: Transaction,
    pub network: Network,
}

#[derive(Default)]
pub struct Peer {
    pub peers: HashMap<SocketAddr, Arc<SendToPeer>>,
}


pub struct SendToPeer {
    pub write: WriteHandle,
}

impl SendToPeer {
    pub fn new(write: WriteHandle) -> Self {
        Self { write }
    }
}

impl PeerManager {
    pub async fn new(max_outbound: usize, tx: Transaction, network: Network) -> Arc<Self> {
        Arc::new(Self {
            max_outbound,
            peer: Mutex::new(Peer::default()),
            tx,
            network,
        })
    }

    pub async fn broadcast_tx(self: Arc<Self>) -> Option<String> {
        loop {
            if self.peer.lock().await.peers.len() > 8 {
                let mut key = Vec::new();

                let peers = &self.peer.lock().await.peers;

                for keys in peers.keys() {
                    key.push(keys);
                }

                let random_key = key.choose(&mut rand::thread_rng())?;

                let peer = peers.get(random_key)?;

                peer.write
                    .write_message(NetworkMessage::Inv(inv_message(&self.tx).await))
                    .await;

                key.clear();
            }
            sleep(Duration::from_secs(8)).await;
        }
    }

    pub async fn maintain_peers(self: Arc<Self>) {
        let mut addrs = vec![];
        let mut seen = HashSet::new();

        'main: loop {
            let needed = {
                let n_peers = self.peer.lock().await.peers.len();

                self.max_outbound.saturating_sub(n_peers)
            };

            for _ in 0..needed {
                if let Some(addr) = addrs.pop() {
                    tokio::spawn(handle_messages(addr, self.clone()));
                } else {
                    if self.network == Network::Bitcoin {
                        let seeds: Vec<&'static str> = vec![
                            // "seed.bitcoin.sipa.be",
                            // "dnsseed.bluematt.me",
                            // "dnsseed.bitcoin.dashjr.org",
                            // "seed.bitcoinstats.com",
                            "seed.bitcoin.jonasschnelli.ch",
                            // "seed.btc.petertodd.org",
                            // "seed.bitcoin.sprovoost.nl",
                            // "dnsseed.emzy.de",
                            // "seed.bitcoin.wiz.biz",
                        ];

                        for seed in seeds {
                            trace!("fetching addrs from {:?}", seed);

                            if let Ok(seed_addrs) = lookup_host(format!("{}:8333", seed)).await {
                                addrs.extend(
                                    seed_addrs.into_iter().filter(|addr| seen.insert(*addr)),
                                );
                            }
                        }

                        addrs.shuffle(&mut rand::thread_rng());

                        continue 'main;
                    }
                    if self.network == Network::Testnet {
                        let seeds: Vec<&'static str> = vec![
                            "testnet-seed.bitcoin.jonasschnelli.ch",
                            "seed.tbtc.petertodd.org",
                            "seed.testnet.bitcoin.sprovoost.nl",
                            // "testnet-seed.bluematt.me",
                        ];

                        for seed in seeds {
                            trace!("fetching addrs from {:?}", seed);

                            if let Ok(seed_addrs) = lookup_host(format!("{}:18333", seed)).await {
                                addrs.extend(
                                    seed_addrs.into_iter().filter(|addr| seen.insert(*addr)),
                                );
                            }
                        }

                        addrs.shuffle(&mut rand::thread_rng());

                        continue 'main;
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}
