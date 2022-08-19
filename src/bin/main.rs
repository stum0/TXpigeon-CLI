use std::process;

use bitcoin::{consensus::deserialize, hashes::hex::FromHex, Network, Transaction};
use env_logger::{Builder, WriteStyle};
use log::{error, info, LevelFilter};

use clap::Parser;

use anyhow::Result;
use txp::peers::PeerManager;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.logging {
        let mut builder = Builder::new();

        builder
            .filter(None, LevelFilter::Trace)
            .write_style(WriteStyle::Always)
            .init();
    } else {
        let mut builder = Builder::new();
        builder
            .filter(None, LevelFilter::Info)
            .write_style(WriteStyle::Always)
            .init();
    }

    let network = cli.network;
    info!("NETWORK: {:?}", network);

    let txhex: Transaction = deserialize(&Vec::from_hex(&cli.tx)?).unwrap_or_else(|_error| {
        error!("transaction format incorrect, must be a raw bitcoin transaction");
        process::exit(1);
    });

    let peer_manager = PeerManager::new(10, txhex, network).await;

     let t2 = tokio::spawn(peer_manager.clone().broadcast_tx());
    let t1 = tokio::spawn(peer_manager.maintain_peers());
   

    t1.await?;
    t2.await?;

    Ok(())
}

#[derive(Parser)]
#[clap(name = "TxPigeon")]
#[clap(version = "0.3.0")]
#[clap(about = "send raw bitcoin transaction to the network", long_about = None)]

struct Cli {
    /// raw bitcoin transaction
    tx: String,
    /// choose bitcoin network (Bitcoin(default) = MainNet, Testnet = TestNet)
    #[clap(long, default_value_t = Network::Bitcoin, value_parser)]
    network: Network,
    /// enable logging
    #[clap(short, action, default_value_t = false)]
    logging: bool,
}
