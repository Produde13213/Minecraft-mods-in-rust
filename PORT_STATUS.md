# Sodium + Iris port status

**Audit target:** Horizon 26.3, commit `a02b2838d34f4c99726cca349ec0b4d62c5c8fa5`  
**Repository target:** `Produde13213/Minecraft-mods-in-rust`  
**Status:** engine audit complete; no new native mod implementation or runtime validation yet.

## Executive result

A literal Sodium port and a literal Iris port are not the right deliverables for this Horizon revision.

Horizon already has a substantial renderer and a separate OptiFine/Iris-style shader-pack pipeline. The best path is to validate and fill compatibility gaps in those existing systems, rather than add a duplicate renderer or duplicate shader loader.

## Sodium: what Horizon already has

The Horizon renderer contains equivalents for important Sodium responsibilities:

- GPU vertex-buffer arenas and range allocation/reuse: `crates/horizon-client-render/src/gfx/arena.rs`
- CPU occlusion culling: `crates/horizon-client-render/src/gfx/cull.rs`
- GPU-driven culling and indirect draw generation: `crates/horizon-client-render/src/gfx/gpu_culling.rs`
- Chunk/world render data and batching: `crates/horizon-client-render/src/chunk.rs`, `chunk_map.rs`, and the rest of `crates/horizon-client-render/src/gfx/`

These are Horizon-native systems, not Sodium code. Porting Sodium wholesale would duplicate the renderer and would not automatically improve performance.

### Remaining Sodium work

1. Benchmark Horizon's current chunk renderer and identify measurable gaps against the desired workload.
2. Map only missing renderer capabilities to Horizon's existing abstractions.
3. Add focused Rust changes only where a benchmark or correctness test demonstrates a gap.
4. Run renderer tests and in-game performance/correctness checks.

**Not claimed:** Sodium feature parity, performance parity, or a completed Sodium port.

## Iris: what Horizon already has

Horizon documents a separate loader for Iris/OptiFine-format packs (the `shaders/` directory format), distinct from Horizon's own `.hshader` format.

The pinned source includes:

- `crates/horizon-shaders`: pack discovery, ZIP/folder reading, properties, language files, pack options/screens, includes and GLSL preprocessing, shader program translation, post passes, custom textures, and related metadata.
- `naga` GLSL translation and renderer integration.
- `crates/horizon-client-render/src/shader_pack_list.rs` and `shader_ui.rs`: pack listing and user interface.
- `crates/horizon-client/src/game/interface/shader_packs.rs`: selection and path resolution.
- `crates/horizon-client/src/game/modding/native_shaders.rs`: runtime pack selection, reload/error handling and renderer hand-off.
- Integration tests and fixture packs under `crates/horizon-shaders/tests`.

The guide says packs can be placed in `shaderpacks/`, selected in the shader-pack UI, and reloaded while running. The clean-room format specification is at `docs/cleanroom/shader-packs-spec.md`.

### Remaining Iris compatibility work

1. Build a compatibility matrix against the supplied Iris 1.11.7 / Minecraft 26.3 target and representative shader packs.
2. Run the existing `horizon-shaders` real-pack tests and inspect any failures.
3. Identify missing Iris-specific extensions or rendering semantics from test evidence.
4. Implement those gaps in Horizon's existing Rust loader/renderer, without copying Iris implementation code.
5. Validate pack loading, shadows, terrain, entities, water/translucency, post-processing, options and reloads in-game.

**Not claimed:** complete Iris 1.11.7 compatibility or that every third-party shader pack works.

## Dependency decision

Do not make the Horizon-native shader project depend on Sodium's Java implementation. Horizon's renderer is the backend. The shader loader should integrate with Horizon's own render passes and Naga/wgpu pipeline.

## Toolchain / verification

The assistant's execution container currently has neither `cargo` nor `rustc`, and DNS/outbound package access is unavailable. Therefore no Rust compilation or tests have been run in this environment. The code must be built in a Rust-enabled environment against the exact pinned Horizon workspace before it can be called complete.

## Completion criteria

- [ ] Horizon workspace builds on the pinned revision.
- [ ] Existing shader crate unit/integration tests pass.
- [ ] Sodium-related renderer benchmarks and correctness tests are recorded.
- [ ] Compatibility matrix covers representative Iris/OptiFine shader packs.
- [ ] Any identified gaps are fixed in Horizon-native Rust.
- [ ] In-game test confirms pack selection, rendering, options and reloads.
- [ ] No claim of completion until the above checks are evidenced.
