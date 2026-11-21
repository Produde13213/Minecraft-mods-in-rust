# Full Sodium + Iris implementation matrix

Target game: Horizon 26.3  
Target Horizon revision: `a02b2838d34f4c99726cca349ec0b4d62c5c8fa5`  
Target upstream references: Sodium 0.9.2+mc26.3, Iris 1.11.7+mc26.3

## Critical architecture decision

These are not Fabric mods that can be translated file-for-file into a standalone Rust plugin. Horizon is already a Rust game client with its own renderer. The real port target is feature parity implemented inside Horizon's renderer and shader pipeline. This repository records and tracks that work; it cannot become a drop-in Rust replacement JAR.

A full result means the feature groups below are implemented, tested, and integrated in Horizon. Existing Horizon features count only after their behaviour is verified against the target compatibility tests.

## Sodium parity matrix

| Feature area | Horizon evidence at pinned revision | Work required |
|---|---|---|
| Chunk section rendering and batching | `horizon-client-render` chunk renderer | Benchmark and compare behaviour |
| GPU vertex storage / arenas | `gfx/arena.rs` | Verify growth, reuse, compaction, stress cases |
| Frustum / occlusion culling | `gfx/cull.rs` | Differential visibility and edge-case tests |
| GPU-driven culling | `gfx/gpu_culling.rs` | Validate indirect draw correctness across backends |
| Render layers / terrain passes | Existing Horizon renderer | Map every target material/layer and transparency path |
| Entity/block-entity rendering | Existing renderer and shader integration | Verify shader-stage compatibility and batching |
| Translucency sorting | Existing renderer paths | Compare water/glass ordering and camera movement |
| Resource/memory controls | Existing arenas and renderer | Profile allocations, VRAM and large worlds |
| Options and diagnostics | Horizon settings/UI | Add missing user controls only where required |

**Sodium completion gate:** documented feature-by-feature parity, regression tests, renderer benchmarks on supported backends, and an in-game test matrix. A source-code resemblance is not a completion criterion.

## Iris parity matrix

| Feature area | Horizon evidence at pinned revision | Work required |
|---|---|---|
| Folder and ZIP pack discovery | `horizon-shaders::discover`, `Pack` | Test malformed, nested and large archives |
| GLSL preprocessing / includes | `preprocess` | Validate real-pack macros and include edge cases |
| Properties, profiles and options | `Properties`, `options`, `Screens` | Compare option semantics and persistence |
| GLSL program translation | `translate_* `, Naga | Run representative packs; fix translation gaps |
| Shadow pipeline | `translate_shadow` and renderer integration | Validate shadow maps, cascades, filtering and entities |
| G-buffer / terrain programs | `translate_gbuffers` | Validate terrain, cutout, translucent and water |
| Composite / deferred / final | `load_post` | Validate pass ordering, buffers and blending |
| Custom textures / uniforms | `load_textures`, post inputs | Verify documented extensions and pack expectations |
| Dimension-specific programs | `dimension_folder* ` | Test Overworld, Nether, End and custom dimensions |
| Pack UI and live reload | shader pack list/UI and native shader runtime | Verify selection, errors, option changes and reload |
| Mod compatibility | Existing Horizon mod/runtime interfaces | Define and test supported integration surface |
| GPU/backend portability | Naga + wgpu renderer | Validate Vulkan, DX12, Metal and GLES where supported |

**Iris completion gate:** compatibility results for a named pack corpus, all documented Tier A features, selected Tier E features explicitly marked, no silent rendering corruption, and in-game tests across dimensions and GPU backends.

## Work order

1. **Baseline:** build pinned Horizon and run its existing shader, renderer and real-pack tests.
2. **Sodium audit:** benchmark each mapped renderer subsystem; implement only measured gaps in Horizon.
3. **Iris audit:** run real packs through the existing loader; categorize failures by parser, translation, pipeline, or rendering.
4. **Implement:** fix the highest-impact reproducible gaps, adding regression tests with every fix.
5. **Integration:** run in-game pack tests and renderer regression/benchmark suite.
6. **Release:** publish exact supported feature matrix, limitations and tested builds.

## Current verified state

- Repository has the architecture and status documents.
- Horizon source inspection confirms substantial native equivalents for both projects.
- No Rust compiler/Cargo or network package installation is available in the current assistant execution container.
- No build, unit test, benchmark, or in-game test has been run here.
- No new Sodium/Iris source implementation is claimed yet.

## Honest completion policy

Do not mark either project complete based on scaffolding, documentation, or an uncompiled patch. Mark each feature complete only with a passing test or reproducible in-game verification attached to the change.
