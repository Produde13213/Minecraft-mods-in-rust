use std::{env, net::SocketAddr};

use tokio::net::UdpSocket;
use tracing::{info, warn};

use geyser_rs::raknet::{
    answer_open_connection_request_1, answer_open_connection_request_2,
    answer_unconnected_ping, ServerAdvertisement, OPEN_CONNECTION_REQUEST_1,
    OPEN_CONNECTION_REQUEST_2,
};

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
    info!(%bind, "Geyser-RS RakNet listener started");

    let mut buffer = vec![0u8; 65_535];
    loop {
        let (length, peer) = socket.recv_from(&mut buffer).await?;
        let packet = &buffer[..length];
        let response = match packet.first().copied() {
            Some(0x01) => answer_unconnected_ping(packet, &advertisement),
            Some(OPEN_CONNECTION_REQUEST_1) => {
                answer_open_connection_request_1(packet, advertisement.server_id)
            }
            Some(OPEN_CONNECTION_REQUEST_2) => {
                answer_open_connection_request_2(packet, peer, advertisement.server_id)
            }
            _ => None,
        };

        if let Some(response) = response {
            socket.send_to(&response, peer).await?;
            info!(%peer, packet_id = packet[0], "answered RakNet handshake packet");
        } else {
            warn!(%peer, datagram_bytes = length, "unsupported or malformed RakNet datagram");
        }
    }
}
