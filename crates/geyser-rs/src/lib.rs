//! Clean-room Rust implementation of the Bedrock-to-Java bridge protocol.
//!
//! The crate is split into protocol codecs, RakNet transport, Bedrock session
//! handling, Java protocol handling, and translation layers.

pub mod bedrock;
pub mod raknet;
pub mod connected;

pub use bedrock::codec::{DecodeError, Reader, Writer};
