# Geyser-RS

A clean-room Rust Bedrock-to-Java protocol bridge project.

## Current milestone

- Bounded Bedrock primitive reader/writer: little-endian integers, unsigned
  VarInts, and VarInt-prefixed UTF-8 strings.
- RakNet unconnected ping/pong, allowing the server to answer Bedrock server-list
  discovery requests.
- Unit tests for codec boundaries, malformed data, and RakNet discovery packets.
- Tokio UDP listener on port 19132.

## Not implemented yet

This is **not yet a working Bedrock server or Java proxy**. RakNet open-connection
negotiation and reliable sessions, Bedrock login/encryption, Java protocol
connections, and packet translation are still required before clients can join
a Java server.

## Run

```sh
cargo test --manifest-path crates/geyser-rs/Cargo.toml
cargo run --manifest-path crates/geyser-rs/Cargo.toml
```

Set `GEYSER_BEDROCK_BIND` to override the bind address and `GEYSER_MOTD` to
change the server-list description.
