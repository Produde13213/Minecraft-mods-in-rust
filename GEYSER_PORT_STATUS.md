# Geyser-RS port status

**Upstream:** https://github.com/GeyserMC/Geyser  
**License:** MIT (upstream project)  
**Target:** standalone Rust Bedrock-to-Java proxy  
**Scope:** Geyser only. Floodgate is intentionally excluded.

## Implemented in the repository

- [x] Rust crate at `crates/geyser-rs`.
- [x] Bounded Bedrock primitive reader/writer: little-endian integers, unsigned VarInts and length-prefixed UTF-8 strings.
- [x] RakNet unconnected ping/pong for server-list discovery.
- [x] RakNet Open Connection Request 1 / Reply 1.
- [x] RakNet Open Connection Request 2 / Reply 2 for IPv4 peers.
- [x] UDP listener routes discovery and open-connection negotiation packets.
- [x] Unit tests authored for codec boundaries, malformed data, discovery and handshake packet layouts.

## Not implemented yet

This is **not yet a working Bedrock server or Java proxy**. The current server can answer discovery and the two offline open-connection negotiation requests, but it does not yet establish a connected RakNet session.

Remaining major subsystems:

1. Connected RakNet reliability, ordering, ACK/NACK, fragmentation and reassembly.
2. Connected Connection Request / New Incoming Connection handshake.
3. Bedrock packet framing, login, JWT chain validation and encryption.
4. Java Edition TCP connection, handshake, login and protocol version negotiation.
5. Bedrock-to-Java packet translation, world/chunk/entity/inventory handling.
6. Configuration, authentication modes, resource packs, commands and operational compatibility.
7. IPv6 RakNet address encoding.

## Architecture plan

- `bedrock/codec.rs`: bounded primitive packet codec.
- `raknet.rs`: discovery and offline open-connection handshake; connected transport remains.
- `bedrock/packet/`: Bedrock packet IDs and payload codecs.
- `java/`: Java Edition protocol framing and connection state.
- `session/`: one Bedrock-to-Java player session.
- `translate/`: bidirectional packet translation.
- `config/`: proxy configuration and validation.

## Verification

Tests are authored but have **not** been run in the assistant execution environment. Do not describe this milestone as a working Geyser replacement. The next acceptance gate is a connected RakNet session with reliability and ordering, followed by Bedrock login.
