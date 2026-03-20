# Scripting Language Migration Plan for Geometry Generation

## Status: Proposal

## Problem

`structure.rs` (648 lines) hardcodes all 3D geometry generation in Rust. Every change to window dimensions, roof trim, wall construction, or visual style requires recompilation. As the geometry vocabulary grows (hip roofs, dormers, cross-gables, parametric windows), this file will balloon and become the bottleneck for iteration speed, user customization, and architectural cleanliness.

## Current Architecture

```
SolvedBuilding (solver/types.rs)
  → generate_building_meshes() (solver/structure.rs)
    → Vec<BuildingMesh>
      → RenderScene::from_solved_building() (renderer/scene.rs)
        → DrawableMesh GPU uploads
```

The boundary is already clean: two public entry points (`generate_building_meshes`, `generate_edge_meshes`), consumed by exactly two callers (`renderer/scene.rs`, `renderer/screenshot.rs`). Input is a plain data tree (`SolvedBuilding`), output is a flat mesh list (`Vec<BuildingMesh>`). No callbacks, no shared mutable state, no GPU resources at this stage.

## Language Recommendation: Rhai

| Criterion | Rhai | Lua (mlua) | Starlark | WASM |
|-----------|------|------------|----------|------|
| **Rust-native** | Pure Rust, no FFI | C binding, unsafe | Rust impl | Varies |
| **Syntax** | Rust-like | Different conventions | Python-like | N/A |
| **Type registration** | Register Rust structs directly | Manual userdata tables | Limited | Complex serde |
| **Build deps** | `rhai = "1"`, no system libs | Needs C compiler | Heavier dep | Heavy toolchain |
| **Debugging** | Line-number errors | Good | Acceptable | Poor |
| **Sandboxing** | Built-in ops limit | Manual | Built-in | Built-in |

**Why not extend OIL?** OIL is declarative ("room entry, area: ~6sqm"). Geometry generation is imperative (for loops, math, conditionals). Forcing both paradigms into one language creates a franken-DSL. Keep OIL declarative, use Rhai for procedural geometry.

**Why not Lua?** `mlua` uses `unsafe` internally and requires a C toolchain. For geometry that runs once on startup (not per-frame), Lua's JIT speed advantage is irrelevant.

## Timing: Not Yet — But Prepare the Boundary Now

### Arguments against migrating now
- `structure.rs` is only 648 lines and works correctly
- OIL and `SolvedBuilding` are still evolving — the scripting API surface would be unstable
- The geometry primitive set (box, quad, triangle) may expand (CSG, curves, boolean ops)
- Every abstraction layer adds debugging overhead, dependency weight, and cognitive cost
- Current architecture is a single `cargo build` — no script versioning to manage

### When to pull the trigger
- When `structure.rs` exceeds ~1500 lines
- When you're implementing multiple roof forms (hip, shed, gambrel) or parametric components
- When users need to define custom geometry without the Rust toolchain
- When `SolvedBuilding` schema has been stable for several months

---

## Phase 0: Prepare the Boundary (Do Now)

Pure Rust refactoring. No new dependencies. Makes future migration trivial.

### 0a: Extract hardcoded constants into `GeometryConfig`

Move scattered constants into a data struct:

```rust
// solver/types.rs (or new solver/geometry_config.rs)
pub struct GeometryConfig {
    pub window_width: f32,      // currently WIN_W = 1.0
    pub window_height: f32,     // currently WIN_H = 1.2
    pub window_sill: f32,       // currently WIN_SILL = 0.9
    pub door_width: f32,        // currently DOOR_W = 0.9
    pub door_height: f32,       // currently DOOR_H = 2.1
    pub frame_thickness: f32,   // currently FRAME_T = 0.05
    pub frame_color: [f32; 3],  // currently [0.30, 0.25, 0.20]
    pub glass_color: [f32; 3],  // currently [0.65, 0.75, 0.85]
    // roof trim constants...
}
```

`generate_building_meshes` takes `&GeometryConfig` alongside `&SolvedBuilding`. Valuable independent of scripting — enables OIL style blocks to control these values.

