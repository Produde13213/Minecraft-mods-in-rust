# Geyser-RS

A clean-room Rust Bedrock-to-Java protocol bridge project.

## Current milestone

- Bounded Bedrock primitive codec: little-endian integers, unsigned VarInts,
  and VarInt-prefixed UTF-8 strings.
- Unit tests for boundary values, malformed VarInts, truncation, and round trips.
- UDP listener bootstrap on port 19132 for transport bring-up diagnostics.

## Not implemented yet

The UDP listener is **not** a working Bedrock server. RakNet connection
negotiation, Bedrock login/encryption, Java protocol connections, and packet
translation are still required before clients can join a Java server.

## Run

```sh
cargo test --manifest-path crates/geyser-rs/Cargo.toml
cargo run --manifest-path crates/geyser-rs/Cargo.toml
```

Set `GEYSER_BEDROCK_BIND` to override the default bind address.
