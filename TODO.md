# Orbit CAD — Development TODO

Prioritized based on the [Architecture Review](ARCHITECTURE_REVIEW.md) strategic recommendation:
**Double down on the OIL compiler path. Generate genuinely useful buildings with proper openings, interior walls, and multiple roof forms. Make screenshots good enough to show a client.**

---

## Phase 1: Geometry Engine — Make Buildings Look Real
*Priority: HIGHEST. This is the unique value. Nothing else matters until the output is useful.*

- [ ] **1.1 Wall openings (windows & doors)**
  - CSG/boolean subtraction: cut rectangular holes in wall boxes
  - Place window frames (recessed box) and door frames based on OIL `windows:` and `has: front_door`
  - Glass pane mesh (semi-transparent) for windows
  - Each opening is an entity in the scene graph with proper AABB

- [ ] **1.2 Interior walls**
  - Restore interior partition walls from the solver (removed for exterior-only view)
  - Add section-cut view mode (E cycles: Solid → Solid+Wireframe → Wireframe → Section Cut)
  - Section cut at floor level shows floor plan with room colors and wall outlines
  - Interior walls terminate cleanly at exterior walls (clipped to interior zone)

- [ ] **1.3 Multiple roof forms**
  - Hip roof (all 4 sides slope, no gable walls)
  - Shed roof (single slope)
  - Cross-gable (intersecting gable volumes — already parsed in OIL)
  - Dormers (small gable projections on roof slope — already parsed)
  - All use exact-vertex quads, no rotation math

- [ ] **1.4 Multi-story buildings**
  - Restore the full Tudor OIL (2 floors, 12 rooms, garage)
  - Stairwell openings between floors
  - Per-floor interior walls with proper vertical stacking
  - Upper floor setbacks (smaller footprint than ground floor)

- [ ] **1.5 Chimneys & exterior details**
  - Chimney as a tall box attached to an exterior wall
  - Porch/entry roof overhang
  - Foundation visible as a slightly wider base course

---

## Phase 2: Layout Algorithm — Make Floor Plans Useful
*Priority: HIGH. BSP is a dead end for real architecture.*

- [ ] **2.1 Adjacency verification**
  - After BSP layout, check all `adjacent_to` constraints
  - Report violations as solver diagnostics
  - Score layout quality (% of adjacency constraints satisfied)

- [ ] **2.2 Layout refinement pass**
  - Simulated annealing: swap room positions to improve adjacency score
  - Cost function: adjacency violations + aspect ratio deviation + area deviation
  - Run for N iterations after BSP seed, keep best layout

- [ ] **2.3 Non-rectangular rooms**
  - L-shaped rooms (merge two rectangles)
  - Hallways as linear connectors between rooms
  - Garage attached to side of building (different ceiling height)

- [ ] **2.4 Staircase placement**
  - Automatic staircase between floors with aligned opening
  - Vertical alignment constraint between floor layouts
  - Staircase geometry (steps, landing, railing)

---

## Phase 3: Document Model & Scene Graph
*Priority: MEDIUM. Required for interactive editing, not for the compiler path.*

- [ ] **3.1 Document type**
  - `Document` struct: persistent in-memory model the renderer observes
  - Wraps `SolvedBuilding` + metadata + dirty flags
  - Incremental updates: change a room area → re-solve only that floor → regenerate only affected meshes

- [ ] **3.2 Scene tree with hierarchy**
  - Replace flat `Vec<DrawableMesh>` with tree matching .orb entity hierarchy
  - Building → Floor → Room → Wall/Opening/Fixture
  - Group-level transforms, visibility, selection
  - Frustum culling at group level

- [ ] **3.3 Instance rendering**
  - Component instances share mesh data (e.g., all standard windows use one mesh)
  - GPU instancing: single draw call for N instances of the same component
  - Reduces draw calls from O(entities) to O(unique_components)

- [ ] **3.4 Undo/redo architecture**
  - Command pattern: every mutation is an undoable `Operation`
  - Operation log stored in .orb (enables collaboration replay)
  - Inverse operations for undo, forward replay for redo

---

## Phase 4: Renderer — Make It Look Professional
*Priority: MEDIUM-LOW. Visual polish after geometry is correct.*

- [ ] **4.1 Shadow mapping**
  - Directional light shadow map (sun shadows)
  - Buildings without shadows look like floating objects
  - Shadow acne prevention (bias)

- [ ] **4.2 Storage buffer for uniforms**
  - Replace per-mesh uniform buffer with single storage buffer
  - All transforms + colors in one GPU buffer, indexed by instance ID
  - Enables instanced rendering

- [ ] **4.3 PBR materials**
  - Use the material model already in .orb (metallic, roughness, normal maps)
  - Texture sampling for brick, stone, stucco, wood
  - Environment map for reflections

- [ ] **4.4 Post-process edge detection**
  - Replace geometry-based edge strips with screen-space edge detection shader
  - Detect depth/normal discontinuities → draw edges
  - Zero geometry overhead, works on any mesh

---

## Phase 5: .orb Format Maturity
*Priority: LOW for now. "Well-designed enough to grow into."*

- [ ] **5.1 Schema migration**
  - `orb_migrations` table tracking applied versions
  - Migration functions: v1.0→v1.1, v1.1→v1.2, etc.
  - `ALTER TABLE` for additive changes, data transforms for breaking changes

- [ ] **5.2 B-rep geometry**
  - Start writing `orb_geometry_brep` alongside mesh data
  - Parametric wall definitions (width, height, openings list)
  - Enables re-tessellation at different LOD levels

- [ ] **5.3 Texture sidecar**
  - Move texture BLOBs to `orb_textures.db` attached database
  - Keep references in main DB, lazy-load textures
  - Keeps main DB small and page-cache effective

- [ ] **5.4 Change journaling**
  - `orb_operations` table: timestamped mutation log
  - Each operation: type, entity_id, before/after values
  - Foundation for undo/redo and multi-user collaboration

---

## Current Status
- [x] OIL parser (lexer + recursive descent, handles Tudor canonical example)
- [x] Constraint solver (style resolution, BSP floor plan, structure generation)
- [x] Gable roof with exact-vertex quads
- [x] wgpu renderer with orbit camera
- [x] Headless screenshot export (4 angles + wireframe)
- [x] 3 view modes (Solid, Solid+Wireframe, Wireframe Only)
- [x] .orb file format (full schema, read/write, spatial index, occupancy, clash tables)
- [x] Spatial integrity persistence layer (occupancy, clearance envelopes, clash results)
- [x] Process auto-kill on startup
- [x] Sequential screenshot archive
