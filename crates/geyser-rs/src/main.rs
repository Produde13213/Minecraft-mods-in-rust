use std::{env, net::SocketAddr};

use tokio::net::UdpSocket;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "geyser_rs=info".to_owned()),
        )
        .init();

    let bind: SocketAddr = env::var("GEYSER_BEDROCK_BIND")
        .unwrap_or_else(|_| "0.0.0.0:19132".to_owned())
        .parse()?;

    let socket = UdpSocket::bind(bind).await?;
    info!(%bind, "Geyser-RS transport bootstrap listening");

    let mut buffer = vec![0u8; 65_535];
    loop {
        let (length, peer) = socket.recv_from(&mut buffer).await?;
        // This is deliberately diagnostic only: a real Bedrock endpoint must
        // complete the RakNet unconnected ping/open-connection handshake
        // before it can accept a Bedrock login. Do not send fake responses.
        warn!(
            %peer,
            datagram_bytes = length,
            "received UDP datagram; RakNet handshake is not implemented yet"
        );
    }
}
