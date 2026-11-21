//! Connected RakNet datagram parsing primitives.
//!
//! This module deliberately parses frames without implementing socket-level
//! retransmission yet. It validates lengths and metadata before exposing payloads.

use crate::bedrock::codec::{DecodeError, Reader};

pub const ACK: u8 = 0xc0;
pub const NACK: u8 = 0xa0;
pub const DATA_PACKET_MIN: u8 = 0x80;
pub const DATA_PACKET_MAX: u8 = 0x8d;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triad(pub u32);

impl Triad {
    pub fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let bytes = reader.read_exact(3)?;
        Ok(Self(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])))
    }

    pub fn write(self, out: &mut Vec<u8>) {
        let bytes = self.0.to_le_bytes();
        out.extend_from_slice(&bytes[..3]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceRange {
    pub start: Triad,
    pub end: Triad,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckPacket {
    pub is_nack: bool,
    pub ranges: Vec<SequenceRange>,
}

impl AckPacket {
    pub fn decode(packet: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(packet);
        let id = reader.read_u8()?;
        if id != ACK && id != NACK {
            return Err(DecodeError::InvalidData("not an ACK/NACK packet"));
        }
        let count = reader.read_u16_be()? as usize;
        if count > 4096 {
            return Err(DecodeError::InvalidData("ACK/NACK range count exceeds limit"));
        }
        let mut ranges = Vec::with_capacity(count);
        for _ in 0..count {
            let single = reader.read_u8()? != 0;
            let start = Triad::read(&mut reader)?;
            let end = if single { start } else { Triad::read(&mut reader)? };
            ranges.push(SequenceRange { start, end });
        }
        if reader.remaining() != 0 {
            return Err(DecodeError::InvalidData("trailing bytes in ACK/NACK packet"));
        }
        Ok(Self { is_nack: id == NACK, ranges })
    }

    pub fn encode(&self) -> Result<Vec<u8>, DecodeError> {
        let count = u16::try_from(self.ranges.len())
            .map_err(|_| DecodeError::InvalidData("too many ACK/NACK ranges"))?;
        let mut out = Vec::with_capacity(3 + self.ranges.len() * 7);
        out.push(if self.is_nack { NACK } else { ACK });
        out.extend_from_slice(&count.to_be_bytes());
        for range in &self.ranges {
            let single = range.start == range.end;
            out.push(u8::from(single));
            range.start.write(&mut out);
            if !single {
                range.end.write(&mut out);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncapsulatedFrame {
    pub reliability: u8,
    pub reliable_index: Option<Triad>,
    pub sequence_index: Option<Triad>,
    pub ordering_index: Option<Triad>,
    pub ordering_channel: Option<u8>,
    pub split: Option<SplitInfo>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitInfo {
    pub count: u32,
    pub id: u16,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDatagram {
    pub sequence: Triad,
    pub frames: Vec<EncapsulatedFrame>,
}

impl DataDatagram {
    pub fn decode(packet: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(packet);
        let id = reader.read_u8()?;
        if !(DATA_PACKET_MIN..=DATA_PACKET_MAX).contains(&id) {
            return Err(DecodeError::InvalidData("not a RakNet data datagram"));
        }
        let sequence = Triad::read(&mut reader)?;
        let mut frames = Vec::new();
        while reader.remaining() > 0 {
            if frames.len() >= 512 {
                return Err(DecodeError::InvalidData("too many encapsulated frames"));
            }
            let flags = reader.read_u8()?;
            let reliability = (flags >> 5) & 0x07;
            let split_flag = flags & 0x10 != 0;
            let bit_length = reader.read_u16_be()? as usize;
            if bit_length == 0 || bit_length % 8 != 0 {
                return Err(DecodeError::InvalidData("frame payload length is not byte aligned"));
            }
            let reliable_index = if matches!(reliability, 2 | 3 | 4 | 6 | 7) {
                Some(Triad::read(&mut reader)?)
            } else { None };
            let sequence_index = if matches!(reliability, 1 | 4 | 5) {
                Some(Triad::read(&mut reader)?)
            } else { None };
            let (ordering_index, ordering_channel) = if matches!(reliability, 1 | 3 | 4 | 7) {
                (Some(Triad::read(&mut reader)?), Some(reader.read_u8()?))
            } else { (None, None) };
            let split = if split_flag {
                let count = reader.read_u32_be()?;
                let id = reader.read_u16_be()?;
                let index = reader.read_u32_be()?;
                if count == 0 || count > 4096 || index >= count {
                    return Err(DecodeError::InvalidData("invalid split-frame metadata"));
                }
                Some(SplitInfo { count, id, index })
            } else { None };
            let payload_len = bit_length / 8;
            let payload = reader.read_exact(payload_len)?.to_vec();
            frames.push(EncapsulatedFrame {
                reliability, reliable_index, sequence_index, ordering_index,
                ordering_channel, split, payload,
            });
        }
        Ok(Self { sequence, frames })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triad_roundtrip_and_wrap_width() {
        let mut bytes = Vec::new();
        Triad(0xabcdef).write(&mut bytes);
        assert_eq!(bytes, [0xef, 0xcd, 0xab]);
        let mut reader = Reader::new(&bytes);
        assert_eq!(Triad::read(&mut reader).unwrap(), Triad(0xabcdef));
    }

    #[test]
    fn ack_and_nack_roundtrip() {
        for is_nack in [false, true] {
            let packet = AckPacket {
                is_nack,
                ranges: vec![
                    SequenceRange { start: Triad(7), end: Triad(7) },
                    SequenceRange { start: Triad(10), end: Triad(13) },
                ],
            };
            assert_eq!(AckPacket::decode(&packet.encode().unwrap()).unwrap(), packet);
        }
    }

    #[test]
    fn parses_unreliable_frame() {
        let mut packet = vec![0x80];
        Triad(9).write(&mut packet);
        packet.push(0x00); // Unreliable, not split.
        packet.extend_from_slice(&24u16.to_be_bytes());
        packet.extend_from_slice(b"abc");
        let decoded = DataDatagram::decode(&packet).unwrap();
        assert_eq!(decoded.sequence, Triad(9));
        assert_eq!(decoded.frames.len(), 1);
        assert_eq!(decoded.frames[0].payload, b"abc");
        assert_eq!(decoded.frames[0].reliability, 0);
    }

    #[test]
    fn rejects_truncated_frame() {
        let mut packet = vec![0x80];
        Triad(1).write(&mut packet);
        packet.push(0);
        packet.extend_from_slice(&80u16.to_be_bytes());
        packet.extend_from_slice(b"short");
        assert!(DataDatagram::decode(&packet).is_err());
    }
}
