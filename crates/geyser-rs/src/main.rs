use std::{env, net::SocketAddr};

use tokio::net::UdpSocket;
use tracing::{info, warn};

use geyser_rs::raknet::{answer_unconnected_ping, ServerAdvertisement};

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

    let mut advertisement = ServerAdvertisement::default();
    if let Ok(motd) = env::var("GEYSER_MOTD") {
        advertisement.motd = motd;
    }

    let socket = UdpSocket::bind(bind).await?;
    info!(%bind, "Geyser-RS RakNet discovery listener started");

    let mut buffer = vec![0u8; 65_535];
    loop {
        let (length, peer) = socket.recv_from(&mut buffer).await?;
        if let Some(response) = answer_unconnected_ping(&buffer[..length], &advertisement) {
            socket.send_to(&response, peer).await?;
            info!(%peer, "answered Bedrock server-list ping");
        } else {
            // Open-connection requests and connected RakNet frames are not
            // accepted until the reliable transport state machine is ready.
            warn!(%peer, datagram_bytes = length, "unsupported RakNet datagram");
        }
    }
}
