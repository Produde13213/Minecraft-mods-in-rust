//! RakNet offline discovery packets.
//!
//! This module handles only unconnected ping/pong. It does not implement
//! connection negotiation or reliable RakNet sessions yet.

pub const UNCONNECTED_PING: u8 = 0x01;
pub const UNCONNECTED_PONG: u8 = 0x1c;
pub const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

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

/// Returns a pong for a valid RakNet unconnected ping, or None for any other
/// datagram. The response echoes the client's 64-bit timestamp.
pub fn answer_unconnected_ping(
    packet: &[u8],
    advertisement: &ServerAdvertisement,
) -> Option<Vec<u8>> {
    // ID + timestamp + magic + client GUID.
    if packet.len() < 33 || packet[0] != UNCONNECTED_PING {
        return None;
    }
    if packet[9..25] != RAKNET_MAGIC {
        return None;
    }

    let timestamp = &packet[1..9];
    let mut response = Vec::with_capacity(35 + 160);
    response.push(UNCONNECTED_PONG);
    response.extend_from_slice(timestamp);
    response.extend_from_slice(&advertisement.server_id.to_be_bytes());
    response.extend_from_slice(&RAKNET_MAGIC);

    let motd = advertisement.motd.replace(';', ":");
    let advertised_players = advertisement.online_players;
    let max_players = advertisement.max_players.max(advertised_players.saturating_add(1));
    let text = format!(
        "MCPE;{};{};{};{};{};{};{};Survival;1;{};{}",
        motd,
        advertisement.protocol,
        advertisement.version,
        advertised_players,
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
}
