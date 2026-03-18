# Orbit CAD — Development Guide

## Building & Running

```bash
RUST_LOG=info cargo run
```

On startup, Orbit:
1. Kills any previously running orbit process
2. Parses the OIL source in `src/main.rs`
3. Solves the floor plan and generates 3D geometry
4. Opens an interactive window with orbit camera (drag to rotate, scroll to zoom)
5. Exports screenshots to `screenshots/` (sequentially numbered, 4 angles: front, gable, rear, above)

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **E** | Cycle view modes: Solid → Solid+Wireframe → Wireframe Only → Solid |
| **Escape** | Close window |
| Mouse drag | Orbit camera |
| Scroll | Zoom in/out |

## Troubleshooting Geometry

### View Modes (press E to cycle)
- **Solid**: Normal shaded rendering (default)
- **Solid + Wireframe**: Shaded surfaces with dark edge lines overlaid — best for spotting gaps and misaligned edges on the exterior
- **Wireframe Only**: See-through wireframe — best for inspecting interior geometry, overlapping meshes, and hidden faces

These modes reveal:
- Missing faces (holes in the surface)
- Overlapping geometry (z-fighting visible as flickering)
- Misaligned edges (gaps between boxes that should snap)
- Backface culling issues (faces that disappear at certain angles)

### Wireframe Screenshots
To export wireframe screenshots programmatically, use:
```rust
orbit::renderer::screenshot::render_building_to_png_wireframe(
    &building, &camera, 1920, 1080, &path,
)
```

### Common Issues
- **Missing faces from certain angles**: Backface culling is disabled (`cull_mode: None`) to avoid this. If faces disappear, check that the pipeline has `cull_mode: None`.
- **box_mesh parameter order**: `MeshData::box_mesh(width, height, depth)` maps to **(X, Z, Y)** — height is the VERTICAL axis (Z), not Y. Use the `make_box(w, d, h)` wrapper which handles this correctly.
- **Roof gaps**: Roof slopes use exact-vertex quads (not rotated boxes). Gable triangles must extend to the overhang extent to avoid gaps at eaves.

## Architecture

```
OIL text → Parser (src/oil/) → AST → Solver (src/solver/) → 3D Geometry → Renderer (src/renderer/) → Screen
```

### Key Modules
- `src/oil/` — OIL language parser (lexer + recursive descent)
- `src/solver/` — Constraint solver: style resolution, floor plan layout (BSP), structure generation
- `src/renderer/` — wgpu renderer: camera, pipeline, scene, headless screenshot export
- `src/orb/` — .orb file format I/O (SQLite-based)
- `src/spatial/` — Spatial index (R-tree), occupancy engine, clash detection types

### Screenshot QA
Every `cargo run` exports 4 screenshots from different angles. All 4 must look correct — a bug visible from only one angle is still a bug. Check `screenshots/` for the latest set.

**IMPORTANT: Do NOT delete screenshots unless explicitly asked by the user.** The screenshot archive is a visual progress log. Old screenshots are valuable for comparing before/after.