### 0b: Extract primitives into `solver/primitives.rs`

Move to their own `pub` module:
- `make_box(w, d, h, cx, cy, cz, color) -> BuildingMesh`
- `make_quad(a, b, c, d, normal, color) -> BuildingMesh`
- `make_triangle(a, b, c, normal, color) -> BuildingMesh`
- `box_edges(w, d, h, cx, cy, cz, t, color) -> Vec<BuildingMesh>`
- `line_edge(a, b, t, color) -> BuildingMesh`

These become the future scripting API.

### 0c: Break `generate_building_meshes` into named sub-generators

```rust
fn generate_floor_slab(...) -> Vec<BuildingMesh>
fn generate_exterior_walls(...) -> Vec<BuildingMesh>
fn generate_ceiling(...) -> BuildingMesh
fn generate_interior_walls(...) -> Vec<BuildingMesh>  // already exists
fn generate_ground_plane(...) -> BuildingMesh
fn generate_roof(...) -> Vec<BuildingMesh>
```

Each becomes a separately scriptable unit later.

### File organization after Phase 0

```
src/solver/
  mod.rs           — unchanged
  types.rs         — add GeometryConfig
  structure.rs     — slimmed orchestrator, calls sub-generators
  primitives.rs    — NEW: geometry primitive helpers
  layout.rs        — unchanged
  style.rs         — resolves GeometryConfig from style block
```

---

## Phase 1: Add Scripting (When Triggered)

### 1a: Add dependency

```toml
[dependencies]
rhai = "1"
```

### 1b: Create `solver/scripting.rs`

```rust
pub fn run_geometry_script(
    building: &SolvedBuilding,
    config: &GeometryConfig,
    script_path: &Path,
) -> Result<Vec<BuildingMesh>>
```

Responsibilities:
1. Create Rhai `Engine` with operations limit
2. Register `BuildingMesh`, `MeshData`, `Vec3`, `Color` types
3. Register primitives (`make_box`, etc.) as native functions
4. Register read-only accessors for `SolvedBuilding` tree
5. Execute script, collect returned mesh list

### 1c: Create default geometry scripts

```
scripts/
  floor_slab.rhai
  exterior_walls.rhai
  interior_walls.rhai
  roof_gable.rhai
  roof_hip.rhai          ← new roof types added as scripts
  openings.rhai
  ground.rhai
```

### 1d: Wire up fallback chain

`generate_building_meshes` tries script first, falls back to Rust:

```rust
for sub_generator in [floor_slab, exterior_walls, roof, ...] {
    if script_exists(sub_generator) {
        meshes.extend(run_script(sub_generator)?);
    } else {
        meshes.extend(rust_fallback(sub_generator));
    }
}
```

This allows incremental migration — move gable roof to script while walls stay in Rust.

---

## Phase 2: Full Script Migration (When Stable)

### 2a: Move all sub-generators to Rhai scripts
### 2b: Bundle scripts via `include_str!()`

Scripts embedded in binary, overridable by files on disk. Single-executable distribution preserved.

### 2c: Remove replaced Rust implementations
### 2d: Add hot-reload

`R` key re-runs scripts and rebuilds `RenderScene` without restarting. Enables live geometry editing during development.

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Performance** | Geometry runs once on startup (~100-200 primitives). Rhai completes in <1ms. Not per-frame. |
| **Debugging** | Rhai gives file/line errors. Screenshot QA catches visual regressions. |
| **Distribution** | `include_str!()` keeps single binary. Override with adjacent `.rhai` files. |
| **Testing** | Integration tests: run scripts against known `SolvedBuilding`, compare mesh counts and bounding boxes. |
| **Stability** | Fallback chain means broken scripts don't break the app — Rust fallback always available. |

## Summary

**Do Phase 0 now** (pure refactoring, no risk, immediately useful). **Wait on Phase 1** until the geometry needs outgrow what's comfortable in Rust. The clean boundary we prepare in Phase 0 makes the Phase 1 migration mechanical rather than architectural.
