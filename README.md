# Minecraft mods in Rust — Horizon shader integration

This repository tracks native Rust work for Horizon (Minecraft 26.3).

## Important discovery (pinned Horizon revision)

At Horizon revision `a02b2838d34f4c99726cca349ec0b4d62c5c8fa5`, Horizon already contains both sides of the work this project initially planned to port:

- **Sodium-like renderer foundations:** Horizon has its own chunk renderer, GPU vertex arenas, occlusion culling, GPU-driven culling and indirect draws. A second Sodium renderer would duplicate the engine.
- **Iris/OptiFine shader-pack support:** Horizon already has a `horizon-shaders` crate that reads `shaderpacks/*.zip`, parses properties/options, preprocesses GLSL, translates programs through Naga, and feeds the renderer. The game has shader-pack selection UI, per-pack options and reload support.

The goal is therefore **native Horizon compatibility and gap-filling**, not copying Sodium or Iris source code. Both upstream projects have their own licenses; this repository must not mechanically translate or copy their implementations.

## Upstream references

- Horizon source: https://github.com/horizon-rust/horizon/tree/a02b2838d34f4c99726cca349ec0b4d62c5c8fa5
- Horizon shader guide: https://github.com/horizon-rust/horizon/blob/a02b2838d34f4c99726cca349ec0b4d62c5c8fa5/docs/shaders.md
- Clean-room shader-pack specification: https://github.com/horizon-rust/horizon/blob/a02b2838d34f4c99726cca349ec0b4d62c5c8fa5/docs/cleanroom/shader-packs-spec.md
- Shader implementation: https://github.com/horizon-rust/horizon/tree/a02b2838d34f4c99726cca349ec0b4d62c5c8fa5/crates/horizon-shaders
- Shader API: https://github.com/horizon-rust/horizon/tree/a02b2838d34f4c99726cca349ec0b4d62c5c8fa5/crates/horizon-shader-api

## Status

See [PORT_STATUS.md](PORT_STATUS.md) for the exact audit and remaining compatibility work.

## Build note

Builds must use the pinned Horizon revision and its workspace toolchain. Do not claim a mod builds or runs until Cargo tests and an in-game shader-pack smoke test pass.

## Geyser-RS (standalone proxy)

A separate clean-room Rust port of Geyser's Bedrock-to-Java bridge is being developed at [`crates/geyser-rs`](crates/geyser-rs). It is a standalone proxy, not a Horizon client mod. Floodgate is out of scope.

See [GEYSER_PORT_STATUS.md](GEYSER_PORT_STATUS.md) for implemented components, missing protocol layers and verification status. The current crate is an early foundation and is **not yet a usable Bedrock-to-Java proxy**.
