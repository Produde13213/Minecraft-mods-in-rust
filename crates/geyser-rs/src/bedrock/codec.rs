use thiserror::Error;

const MAX_STRING_BYTES: usize = 1_048_576;

/// Errors produced while decoding untrusted Bedrock packet data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("packet ended at byte {offset}; needed {needed} more byte(s)")]
    UnexpectedEof { offset: usize, needed: usize },
    #[error("invalid unsigned VarInt")]
    InvalidVarInt,
    #[error("string length {length} exceeds limit {limit}")]
    StringTooLong { length: usize, limit: usize },
    #[error("string is not valid UTF-8")]
    InvalidUtf8,
}

/// A bounds-checked reader for Bedrock's little-endian primitives and
/// unsigned VarInt-prefixed values.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        if self.remaining() < count {
            return Err(DecodeError::UnexpectedEof {
                offset: self.cursor,
                needed: count - self.remaining(),
            });
        }
        let start = self.cursor;
        self.cursor += count;
        Ok(&self.data[start..self.cursor])
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_u16_le(&mut self) -> Result<u16, DecodeError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, DecodeError> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Reads a 32-bit unsigned LEB128/Bedrock VarInt.
    pub fn read_var_u32(&mut self) -> Result<u32, DecodeError> {
        let mut value = 0u32;
        for index in 0..5 {
            let byte = self.read_u8()?;
            if index == 4 && (byte & 0xf0) != 0 {
                return Err(DecodeError::InvalidVarInt);
            }
            value |= u32::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(DecodeError::InvalidVarInt)
    }

    /// Reads a UTF-8 string with a Bedrock unsigned VarInt byte length.
    pub fn read_string(&mut self) -> Result<String, DecodeError> {
        let length = self.read_var_u32()? as usize;
        if length > MAX_STRING_BYTES {
            return Err(DecodeError::StringTooLong {
                length,
                limit: MAX_STRING_BYTES,
            });
        }
        let bytes = self.take(length)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| DecodeError::InvalidUtf8)
    }
}

/// A writer for Bedrock's little-endian primitives and unsigned VarInts.
#[derive(Debug, Default, Clone)]
pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_u16_le(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32_le(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32_le(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_var_u32(&mut self, mut value: u32) {
        while value >= 0x80 {
            self.data.push((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
        self.data.push(value as u8);
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_var_u32(value.len() as u32);
        self.data.extend_from_slice(value.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_round_trips_boundaries() {
        for value in [0, 1, 127, 128, 255, 16_383, 16_384, u32::MAX] {
            let mut writer = Writer::new();
            writer.write_var_u32(value);
            let bytes = writer.into_inner();
            let mut reader = Reader::new(&bytes);
            assert_eq!(reader.read_var_u32().unwrap(), value);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn little_endian_primitives_round_trip() {
        let mut writer = Writer::new();
        writer.write_u16_le(0x1234);
        writer.write_u32_le(0x1234_5678);
        writer.write_i32_le(-123_456);
        let bytes = writer.into_inner();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
        assert_eq!(reader.read_u32_le().unwrap(), 0x1234_5678);
        assert_eq!(reader.read_i32_le().unwrap(), -123_456);
    }

    #[test]
    fn strings_round_trip() {
        let mut writer = Writer::new();
        writer.write_string("Geyser → Rust");
        let bytes = writer.into_inner();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_string().unwrap(), "Geyser → Rust");
    }

    #[test]
    fn truncated_data_is_rejected() {
        let mut reader = Reader::new(&[0x80]);
        assert!(matches!(
            reader.read_var_u32(),
            Err(DecodeError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn overflowing_varint_is_rejected() {
        let mut reader = Reader::new(&[0xff, 0xff, 0xff, 0xff, 0x10]);
        assert_eq!(reader.read_var_u32(), Err(DecodeError::InvalidVarInt));
    }
}
