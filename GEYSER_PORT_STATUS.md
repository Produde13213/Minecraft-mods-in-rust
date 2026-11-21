# Geyser-RS port status

**Upstream:** https://github.com/GeyserMC/Geyser  
**License:** MIT (upstream project)  
**Target:** standalone Rust Bedrock-to-Java proxy  
**Scope:** Geyser only. Floodgate is intentionally excluded.

## Implemented in the repository

- [x] Rust crate created at `crates/geyser-rs`.
- [x] Bounded Bedrock primitive reader/writer for little-endian integers.
- [x] Unsigned 32-bit Bedrock VarInt encode/decode.
- [x] VarInt-prefixed UTF-8 string support with a size limit.
- [x] Unit tests authored for codec round trips, boundaries, truncation and malformed VarInts.
- [x] Tokio UDP listener bootstrap on the default Bedrock port (19132).
- [x] RakNet unconnected ping parsing and pong response, with tests.
- [x] Bedrock server-list advertisement defaults (version 26.65 / protocol 2225), with configurable MOTD.

## Not implemented yet

The current server can answer a Bedrock server-list discovery ping, but it **cannot accept a Bedrock player or connect to a Java server yet**. These major subsystems remain:

1. RakNet open-connection negotiation.
2. RakNet reliability, ordering, fragmentation and reassembly.
3. Bedrock network packet framing, login, JWT chain validation and encryption.
4. Java Edition TCP connection, handshake, status/login and version negotiation.
5. Bedrock-to-Java packet translation, world/chunk/entity/inventory handling.
6. Configuration, authentication modes, resource packs, commands and operational compatibility.

## Architecture plan

- `bedrock/codec.rs`: bounded primitive packet codec.
- `raknet.rs`: currently offline discovery; next is connection negotiation and reliable transport.
- `bedrock/packet/`: Bedrock packet IDs and payload codecs.
- `java/`: Java Edition protocol framing and connection state.
- `session/`: one Bedrock-to-Java player session.
- `translate/`: bidirectional packet translation.
- `config/`: proxy configuration and validation.

## Verification

Tests are authored but have **not** been run in the assistant execution environment. Do not describe this milestone as a working Geyser replacement. The next acceptance gate is a tested RakNet open-connection handshake and reliable session transport, followed by a real Bedrock client connection.
