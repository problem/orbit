# Architecture Review — Orbit CAD

*Reviewer perspective: 20 years on SketchUp's engineering team*

## Strengths

### Declarative-first design (OIL language)
The constraint-based building description is a genuinely differentiating bet. Most CAD tools start with direct manipulation and bolt on constraints later — and it's always painful. Starting constraints-first gives Orbit a real shot at something different. The `~25sqm` approximate value syntax with tolerance ranges is particularly clever; it acknowledges that architecture is about intent, not just dimensions.

### SQLite file format (.orb)
Correct choice. SKP's custom binary format caused endless grief — versioning, partial reads, corruption recovery, concurrent access. SQLite gives you ACID transactions, partial reads, schema migration via `ALTER TABLE`, and bindings in every language. The `application_id` / `format_version` header pattern and WAL journal mode are textbook correct.

### Lean dependency set
`wgpu` + `winit` + `nalgebra` + `rusqlite` is a tight, long-lived foundation. No framework bloat. No ECS engine to fight. This kind of discipline keeps a project alive for a decade.

---

## Critical Issues

### 1. BSP layout algorithm is a dead end
The recursive binary space partition in `layout.rs` splits rooms proportional to area along the longer axis. This can only produce rectangular grids. It cannot produce L-shaped rooms, wrap-around rooms, hallways connecting non-adjacent spaces, stairwells punching through floor plates, or upper-floor setbacks. Adjacency constraints are parsed and ordered-for but never verified post-layout — two rooms declared `adjacent_to` can end up on opposite sides of the plan.

**Recommendation:** Use BSP as an initial seed, then add a refinement pass (simulated annealing with adjacency cost functions, or generate-and-score candidate layouts).

### 2. No undo/redo architecture
No undo mechanism exists anywhere. For a CAD tool, undo is a foundational architectural decision that affects every data structure. The current pipeline (`OIL text → solve → render`) is a compiler, not a CAD tool. The `.orb` format has a full scene graph — that's the skeleton of an interactive editor. Decide which you're building and commit. Straddling both will hurt.

### 3. Flat scene graph
`RenderScene` is `Vec<DrawableMesh>` — no hierarchy, no grouping, no instancing. Every wall/slab/roof face is an independent mesh with its own GPU buffer and bind group. This was the exact antipattern that made SketchUp choke on large models. The `.orb` format already has `parent_id` and `component_instance` entity types, but the renderer ignores hierarchy.

**Needed:** Scene tree matching entity hierarchy, instance rendering, frustum culling at group level, LOD (schema has `lod_level` but nothing uses it).

### 4. No document/view separation
`SolvedBuilding` is the only in-memory representation and it's a one-shot solver output. There's no persistent document the renderer observes and the editor mutates. Any change re-parses everything, re-solves everything, regenerates every mesh. Fine at 20 meshes, a disaster at 2,000.

### 5. Geometry pipeline too primitive
`structure.rs` generates buildings from boxes. No window/door openings, no interior walls, no stairs, no hip/shed/gambrel roofs (parsed in AST but not generated), no trim/fascia. The solver knows about 3 south-facing windows in the kitchen but the renderer draws a solid wall. Need at minimum a simple CSG/boolean kernel for wall openings.

---

## .orb Format Issues

| Issue | Detail |
|-------|--------|
| **No schema migration path** | `format_version` exists but no migration table/code. The moment v1 ships and a column needs adding, this becomes urgent. |
| **Textures as BLOBs** | In the main database, will bloat file and reduce page cache effectiveness. Should be a sidecar/attached DB. |
| **Empty B-rep table** | `orb_geometry_brep` defined but nothing writes to it. For architectural CAD, B-rep should be first-class (parametric dims, booleans, fillets). |
| **No change journaling** | For collaboration and undo, need an operations/changes table logging mutations. |
| **128-byte transforms everywhere** | f64 4x4 for every entity is overkill for objects that only need translation + rotation. Consider compact representation with full-matrix fallback. |

---

## Renderer Issues

- **No texturing** — full PBR material model in `.orb`, but shader is flat `ambient + diffuse`
- **No shadows** — buildings without shadows look like floating objects
- **Per-mesh uniform upload every frame** — needs dynamic/storage buffer with instancing
- **Wireframe edges are geometry** (thin boxes) — expensive; use line-rendering pass or post-process edge detection

---

## Strategic Recommendation

Orbit is currently three products at once:
1. A **declarative building compiler** (OIL → geometry → screenshots) — novel and interesting
2. An **interactive CAD viewer** (orbit camera, view modes) — needs richer scene graph and editing model
3. A **file format spec** (`.orb` with scene graph, materials, clash detection) — enterprise ambitions

**Recommendation:** Double down on the OIL compiler path first. Generate genuinely useful floor plans with proper openings, interior walls, and multiple roof forms. Make screenshots good enough to show a client. That's the unique value. The interactive editor and enterprise format can come later once the geometry engine is worth interacting with.

The `.orb` format is over-engineered for today's product but well-designed enough to grow into. That's the right kind of over-engineering — invest in the data model, stay lean on the UI.
