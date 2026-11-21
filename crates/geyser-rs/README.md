# Geyser-RS

A clean-room Rust Bedrock-to-Java protocol bridge project.

## Current milestone

- Bounded Bedrock primitive reader/writer: little-endian integers, unsigned
  VarInts, and VarInt-prefixed UTF-8 strings.
- RakNet unconnected ping/pong for Bedrock server-list discovery.
- RakNet Open Connection Request 1 / Reply 1.
- RakNet Open Connection Request 2 / Reply 2 for IPv4 peers.
- Unit tests authored for codec boundaries, malformed data, discovery and
  connection-negotiation packet layouts.
- Tokio UDP listener on port 19132.

## Not implemented yet

This is **not yet a working Bedrock server or Java proxy**. Connected RakNet
reliability, ordering, fragmentation and reassembly, the connected
Connection Request/New Incoming Connection exchange, Bedrock login/encryption,
Java protocol connections, and packet translation are still required before
clients can join a Java server.

IPv6 RakNet address encoding is also not implemented yet.

## Run

```sh
cargo test --manifest-path crates/geyser-rs/Cargo.toml
cargo run --manifest-path crates/geyser-rs/Cargo.toml
```

Set `GEYSER_BEDROCK_BIND` to override the bind address and `GEYSER_MOTD` to
change the server-list description.
