//! RakNet offline discovery and the first two connection-negotiation steps.
//!
//! Reliable connected sessions, fragmentation, ordering and retransmission are
//! deliberately separate follow-up work.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub const UNCONNECTED_PING: u8 = 0x01;
pub const UNCONNECTED_PONG: u8 = 0x1c;
pub const OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
pub const OPEN_CONNECTION_REPLY_1: u8 = 0x06;
pub const OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
pub const OPEN_CONNECTION_REPLY_2: u8 = 0x08;
pub const RAKNET_PROTOCOL_VERSION: u8 = 11;
pub const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

const MIN_MTU: usize = 576;
const MAX_MTU: usize = 1492;

#[derive(Debug, Clone)]
pub struct ServerAdvertisement {
    pub motd: String,
    pub version: String,
    pub protocol: u32,
    pub online_players: u32,
    pub max_players: u32,
    pub server_id: u64,
    pub port: u16,
}

impl Default for ServerAdvertisement {
    fn default() -> Self {
        Self {
            motd: "Geyser-RS".to_owned(),
            version: "26.65".to_owned(),
            protocol: 2225,
            online_players: 0,
            max_players: 100,
            server_id: 0x4759_5253_0000_0001,
            port: 19132,
        }
    }
}

/// Answers a valid RakNet unconnected ping with an MCPE server-list pong.
pub fn answer_unconnected_ping(
    packet: &[u8],
    advertisement: &ServerAdvertisement,
) -> Option<Vec<u8>> {
    if packet.len() < 33 || packet[0] != UNCONNECTED_PING {
        return None;
    }
    if packet[9..25] != RAKNET_MAGIC {
        return None;
    }

    let mut response = Vec::with_capacity(195);
    response.push(UNCONNECTED_PONG);
    response.extend_from_slice(&packet[1..9]);
    response.extend_from_slice(&advertisement.server_id.to_be_bytes());
    response.extend_from_slice(&RAKNET_MAGIC);

    let motd = advertisement.motd.replace(';', ":");
    let players = advertisement.online_players;
    let max_players = advertisement.max_players.max(players.saturating_add(1));
    let text = format!(
        "MCPE;{};{};{};{};{};{};{};Survival;1;{};{}",
        motd,
        advertisement.protocol,
        advertisement.version,
        players,
        max_players,
        advertisement.server_id,
        motd,
        advertisement.port,
        advertisement.port,
    );
    let bytes = text.as_bytes();
    let length = u16::try_from(bytes.len()).ok()?;
    response.extend_from_slice(&length.to_be_bytes());
    response.extend_from_slice(bytes);
    Some(response)
}

/// Parses an IPv4 RakNet address (version, complemented octets, big-endian port).
fn read_ipv4_address(packet: &[u8], offset: usize) -> Option<(SocketAddrV4, usize)> {
    let address = packet.get(offset..offset + 7)?;
    if address[0] != 4 {
        return None;
    }
    let ip = Ipv4Addr::new(
        !address[1],
        !address[2],
        !address[3],
        !address[4],
    );
    let port = u16::from_be_bytes([address[5], address[6]]);
    Some((SocketAddrV4::new(ip, port), offset + 7))
}

fn write_ipv4_address(out: &mut Vec<u8>, address: SocketAddrV4) {
    out.push(4);
    for octet in address.ip().octets() {
        out.push(!octet);
    }
    out.extend_from_slice(&address.port().to_be_bytes());
}

/// Handles Open Connection Request 1. The datagram size is the client's
/// requested MTU; the response advertises the bounded MTU selected by us.
pub fn answer_open_connection_request_1(
    packet: &[u8],
    server_id: u64,
) -> Option<Vec<u8>> {
    if packet.len() < 18
        || packet[0] != OPEN_CONNECTION_REQUEST_1
        || packet[1..17] != RAKNET_MAGIC
        || packet[17] != RAKNET_PROTOCOL_VERSION
    {
        return None;
    }

    let mtu = packet.len().clamp(MIN_MTU, MAX_MTU) as u16;
    let mut response = Vec::with_capacity(28);
    response.push(OPEN_CONNECTION_REPLY_1);
    response.extend_from_slice(&RAKNET_MAGIC);
    response.extend_from_slice(&server_id.to_be_bytes());
    response.push(0); // Security is not enabled.
    response.extend_from_slice(&mtu.to_be_bytes());
    Some(response)
}

