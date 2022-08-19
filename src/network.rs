use bitcoin::{
    consensus::encode,
    consensus::{deserialize_partial, Encodable},
    network::message::{NetworkMessage, RawNetworkMessage},
    Network,
};

use bytes::{Buf, BytesMut};
use log::{error, trace};

use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};

use crate::peers::PeerManager;

type WriterMessage = RawNetworkMessage;

type MessageReceiver = mpsc::Receiver<NetworkMessage>;
type MessageSender = mpsc::Sender<NetworkMessage>;

#[derive(Clone)]
pub struct WriteHandle {
    pub writer_tx: UnboundedSender<WriterMessage>,
    pub network: Network,
}

impl WriteHandle {
    pub async fn write_message(&self, message: NetworkMessage) {
        let _ = self.writer_tx.send(RawNetworkMessage {
            payload: message,
            magic: self.network.magic(),
        });
    }
}

pub async fn create_connection(
    addr: SocketAddr,
    stream: TcpStream,
    network: Network,
    peer_manager: Arc<PeerManager>,
) -> (WriteHandle, MessageReceiver) {
    let (writer_tx, writer_rx) = mpsc::unbounded_channel();
    let (message_tx, message_rx) = mpsc::channel(1);

    tokio::spawn(run_connection(
        addr,
        stream,
        message_tx,
        writer_rx,
        peer_manager,
    ));

    (WriteHandle { writer_tx, network }, message_rx)
}

pub async fn run_connection(
    addr: SocketAddr,
    mut stream: TcpStream,
    message_tx: MessageSender,
    mut writer_rx: mpsc::UnboundedReceiver<WriterMessage>,
    peer_manager: Arc<PeerManager>,
) {
    let (mut reader, mut writer) = stream.split();

    tokio::select! {
        _handle_read = async {
        let mut buf = BytesMut::new();
        loop {
            match deserialize_partial::<RawNetworkMessage>(&buf[..]) {
                Ok((message, n_bytes)) => {
                    buf.advance(n_bytes);
                    trace!("received {:?} ({})", message.payload, addr);
                    message_tx.send(message.payload).await?;
                }
                Err(error) => {
                    use encode::Error;
                    use std::io::ErrorKind;

                    if matches!(&error, Error::Io(error) if error.kind() == ErrorKind::UnexpectedEof)
                    {
                        let bytes_read = reader.read_buf(&mut buf).await?;
                        if bytes_read == 0 {
                            return anyhow::Ok(())
                        }
                    } else {
                        return Err(error.into());
                    }
                }
            }
        }
    } => {}
    _handle_write  = async {
        let mut bytes = Vec::new();
        loop {
            if let Some(message) = writer_rx.recv().await {
                // encode message
                message
                    .consensus_encode(&mut bytes)?;

                // write message
                writer.write_all(&bytes).await?;

                writer.flush().await?;

                bytes.clear();

                trace!("sent {} ({})", message.cmd(), addr);
            } else {
                return anyhow::Ok(())
            }
        }
    } => {}
    };

    peer_manager.peer.lock().await.peers.remove(&addr);
    error!("connection disconnected {}", addr);
}
