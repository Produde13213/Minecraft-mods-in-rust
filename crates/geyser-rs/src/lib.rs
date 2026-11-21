//! Clean-room Rust implementation of the Bedrock-to-Java bridge protocol.
//!
//! The crate is intentionally split into protocol codecs, RakNet transport,
//! Bedrock session handling, Java protocol handling, and translation layers.
//! This first milestone implements the bounded Bedrock primitive codec.

pub mod bedrock;

pub use bedrock::codec::{DecodeError, Reader, Writer};