/// Handles Open Connection Request 2 for IPv4 peers. IPv6 peers are rejected
/// until the IPv6 RakNet address codec is implemented.
pub fn answer_open_connection_request_2(
    packet: &[u8],
    peer: SocketAddr,
    server_id: u64,
) -> Option<Vec<u8>> {
    if packet.len() < 34
        || packet[0] != OPEN_CONNECTION_REQUEST_2
        || packet[1..17] != RAKNET_MAGIC
    {
        return None;
    }
    let (requested_server, offset) = read_ipv4_address(packet, 17)?;
    let mtu_bytes = packet.get(offset..offset + 2)?;
    let requested_mtu = u16::from_be_bytes([mtu_bytes[0], mtu_bytes[1]]) as usize;
    let guid_offset = offset + 2;
    let client_guid = packet.get(guid_offset..guid_offset + 8)?;
    let client_guid = u64::from_be_bytes(client_guid.try_into().ok()?);

    let peer = match peer {
        SocketAddr::V4(address) => address,
        SocketAddr::V6(_) => return None,
    };
    let mtu = requested_mtu.clamp(MIN_MTU, MAX_MTU) as u16;

    let mut response = Vec::with_capacity(35);
    response.push(OPEN_CONNECTION_REPLY_2);
    response.extend_from_slice(&RAKNET_MAGIC);
    response.extend_from_slice(&server_id.to_be_bytes());
    write_ipv4_address(&mut response, *peer);
    response.extend_from_slice(&mtu.to_be_bytes());
    response.push(0); // Security is not enabled.
    let _ = (requested_server, client_guid); // Parsed for validation/diagnostics.
    Some(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_valid_unconnected_ping() {
        let mut ping = vec![UNCONNECTED_PING];
        ping.extend_from_slice(&1234u64.to_be_bytes());
        ping.extend_from_slice(&RAKNET_MAGIC);
        ping.extend_from_slice(&99u64.to_be_bytes());

        let pong = answer_unconnected_ping(&ping, &ServerAdvertisement::default()).unwrap();
        assert_eq!(pong[0], UNCONNECTED_PONG);
        assert_eq!(&pong[1..9], &1234u64.to_be_bytes());
        assert_eq!(&pong[17..33], &RAKNET_MAGIC);

        let string_len = u16::from_be_bytes([pong[33], pong[34]]) as usize;
        assert_eq!(pong.len(), 35 + string_len);
        let advertisement = std::str::from_utf8(&pong[35..]).unwrap();
        assert!(advertisement.starts_with("MCPE;Geyser-RS;2225;26.65;"));
    }

    #[test]
    fn rejects_wrong_magic_and_non_ping_packets() {
        let mut ping = vec![UNCONNECTED_PING];
        ping.extend_from_slice(&0u64.to_be_bytes());
        ping.extend_from_slice(&[0u8; 16]);
        ping.extend_from_slice(&0u64.to_be_bytes());
        assert!(answer_unconnected_ping(&ping, &ServerAdvertisement::default()).is_none());
        assert!(answer_unconnected_ping(&[0x02; 40], &ServerAdvertisement::default()).is_none());
    }

    #[test]
    fn rejects_short_ping() {
        assert!(answer_unconnected_ping(&[UNCONNECTED_PING; 10], &ServerAdvertisement::default()).is_none());
    }

    #[test]
    fn answers_open_connection_request_1() {
        let mut request = vec![OPEN_CONNECTION_REQUEST_1];
        request.extend_from_slice(&RAKNET_MAGIC);
        request.push(RAKNET_PROTOCOL_VERSION);
        request.resize(1200, 0);

        let reply = answer_open_connection_request_1(&request, 42).unwrap();
        assert_eq!(reply[0], OPEN_CONNECTION_REPLY_1);
        assert_eq!(&reply[1..17], &RAKNET_MAGIC);
        assert_eq!(&reply[17..25], &42u64.to_be_bytes());
        assert_eq!(reply[25], 0);
        assert_eq!(u16::from_be_bytes([reply[26], reply[27]]), 1200);
    }

    #[test]
    fn rejects_bad_open_connection_request_1() {
        let mut request = vec![OPEN_CONNECTION_REQUEST_1];
        request.extend_from_slice(&[0u8; 16]);
        request.push(RAKNET_PROTOCOL_VERSION);
        request.resize(1200, 0);
        assert!(answer_open_connection_request_1(&request, 1).is_none());
    }

    #[test]
    fn answers_open_connection_request_2_for_ipv4() {
        let peer: SocketAddr = "192.0.2.10:50000".parse().unwrap();
        let mut request = vec![OPEN_CONNECTION_REQUEST_2];
        request.extend_from_slice(&RAKNET_MAGIC);
        write_ipv4_address(&mut request, "127.0.0.1:19132".parse().unwrap());
        request.extend_from_slice(&1200u16.to_be_bytes());
        request.extend_from_slice(&77u64.to_be_bytes());

        let reply = answer_open_connection_request_2(&request, peer, 42).unwrap();
        assert_eq!(reply[0], OPEN_CONNECTION_REPLY_2);
        assert_eq!(&reply[1..17], &RAKNET_MAGIC);
        assert_eq!(&reply[17..25], &42u64.to_be_bytes());
        let (client, next) = read_ipv4_address(&reply, 25).unwrap();
        assert_eq!(client, "192.0.2.10:50000".parse().unwrap());
        assert_eq!(u16::from_be_bytes([reply[next], reply[next + 1]]), 1200);
        assert_eq!(reply[next + 2], 0);
    }
}
