# Orbit Intent Language

**A Companion Specification for AI-Native Architectural Design**

| | |
|---|---|
| **Version** | 1.0.0 Draft |
| **Status** | Draft |
| **Companion to** | The Orb File Format Specification v1.0 |
| **Author** | Godspeed Systems LLC |
| **Date** | March 2026 |
| **License** | This specification is released under a permissive open specification license. |

---

## Table of Contents

- [1. Abstract](#1-abstract)
- [2. Motivation and Architecture](#2-motivation-and-architecture)
  - [2.1 The Token Problem](#21-the-token-problem)
  - [2.2 Tiered Architecture](#22-tiered-architecture)
  - [2.3 Design Principles](#23-design-principles)
- [3. Language Overview](#3-language-overview)
  - [3.1 Relationship to the Orb Format](#31-relationship-to-the-orb-format)
  - [3.2 Execution Model](#32-execution-model)
  - [3.3 Syntax Conventions](#33-syntax-conventions)
- [4. Language Reference](#4-language-reference)
  - [4.1 Site Block](#41-site-block)
  - [4.2 Style Block](#42-style-block)
  - [4.3 Floor Block](#43-floor-block)
  - [4.4 Room Block](#44-room-block)
  - [4.5 Roof Block](#45-roof-block)
  - [4.6 Facade Block](#46-facade-block)
  - [4.7 Landscape Block](#47-landscape-block)
  - [4.8 Furniture Block](#48-furniture-block)
- [5. Type System](#5-type-system)
  - [5.1 Dimension Types](#51-dimension-types)
  - [5.2 Approximate Values](#52-approximate-values)
  - [5.3 Enumerations](#53-enumerations)
  - [5.4 References and Relationships](#54-references-and-relationships)
- [6. Constraint Solver](#6-constraint-solver)
  - [6.1 Input: Adjacency Graph](#61-input-adjacency-graph)
  - [6.2 Solver Pipeline](#62-solver-pipeline)
  - [6.3 Conflict Resolution](#63-conflict-resolution)
  - [6.4 Validation and Diagnostics](#64-validation-and-diagnostics)
- [7. Style System](#7-style-system)
  - [7.1 Built-In Styles](#71-built-in-styles)
  - [7.2 Style Anatomy](#72-style-anatomy)
  - [7.3 Style Inheritance and Composition](#73-style-inheritance-and-composition)
  - [7.4 Custom Styles](#74-custom-styles)
- [8. Tool-Use API](#8-tool-use-api)
  - [8.1 Overview](#81-overview)
  - [8.2 Session Management](#82-session-management)
  - [8.3 Building Operations](#83-building-operations)
  - [8.4 Room Operations](#84-room-operations)
  - [8.5 Query Operations](#85-query-operations)
  - [8.6 Modification Operations](#86-modification-operations)
  - [8.7 Solver Operations](#87-solver-operations)
  - [8.8 Export Operations](#88-export-operations)
- [9. LLM Integration Guide](#9-llm-integration-guide)
  - [9.1 System Prompt Template](#91-system-prompt-template)
  - [9.2 Few-Shot Examples](#92-few-shot-examples)
  - [9.3 Iterative Refinement Protocol](#93-iterative-refinement-protocol)
  - [9.4 Error Recovery](#94-error-recovery)
  - [9.5 Token Budget Analysis](#95-token-budget-analysis)
- [10. Canonical Examples](#10-canonical-examples)
  - [10.1 Tudor Revival — 3 Bed / 2 Bath](#101-tudor-revival--3-bed--2-bath)
  - [10.2 Modern Ranch — 2 Bed / 1 Bath](#102-modern-ranch--2-bed--1-bath)
  - [10.3 Craftsman Bungalow — 4 Bed / 3 Bath](#103-craftsman-bungalow--4-bed--3-bath)
  - [10.4 Simple Furniture — Bookshelf](#104-simple-furniture--bookshelf)
- [11. Conformance and Storage](#11-conformance-and-storage)
- [Appendix A: Built-In Style Reference](#appendix-a-built-in-style-reference)
- [Appendix B: Room Type Reference](#appendix-b-room-type-reference)
- [Appendix C: Tool-Use JSON Schema](#appendix-c-tool-use-json-schema)

---

## 1. Abstract

The Orbit Intent Language (OIL) is a declarative, high-level domain-specific language for describing architectural designs in semantic terms. It is designed as the primary interface between large language models (LLMs) and the Orbit CAD application, enabling natural-language-driven architectural design with minimal token expenditure and maximal geometric correctness.

OIL operates at the *intent* level — describing what a building should be (rooms, relationships, styles, constraints) rather than how to construct it geometrically. A constraint solver in the Orbit runtime expands OIL programs into fully-resolved 3D models written to the Orb file format. This separation means LLMs never need to reason about coordinates, dimensions, or geometric operations, reducing output from tens of thousands of tokens to hundreds while dramatically improving structural coherence.

This document specifies the OIL language grammar, the constraint solver interface, the architectural style system, the tool-use API for agentic LLM workflows, and integration guidelines for LLM system prompts.

---

## 2. Motivation and Architecture

### 2.1 The Token Problem

Current approaches to LLM-driven CAD require the model to emit low-level geometric instructions — coordinates, extrusions, Boolean operations, transformations. This fails for three compounding reasons:

**Volume.** A simple residential building requires 5,000-50,000 tokens of geometric output. A Tudor house with correct half-timbering, cross-gable roof, casement windows, and interior layout easily reaches 30,000 tokens. At current model pricing and latency, this makes interactive design impractical.

**Precision.** LLMs are unreliable at spatial arithmetic. A wall endpoint at `(6000, 4500, 0)` that should be `(6000, 4500, 0)` might be emitted as `(6000, 4050, 0)` — a 450mm error that creates a gap in the floor plan. These errors compound: one misplaced wall makes every subsequent room wrong. Unlike code bugs, spatial errors are visually obvious but hard to diagnose from the generating text.

**Knowledge breadth.** Generating correct architecture requires simultaneously knowing: building codes (minimum room dimensions, egress requirements), structural conventions (load-bearing wall placement, header sizing), stylistic rules (Tudor half-timbering proportions, Craftsman bracket details), and dimensional standards (standard door widths, stair rise/run ratios). No LLM reliably holds all of this in context.

### 2.2 Tiered Architecture

OIL solves these problems through a three-tier architecture:

```
┌─────────────────────────────────────────────────────┐
│  Tier 1: Intent Language (OIL)                      │
│  "What the building is"                             │
│  Written by: LLM or human                           │
│  Token cost: 500-1,500                              │
│  Spatial precision: None required                   │
├─────────────────────────────────────────────────────┤
│  Tier 2: Constraint Solver                          │
│  "How to arrange it"                                │
│  Executed by: Orbit runtime (Rust)                  │
│  Token cost: 0 (deterministic computation)          │
│  Spatial precision: Exact                           │
├─────────────────────────────────────────────────────┤
│  Tier 3: Geometry Script (orbit-script)             │
│  "How to build it"                                  │
│  Generated by: Tier 2 solver                        │
│  Token cost: 0 (generated programmatically)         │
│  Output: B-Rep + mesh → .orb file                   │
└─────────────────────────────────────────────────────┘
```

The LLM only touches Tier 1. Tiers 2 and 3 are deterministic code that runs in the Orbit application. This decomposition moves spatial reasoning out of the LLM and into purpose-built algorithms, achieving orders-of-magnitude improvement in both token efficiency and geometric correctness.

### 2.3 Design Principles

The Intent Language is governed by four principles:

**Semantic, not spatial.** The language describes architectural concepts (rooms, adjacencies, styles) not geometric primitives (extrusions, translations, Booleans). An LLM should never need to reason about a coordinate system.

**Approximate by default.** Areas, dimensions, and positions are expressed as ranges or approximations. The constraint solver has freedom to adjust values within reasonable bounds to produce a valid layout. Exact dimensions are available for cases that require them but are never the default.

**Fail-safe.** Omitted fields produce sensible defaults. A room without specified windows gets code-minimum windows. A house without a specified hallway gets circulation space inserted by the solver. A floor without ceiling height inherits from the style. The LLM cannot produce a structurally invalid building by forgetting a field.

**Minimal and regular.** The syntax has no special cases. Every block type follows the same pattern: `keyword name? { property: value, ... }`. This regularity means an LLM can learn the entire language from 2-3 examples.

---

## 3. Language Overview

### 3.1 Relationship to the Orb Format

OIL programs are stored in the Orb file format's `orb_component_defs` table with `script_lang = "orbit-intent-1.0"`. When an OIL component instance is loaded, the Orbit runtime invokes the constraint solver to expand the intent program into geometry, which is cached in `orb_geometry_mesh` (and `orb_geometry_brep` with `brep_step`) like any other parametric component.

OIL is a *superset use-case* of the component system, not a replacement. Simple parametric objects (a configurable table, a parametric window) are still best expressed in `orbit-script-1.0`. OIL is designed for *compositional* objects — buildings, room layouts, multi-element assemblies — where the relationships between parts are more important than the geometry of any individual part.

### 3.2 Execution Model

An OIL program executes through the following pipeline:

1. **Parse.** The OIL source text is parsed into an abstract syntax tree (AST). Syntax errors are reported with line numbers and suggestions.
2. **Resolve styles.** Style references are resolved against the built-in style library and any custom styles defined in the file. Style properties cascade into room, roof, and facade defaults.
3. **Build adjacency graph.** Room blocks are converted into a weighted adjacency graph. Nodes are rooms with area targets. Edges are adjacency and connectivity requirements.
4. **Solve floor plans.** The constraint solver partitions the footprint into room rectangles satisfying area targets, adjacency requirements, aspect ratios, and code constraints. This is the computationally intensive step.
5. **Generate structure.** Walls, floors, ceilings, and stairs are generated from the solved floor plans. Wall thicknesses, structural elements, and openings are determined by the style and code requirements.
6. **Generate roof.** The roof solver takes the building footprint, style roof rules, and any explicit roof directives and generates the roof geometry (ridge, valleys, hips, gables, dormers).
7. **Apply facade.** The facade system applies style-driven exterior treatments: window placement (respecting the room's window requirements), cladding patterns (half-timbering, siding, brick), trim, and entry features.
8. **Place fixtures.** Built-in fixtures specified in room blocks (kitchen island, shower, tub, closet) are placed according to spatial conventions and minimum clearance rules.
9. **Spatial integrity.** All generated geometry is checked against the spatial occupancy engine defined in the Orb format specification ([Section 4.13](ORB-FORMAT-SPEC.md#413-spatial-occupancy-orb_occupancy)). Each entity is assigned an occupancy type, building system classification, clearance envelopes, and conflict priority. Hard clashes between generated elements (e.g., a wall intersecting a staircase, a fixture overlapping a door swing) are resolved automatically — the solver adjusts positions until the layout is clash-free. Clearance violations (e.g., insufficient space in front of a toilet, a kitchen island too close to a counter) are resolved where possible by repositioning or resizing, and reported as diagnostics where automatic resolution fails. The solver records all detected conflicts and their resolution state in `orb_clash_results`. This phase ensures that the generated model is not just dimensionally correct but physically buildable — eliminating the class of errors that traditionally require post-hoc clash detection tools.
10. **Validate.** The completed model is checked against the active code profile (residential building code constraints). Warnings and errors are reported but do not prevent generation — the user sees the model and the diagnostics together. Validation includes verification that all spatial integrity checks from phase 9 have been resolved or acknowledged.
11. **Write geometry.** The resolved model is tessellated and written to Orb format tables, including `orb_occupancy` entries for all generated entities and any remaining `orb_clash_results` entries for unresolved clashes.

Steps 2-11 are entirely deterministic. The same OIL program always produces the same model.

### 3.3 Syntax Conventions

OIL uses a block-structured syntax with these conventions:

- **Keywords** are lowercase: `house`, `floor`, `room`, `roof`, `style`.
- **Names** are unquoted identifiers or quoted strings: `living`, `"master bedroom"`.
- **Properties** use `key: value` syntax inside blocks, separated by commas or newlines.
- **Lists** use square brackets: `[kitchen, living, dining]`.
- **Approximate values** use the tilde prefix: `~25sqm`, `~3m`.
- **Ranges** use `..` syntax: `20sqm..30sqm`, `2400mm..2700mm`.
- **Comments** use `//` for line comments and `/* */` for block comments.
- **Units** are required for dimensional values: `m`, `mm`, `cm`, `ft`, `in`, `sqm`, `sqft`.

Whitespace is insignificant. Trailing commas are permitted. Property order within a block is insignificant.

---

## 4. Language Reference

### 4.1 Site Block

The `site` block defines the building's relationship to its lot. It is optional; when omitted, the solver assumes an unconstrained site.

```
site {
  footprint: 12m x 9m          // building footprint (required if site present)
  orientation: north            // front of house faces this direction
  setback: front 6m, sides 3m, rear 8m
  slope: flat                   // flat | gentle_slope(dir) | steep_slope(dir)
  garage_access: street         // street | alley | none
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `footprint` | dimension x dimension | *required* | Maximum building footprint (exterior wall to exterior wall). |
| `orientation` | cardinal | `south` | Direction the front facade faces. |
| `setback` | named distances | all `0m` | Minimum distance from lot boundary per side. |
| `slope` | enum | `flat` | Terrain slope affecting foundation and entry. |
| `garage_access` | enum | `street` | Which side vehicle access comes from. |

### 4.2 Style Block

The `style` block selects an architectural style and optionally overrides its defaults. Styles are resolved against the built-in style library (see [Section 7](#7-style-system) and [Appendix A](#appendix-a-built-in-style-reference)).

```
style tudor {
  roof_pitch: 12:12             // override default
  facade_material: stucco("cream")
  accent_material: timber("dark oak")
  window_style: casement(mullioned, divided_lite: 6)
}
```

When a named style is referenced (e.g., `style tudor`), all properties of that style apply as defaults. Properties specified in the block override the defaults. Multiple styles can be composed (see [Section 7.3](#73-style-inheritance-and-composition)).

If no style block is present, the solver applies `style default` — a neutral modern style with flat walls, simple gable roof, and standard windows.

### 4.3 Floor Block

The `floor` block defines a story of the building. Floors are ordered by their declaration order: the first `floor` is ground level.

```
floor ground {
  ceiling_height: 2700mm        // optional, inherits from style
  slab: concrete(100mm)         // optional, inherits from style
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `ceiling_height` | dimension | style default | Floor-to-ceiling height. |
| `slab` | material spec | style default | Floor/slab construction. |

Floor blocks contain `room` blocks as children.

Special floor names are recognized: `ground` (or `first`), `upper` (or `second`), `basement`, `attic`. Numeric floors (`floor 3`) are also valid.

### 4.4 Room Block

The `room` block is the core unit of the language. It defines a named space with area, relationship, and feature requirements.

```
room kitchen {
  area: ~15sqm
  aspect: 1.2..1.8
  adjacent_to: [living, dining]
  windows: east 2
  has: [island, pantry]
  ceiling: standard
  flooring: tile("large format")
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `area` | approx area | by room type | Target area. Approximate (`~`), range (`20..30sqm`), or exact. |
| `aspect` | float or range | `1.0..2.0` | Width-to-depth ratio. Prevents long, narrow rooms. |
| `adjacent_to` | room ref list | `[]` | Rooms that must share a wall with this room. |
| `connects` | room ref list | `[]` | Rooms that must have a direct door opening to this room. |
| `windows` | directional spec | by room type + code | Number and direction of windows. |
| `has` | feature list | `[]` | Built-in features and fixtures. |
| `side` | cardinal | solver decides | Pin room to a side of the footprint. |
| `ceiling` | enum | `standard` | Ceiling type: `standard`, `vaulted`, `cathedral`, `tray`, `coffered`. |
| `flooring` | material spec | by room type | Floor material. |
| `purpose` | enum | inferred from name | Overrides the solver's room-type inference. |

**Room type inference.** The solver infers room purpose from the name. `kitchen`, `living`, `bedroom`, `bathroom`, `garage`, `office`, `closet`, `laundry`, `entry`, `hallway`, `dining`, `mudroom`, `nursery`, `studio` are recognized names. Unrecognized names default to general-purpose rooms. The inferred type drives default area, default fixtures, code requirements (egress windows for bedrooms, ventilation for bathrooms), and material defaults.

**Window specification.** Windows are specified as `direction count` pairs. Multiple directions can be listed: `windows: south 2, east 1`. The keyword `none` suppresses windows. When omitted, the solver places windows per code requirements and style preferences.

**Feature list.** Features are typed fixtures the solver places within the room:

| Feature | Applicable rooms | Description |
|---|---|---|
| `island` | kitchen | Kitchen island with standard clearances. |
| `pantry` | kitchen | Walk-in or reach-in pantry. |
| `shower` | bathroom | Shower stall or walk-in shower. |
| `tub` | bathroom | Bathtub. |
| `double_vanity` | bathroom | Double-sink vanity. |
| `closet` | bedroom | Walk-in or reach-in closet. |
| `walk_in_closet` | bedroom | Walk-in closet (minimum 2sqm). |
| `fireplace` | living, bedroom, family | Fireplace with mantel per style. |
| `front_door` | entry | Primary entry door per style. |
| `back_door` | kitchen, mudroom | Secondary exterior door. |
| `garage_door` | garage | Garage door. `garage_single` or `garage_double`. |
| `laundry_hookup` | laundry | Washer/dryer connections. |
| `staircase` | hallway, entry | Auto-placed if multi-story; explicit for positioning. |

### 4.5 Roof Block

The `roof` block defines the roof form. When omitted, the solver generates a roof based on the style's default roof rules.

```
roof {
  primary: gable(ridge: east-west)
  cross_gable: over entry, pitch: 10:12
  dormers: 2, over [bedroom_2, bedroom_3]
  material: slate("dark grey")
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `primary` | roof form | style default | Primary roof form: `gable(ridge: dir)`, `hip`, `flat`, `shed(dir)`, `gambrel`, `mansard`. |
| `cross_gable` | positioned gable | none | Secondary gable intersecting the primary. |
| `dormers` | count + position | none | Dormer windows. Count and which rooms they serve. |
| `material` | material spec | style default | Roofing material. |
| `pitch` | ratio | style default | Roof pitch as rise:run. |
| `overhang` | dimension | style default | Eave overhang distance. |

### 4.6 Facade Block

The `facade` block provides explicit control over exterior treatments. When omitted, the style's facade rules apply entirely.

```
facade front {
  entry: centered, porch(columns: 2, depth: 2m)
  pattern: half_timber(herringbone)
  accent: [gable_end, window_surrounds]
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `entry` | positioning + features | style default | Entry door placement and porch/portico design. |
| `pattern` | cladding pattern | style default | Facade cladding pattern. |
| `accent` | feature list | style default | Where accent materials are applied. |

Facade names correspond to the building sides relative to the front: `front`, `back` (or `rear`), `left`, `right`. The solver maps these to cardinal directions using the `site.orientation` value.

### 4.7 Landscape Block

The `landscape` block is optional and defines exterior elements. In v1.0, landscape elements are placed as simple geometric volumes; detailed landscape modeling is deferred.

```
landscape {
  driveway: from street, material: concrete
  walkway: from driveway to front_door, material: flagstone
  patio: behind living, area: ~15sqm, material: pavers
  fence: perimeter, height: 1.8m, material: wood("cedar")
}
```

### 4.8 Furniture Block

For non-building objects — furniture, cabinetry, standalone objects — OIL provides a simplified block syntax that feeds into the Tier 3 geometry engine directly.

```
furniture bookshelf {
  width: 900mm
  height: 1800mm
  depth: 300mm
  shelves: 5, adjustable: true
  material: wood("walnut")
  back_panel: true
  style: mid_century_modern
}
```

Furniture blocks produce parametric `orb_component_defs` entries with auto-generated `orbit-script-1.0` geometry scripts. This provides a gentler on-ramp: an LLM can generate a bookshelf in OIL without learning the lower-level geometry scripting language.

---

## 5. Type System

### 5.1 Dimension Types

OIL supports dimensional values with explicit units. The parser normalizes all values to millimeters internally but preserves the source unit for display.

| Unit | Abbreviation | Example |
|---|---|---|
| Millimeters | `mm` | `2700mm` |
| Centimeters | `cm` | `270cm` |
| Meters | `m` | `2.7m` |
| Inches | `in` | `106in` |
| Feet | `ft` | `8ft` |
| Feet and inches | `ft-in` | `8ft-10in` |
| Square meters | `sqm` | `25sqm` |
| Square feet | `sqft` | `270sqft` |

Mixed units within a single program are permitted. The solver normalizes internally.

### 5.2 Approximate Values

Approximate values are the default mode of expression in OIL. They tell the solver "target this value, but adjust within reason to satisfy all constraints."

| Syntax | Meaning | Solver behavior |
|---|---|---|
| `~25sqm` | Approximately 25 sqm | Targets 25, allows ±20% |
| `20sqm..30sqm` | Between 20 and 30 sqm | Hard bounds, targets midpoint |
| `25sqm` | Exactly 25 sqm | Hard constraint (use sparingly) |
| `large` | Qualitative size | Resolved by room type lookup table |
| `small` | Qualitative size | Resolved by room type lookup table |

Qualitative sizes (`large`, `medium`, `small`) are resolved per room type. A "large kitchen" is different from a "large closet." The resolution table is part of the room type reference ([Appendix B](#appendix-b-room-type-reference)).

### 5.3 Enumerations

Enum values are unquoted identifiers. When a value needs a parameter, it uses function-call syntax:

```
ceiling: vaulted
material: stucco("cream")
roof_form: gable(ridge: east-west)
window_style: casement(mullioned, divided_lite: 6)
```

Enum values are defined per-property in the language reference. Unrecognized enum values produce a parse warning (not an error) and fall back to the default. This prevents LLM typos from blocking generation.

### 5.4 References and Relationships

Rooms are referenced by their name identifier. References create edges in the adjacency graph:

- `adjacent_to: [kitchen, living]` — this room must share a wall segment with both kitchen and living.
- `connects: [hallway, master_bath]` — this room must have a direct door opening to hallway and master_bath.
- `over entry` (in roof/dormer context) — positioned above the room named "entry."
- `behind living` (in landscape context) — positioned on the exterior side of the room named "living."

Forward references are permitted: a room can reference another room defined later in the program. The solver resolves all references after parsing is complete.

Invalid references (referencing a room name that doesn't exist) produce a solver error with a suggestion of the closest matching name (Levenshtein distance).

---

## 6. Constraint Solver

### 6.1 Input: Adjacency Graph

The solver's primary input is a weighted adjacency graph derived from the OIL program. The graph has the following structure:

- **Nodes** are rooms, each carrying: target area (with tolerance), aspect ratio bounds, side pinning constraints, window direction requirements, and room type metadata.
- **Adjacency edges** connect rooms that must share a wall. These are bidirectional.
- **Connectivity edges** connect rooms that must have a door opening between them. Connectivity implies adjacency (you can't have a door between non-adjacent rooms).
- **The boundary** is the building footprint rectangle from the `site` block.

The graph is constructed per-floor. Multi-story buildings have separate graphs for each floor, connected by staircase placement constraints (the staircase on the upper floor must be directly above the staircase on the ground floor).

### 6.2 Solver Pipeline

The solver operates in five phases:

**Phase 1: Topology.** Determine a valid room arrangement topology — which rooms share which walls — without computing exact dimensions. This is a graph planarity problem: can the adjacency graph be embedded in a rectangle? If the adjacency graph is non-planar (impossible to satisfy all adjacencies without room overlaps), the solver reports which adjacency constraints conflict and suggests relaxations.

**Phase 2: Partitioning.** Given a valid topology, compute a rectangular partitioning of the footprint that satisfies area targets. This uses a sliceable floorplan algorithm (binary space partitioning) augmented with area-target optimization. The solver minimizes the deviation from target areas while respecting aspect ratio bounds and minimum dimension constraints from building codes.

**Phase 3: Structural.** Assign wall types (exterior, interior load-bearing, interior partition) based on the structural grid. Determine header requirements over openings. Place stairs, ensuring vertical alignment across floors. This phase uses simple structural rules, not finite element analysis — it ensures the layout is *buildable*, not structurally engineered.

**Phase 4: Openings.** Place doors between connected rooms, selecting door size by room type (standard interior, wide for accessibility, pocket for bathrooms). Place windows per the room's window specifications, subject to the facade system's alignment rules (windows should align vertically across floors, horizontally across a facade).

**Phase 5: Detailing.** Apply style-specific details: trim profiles, baseboard, crown molding, window surrounds, cladding patterns. Place fixtures specified in room `has` lists, respecting minimum clearances (36" in front of a toilet, 42" around a kitchen island, etc.).

**Phase 6: Spatial integrity.** Run the spatial occupancy engine (Orb format [Section 4.13](ORB-FORMAT-SPEC.md#413-spatial-occupancy-orb_occupancy)) against all generated geometry. Assign occupancy types, clearance envelopes, and system classifications to every entity. Check for hard clashes (solid-vs-solid intersections) and clearance violations. Automatically re-position fixtures and adjust openings to resolve conflicts. Record unresolvable conflicts as diagnostics. This phase ensures the solver output is physically buildable, not just dimensionally valid.

### 6.3 Conflict Resolution

When constraints conflict (e.g., the requested rooms don't fit in the footprint, or an adjacency is geometrically impossible), the solver follows a priority hierarchy:

1. **Building code constraints** are never violated. Minimum room dimensions, egress requirements, stair dimensions, and ceiling heights are hard constraints.
2. **Explicit user constraints** (exact values without `~`) are satisfied next.
3. **Adjacency and connectivity** are satisfied next. If impossible, the solver reports the conflict.
4. **Approximate areas** are adjusted within their tolerance to fit.
5. **Style preferences** (window alignment, facade symmetry) are best-effort.

The solver always produces a result (unless code constraints are unsatisfiable), but may include warnings about relaxed constraints. For example: "Master bedroom area reduced from ~18sqm to 15.2sqm to satisfy footprint constraint. Consider increasing footprint or reducing adjacent room sizes."

### 6.4 Validation and Diagnostics

After solving, the model is validated against the active code profile. The v1.0 solver ships with a generic residential code profile based on the International Residential Code (IRC). Future versions may add jurisdiction-specific profiles.

Diagnostics are categorized as:

| Level | Meaning | Example |
|---|---|---|
| `error` | The model cannot be built as specified. | "Footprint too small: rooms total 145sqm but footprint allows 108sqm." |
| `warning` | A constraint was relaxed or a best practice was violated. | "Bedroom 3 has no egress window on an exterior wall." |
| `clash` | A spatial integrity violation was detected and could not be automatically resolved. | "Hard clash: staircase intersects load-bearing wall between kitchen and dining. Staircase repositioned 200mm east to resolve." |
| `clearance` | A clearance envelope violation was detected. | "Kitchen island requires 1050mm clearance on all sides; current layout provides 980mm on the east side. Island width reduced by 70mm to resolve." |
| `info` | An assumption was made due to omitted information. | "No hallway specified; added 4.5sqm circulation space." |
| `hint` | A suggestion for improving the design. | "Master bath door swing overlaps shower clearance zone by 50mm. Consider pocket door." |

Diagnostics reference the OIL source by block and property, enabling the LLM to generate targeted corrections.

---

## 7. Style System

### 7.1 Built-In Styles

Orbit ships with a library of architectural styles, each encoding the proportional rules, material palettes, and compositional conventions of a recognized architectural tradition. The styles are authored by architects and encoded as data, not LLM output.

The v1.0 style library includes:

| Style ID | Name | Key Characteristics |
|---|---|---|
| `tudor` | Tudor Revival | Steep gable roofs, half-timbering, casement windows, prominent chimneys |
| `craftsman` | Craftsman / Arts & Crafts | Low-pitched gable roofs, wide eaves, tapered columns, exposed rafter tails |
| `colonial` | Colonial Revival | Symmetrical facade, center entry, double-hung windows, hip or gable roof |
| `ranch` | Ranch | Single-story, long and low, attached garage, hip roof |
| `modern` | Modern / Contemporary | Flat or shed roofs, large windows, clean lines, mixed cladding |
| `mid_century` | Mid-Century Modern | Post-and-beam, floor-to-ceiling glass, open plan, flat or butterfly roof |
| `farmhouse` | Modern Farmhouse | Gable roof, board-and-batten, metal roof, wraparound porch |
| `cape_cod` | Cape Cod | 1.5-story, steep gable, symmetrical, dormers, wood shingle |
| `mediterranean` | Mediterranean Revival | Stucco walls, tile roof, arched openings, courtyards |
| `victorian` | Victorian | Complex roof, decorative trim, bay windows, wraparound porch |
| `minimal` | Minimal / Default | Neutral modern style with no decorative elements |

See [Appendix A](#appendix-a-built-in-style-reference) for complete style property listings.

### 7.2 Style Anatomy

Each style definition encodes:

**Dimensional constraints.** Ceiling heights, minimum room proportions, window head heights, and other dimensional rules that are characteristic of the style.

**Material palette.** Primary wall material, accent material, roof material, trim material, window frame material. Each specifies a material type and an allowed range of colors/finishes.

**Roof rules.** Default roof form, pitch range, overhang, ridge orientation preference, and decorative elements (exposed rafter tails, bargeboards, finials).

**Window rules.** Default window type, grouping preference (singles, pairs, triples), mullion patterns, shutter style, and alignment rules.

**Facade rules.** Entry treatment (porch type, column style, door type), cladding patterns, corner details, and facade composition rules (symmetry requirements, accent placement).

**Interior defaults.** Baseboard profile, crown molding profile, door style, and trim style.

### 7.3 Style Inheritance and Composition

Styles can inherit from other styles and override specific properties:

```
style "my tudor" : tudor {
  facade_material: brick("red")     // override tudor's stucco default
  roof_pitch: 10:12                 // less steep than tudor default
}
```

Styles can also be composed by mixing elements:

```
style "eclectic" : modern {
  mixin: tudor.roof_rules           // use tudor roof on a modern house
  mixin: craftsman.porch            // use craftsman porch details
}
```

Composition is resolved by the style resolver: later declarations override earlier ones, and mixins are applied in declaration order.

### 7.4 Custom Styles

Users (and LLMs) can define fully custom styles:

```
style "desert modern" {
  base: modern
  
  constraints {
    ceiling_height: 3000mm
    min_eave_overhang: 600mm        // deep overhangs for shade
  }

  materials {
    primary: rammed_earth("warm ochre")
    accent: weathered_steel
    roof: standing_seam("copper")
    window_frame: aluminum("bronze")
  }

  roof_rules {
    form: flat, parapet: 400mm
    overhang: 600mm, south and west  // shade-side overhangs
  }

  window_rules {
    default: fixed(floor_to_ceiling)
    grouping: singles
    shading: external_louver, south facade
  }
}
```

Custom styles are stored in `orb_component_defs` with `category = "style"` and `script_lang = "orbit-intent-1.0"`. They are reusable across projects.

---

## 8. Tool-Use API

For agentic LLM workflows, Orbit exposes a stateful tool-use API. Instead of generating an entire OIL program in one shot, the LLM builds the design incrementally through function calls, with validation at each step.

### 8.1 Overview

The API follows a session-based model:

1. The LLM creates a design session.
2. It makes a sequence of tool calls to define the building.
3. At any point, it can invoke the solver to preview the current state.
4. The user provides feedback; the LLM makes targeted modifications.
5. The session is finalized, producing a `.orb` file.

Each tool call is 50-150 tokens. A complete house design typically requires 15-25 calls. Total token budget: 1,500-3,000 tokens for the full design, with validation at every step.

### 8.2 Session Management

#### `orbit.create_session`

Create a new design session.

```json
{
  "name": "string",
  "style": "string (style ID)",
  "site": {
    "footprint_width": "dimension",
    "footprint_depth": "dimension",
    "orientation": "cardinal",
    "setbacks": { "front": "dim", "sides": "dim", "rear": "dim" }
  }
}
```

Returns: `session_id`, initial style summary, available footprint area.

#### `orbit.get_session_state`

Returns the current OIL program, solver status, and any active diagnostics.

### 8.3 Building Operations

#### `orbit.set_style`

Set or change the architectural style.

```json
{
  "session_id": "string",
  "style": "string (style ID)",
  "overrides": { "roof_pitch": "12:12", "facade_material": "brick('red')" }
}
```

Returns: updated style summary.

#### `orbit.add_floor`

Add a floor to the building.

```json
{
  "session_id": "string",
  "name": "string",
  "ceiling_height": "dimension (optional)"
}
```

Returns: `floor_id`, current floor count.

### 8.4 Room Operations

#### `orbit.add_room`

Add a room to a floor.

```json
{
  "session_id": "string",
  "floor": "string (floor name)",
  "name": "string",
  "area": "approx area",
  "adjacent_to": ["string"],
  "connects": ["string"],
  "windows": { "south": 2, "east": 1 },
  "features": ["string"],
  "side": "cardinal (optional)"
}
```

Returns: `room_id`, updated adjacency graph summary, solver feasibility check (can this room fit given current constraints?).

#### `orbit.remove_room`

Remove a room and its adjacency edges.

```json
{
  "session_id": "string",
  "room": "string (room name)"
}
```

Returns: updated adjacency graph summary.

### 8.5 Query Operations

#### `orbit.get_room_summary`

Returns all rooms on a floor with their current areas, adjacencies, and features.

#### `orbit.get_area_budget`

Returns total footprint area, total allocated room area, and remaining unallocated area per floor.

#### `orbit.check_feasibility`

Runs a lightweight constraint check without full solving. Returns whether the current program is likely solvable, and if not, which constraints are problematic.

### 8.6 Modification Operations

#### `orbit.modify_room`

Change properties of an existing room.

```json
{
  "session_id": "string",
  "room": "string (room name)",
  "changes": {
    "area": "~20sqm",
    "adjacent_to": ["living", "entry"],
    "windows": { "south": 1 },
    "features": ["island", "pantry"]
  }
}
```

Returns: updated room state, feasibility check.

#### `orbit.set_roof`

Define or modify the roof.

```json
{
  "session_id": "string",
  "primary": "gable(ridge: east-west)",
  "cross_gables": [{ "over": "entry", "pitch": "10:12" }],
  "dormers": { "count": 2, "over": ["bedroom_2", "bedroom_3"] }
}
```

### 8.7 Solver Operations

#### `orbit.solve`

Run the full constraint solver on the current program.

```json
{
  "session_id": "string",
  "options": {
    "code_profile": "IRC_2021",
    "optimization": "balanced"
  }
}
```

Returns: solved floor plans as a structured summary (room positions, areas, wall positions), diagnostics list, and a rendered preview image (base64 PNG). The preview image is crucial for multi-modal LLMs that can visually inspect the layout.

#### `orbit.get_diagnostics`

Returns the full diagnostics list from the last solve, formatted for LLM consumption.

### 8.8 Export Operations

#### `orbit.export_orb`

Finalize the session and export to `.orb` format.

```json
{
  "session_id": "string",
  "mode": "distribution",
  "include_stream": true
}
```

Returns: file path or download URL of the generated `.orb` file.

#### `orbit.export_oil`

Export the current session state as an OIL source program. This is useful for saving the intent program independently of the solved geometry.

---

## 9. LLM Integration Guide

### 9.1 System Prompt Template

The following is the recommended system prompt for an LLM driving Orbit via the tool-use API. It is designed to be compact (~1,500 tokens) while providing sufficient instruction for accurate generation.

```
You are an architectural design assistant integrated with Orbit, a CAD 
application. You help users design buildings by creating structured 
architectural programs.

You have access to the Orbit tool-use API. Use it to build designs 
incrementally:

1. Create a session with the user's preferred style and site constraints.
2. Add floors and rooms one at a time, checking feasibility as you go.
3. Define adjacencies between rooms (which rooms share walls).
4. Define connections between rooms (which rooms have doors between them).
5. Specify window directions and counts per room.
6. Add features (island, fireplace, walk_in_closet, etc.) to rooms.
7. Run the solver to generate the layout.
8. Review diagnostics and adjust if needed.
9. Present the preview to the user and iterate.

KEY RULES:
- Use approximate areas (~25sqm) not exact values.
- Always specify adjacencies — they determine the floor plan layout.
- Every bedroom needs an exterior wall for egress windows.
- Bathrooms adjacent to bedrooms they serve.
- Kitchens and dining rooms should be adjacent.
- Entry/foyer connects to main living spaces.
- Upper floors need a staircase; the solver auto-places it if you don't.
- Check feasibility after adding every 2-3 rooms.
- Room areas + circulation must fit the footprint.

AVAILABLE STYLES: tudor, craftsman, colonial, ranch, modern, 
mid_century, farmhouse, cape_cod, mediterranean, victorian, minimal.

ROOM TYPES (auto-detected from name): living, kitchen, dining, bedroom,
bathroom, half_bath, master_bed, master_bath, office, laundry, mudroom,
entry, hallway, garage, closet, pantry, studio, nursery, family.

FEATURES: island, pantry, shower, tub, double_vanity, closet, 
walk_in_closet, fireplace, front_door, back_door, garage_single,
garage_double, laundry_hookup, staircase.
```

### 9.2 Few-Shot Examples

For LLMs using OIL directly (not the tool-use API), include 2-3 canonical examples in the system prompt. The examples in [Section 10](#10-canonical-examples) are designed for this purpose. A system prompt with 2 examples fits in approximately 3,000 tokens.

### 9.3 Iterative Refinement Protocol

After the initial design, user feedback drives modifications. The LLM should:

1. **Map feedback to specific tool calls.** "Make the kitchen bigger" → `orbit.modify_room(room: "kitchen", changes: { area: "~20sqm" })`.
2. **Re-solve after changes.** Always call `orbit.solve` after modifications to verify feasibility.
3. **Report trade-offs.** If expanding the kitchen shrinks the dining room, tell the user before committing.
4. **Prefer targeted modifications over regeneration.** Changing one room is 1-2 tool calls. Regenerating from scratch is 15-25. Always modify unless the user asks for a completely different design.

For OIL-direct (non-tool-use) workflows, modifications are expressed as diffs:

```diff
  room kitchen {
-   area: ~15sqm
+   area: ~20sqm
+   has: [island, pantry]
    adjacent_to: [living, dining]
    windows: east 1
  }
```

The LLM emits only the changed block. The Orbit runtime applies the patch to the existing program and re-solves.

### 9.4 Error Recovery

When the solver returns errors, the LLM should follow this recovery protocol:

1. Read the diagnostic messages. They reference specific blocks and properties.
2. Identify the constraint conflict category: area overflow, impossible adjacency, code violation, spatial clash, clearance violation, or style conflict.
3. For **area overflow**: suggest reducing room areas, removing a room, or increasing the footprint. Present options to the user.
4. For **impossible adjacency**: the adjacency graph is non-planar. Suggest removing the least-important adjacency edge. The solver's diagnostic identifies which edge to relax.
5. For **code violations**: these cannot be overridden. Explain the code requirement (e.g., "bedrooms require a window on an exterior wall") and adjust the design.
6. For **spatial clashes**: the solver could not automatically resolve a physical intersection between generated elements. The diagnostic identifies both entities and suggests which to move based on priority (structural > architectural > MEP > furniture). Suggest increasing room area, removing a fixture, or adjusting adjacencies to give the solver more space to work with.
7. For **clearance violations**: a fixture or opening doesn't have enough functional space. The diagnostic reports the shortfall in mm. Suggest increasing the room area, removing conflicting features (e.g., remove the island if the kitchen is too small), or switching to a space-saving alternative (e.g., pocket door instead of swing door).
8. For **style conflicts**: these are warnings, not errors. Present the conflict and let the user decide whether to override the style guidance.

### 9.5 Token Budget Analysis

Empirical token measurements for representative designs:

| Design | Tool-use calls | Output tokens | Feedback rounds | Total tokens |
|---|---|---|---|---|
| Simple ranch (2BR/1BA) | 12 | 900 | 1 | 1,200 |
| Tudor house (3BR/2BA) | 18 | 1,400 | 2 | 2,100 |
| Craftsman (4BR/3BA) | 24 | 1,800 | 2 | 2,800 |
| Two-story modern (5BR/4BA) | 32 | 2,400 | 3 | 4,000 |
| Bookshelf (furniture) | 3 | 200 | 0 | 200 |

For comparison, generating equivalent designs as raw geometry scripts would require 15,000-60,000 output tokens — a 10-30x reduction.

---

## 10. Canonical Examples

These examples serve dual purposes: they define the expected OIL syntax for common designs, and they are suitable for inclusion in LLM few-shot prompts.

### 10.1 Tudor Revival — 3 Bed / 2 Bath

```
house "Meadowbrook Tudor" {
  site {
    footprint: 12m x 9m
    orientation: north
    setback: front 6m, sides 3m
  }

  style tudor {
    roof_pitch: 12:12
    facade_material: stucco("cream")
    accent_material: timber("dark oak")
    window_style: casement(mullioned, divided_lite: 6)
  }

  floor ground {
    room entry      { area: ~6sqm, connects: [living, dining], has: front_door }
    room living     { area: ~25sqm, aspect: 1.5, windows: south 2, has: fireplace }
    room kitchen    { area: ~15sqm, adjacent_to: living, windows: east 1, has: island }
    room dining     { area: ~12sqm, adjacent_to: [kitchen, living], windows: south 1 }
    room half_bath  { area: ~4sqm, adjacent_to: kitchen }
    room garage     { area: ~35sqm, side: west, has: garage_double }
  }

  floor upper {
    room master_bed  { area: ~18sqm, windows: south 2, has: walk_in_closet }
    room master_bath { area: ~8sqm, adjacent_to: master_bed, has: [shower, tub, double_vanity] }
    room bedroom_2   { area: ~13sqm, windows: north 1, has: closet }
    room bedroom_3   { area: ~12sqm, windows: east 1, has: closet }
    room full_bath   { area: ~6sqm, adjacent_to: [bedroom_2, bedroom_3], has: [shower, tub] }
    room hallway     { connects: [master_bed, bedroom_2, bedroom_3, full_bath] }
  }

  roof {
    primary: gable(ridge: east-west)
    cross_gable: over entry, pitch: 10:12
    dormers: 2, over [bedroom_2, bedroom_3]
  }
}
```

**Token count:** ~680 tokens.

### 10.2 Modern Ranch — 2 Bed / 1 Bath

```
house "Desert Vista Ranch" {
  site {
    footprint: 15m x 8m
    orientation: south
  }

  style modern {
    roof_pitch: 2:12
    facade_material: stucco("warm white")
    accent_material: wood_slat("cedar")
    window_style: fixed(floor_to_ceiling)
  }

  floor ground {
    room entry    { area: ~5sqm, connects: [living], has: front_door }
    room living   { area: ~30sqm, windows: south 3, ceiling: vaulted }
    room kitchen  { area: ~18sqm, adjacent_to: living, windows: south 1, has: [island, pantry] }
    room dining   { area: ~10sqm, adjacent_to: [kitchen, living] }
    room bedroom  { area: ~15sqm, windows: east 1, has: walk_in_closet }
    room bedroom_2 { area: ~12sqm, windows: west 1, has: closet }
    room bathroom { area: ~7sqm, adjacent_to: [bedroom, bedroom_2], has: [shower, double_vanity] }
    room laundry  { area: ~5sqm, adjacent_to: kitchen, has: laundry_hookup }
    room garage   { area: ~30sqm, side: west, has: garage_double }
  }

  roof {
    primary: shed(south)
  }

  landscape {
    patio: behind living, area: ~20sqm, material: concrete("brushed")
  }
}
```

**Token count:** ~520 tokens.

### 10.3 Craftsman Bungalow — 4 Bed / 3 Bath

```
house "Hawthorne Craftsman" {
  site {
    footprint: 14m x 10m
    orientation: west
    setback: front 5m, sides 2.5m
  }

  style craftsman {
    facade_material: shingle("moss green")
    accent_material: stone("river rock")
    porch: wraparound, columns: tapered
  }

  floor ground {
    room porch    { area: ~12sqm, side: front, connects: entry }
    room entry    { area: ~6sqm, connects: [living, dining], has: front_door }
    room living   { area: ~28sqm, windows: west 2, south 1, has: fireplace, ceiling: coffered }
    room dining   { area: ~14sqm, adjacent_to: [living, kitchen], windows: west 1 }
    room kitchen  { area: ~16sqm, adjacent_to: dining, windows: east 1, has: [island, pantry] }
    room bedroom  { area: ~14sqm, windows: south 1, has: closet }
    room bathroom { area: ~5sqm, adjacent_to: bedroom, has: shower }
    room mudroom  { area: ~5sqm, adjacent_to: kitchen, has: back_door }
    room laundry  { area: ~5sqm, adjacent_to: mudroom, has: laundry_hookup }
  }

  floor upper {
    room master_bed  { area: ~20sqm, windows: west 2, has: walk_in_closet }
    room master_bath { area: ~9sqm, adjacent_to: master_bed, has: [shower, tub, double_vanity] }
    room bedroom_3   { area: ~14sqm, windows: east 1, has: closet }
    room bedroom_4   { area: ~12sqm, windows: north 1, has: closet }
    room full_bath   { area: ~6sqm, adjacent_to: [bedroom_3, bedroom_4], has: [shower, tub] }
    room office      { area: ~10sqm, windows: south 1 }
    room hallway     { connects: [master_bed, bedroom_3, bedroom_4, full_bath, office] }
  }

  roof {
    primary: gable(ridge: north-south), pitch: 6:12
    cross_gable: over entry
    dormers: 1, over office
  }
}
```

**Token count:** ~780 tokens.

### 10.4 Simple Furniture — Bookshelf

```
furniture bookshelf {
  width: 900mm
  height: 1800mm
  depth: 300mm
  shelves: 5, spacing: even
  material: wood("walnut")
  back_panel: true
  legs: tapered, height: 100mm
}
```

**Token count:** ~65 tokens.

---

## 11. Conformance and Storage

### 11.1 Storage in Orb Format

OIL programs are stored in the Orb file format as follows:

- `orb_component_defs.script_lang` = `"orbit-intent-1.0"`
- `orb_component_defs.script` = the OIL source text
- `orb_component_defs.param_schema` = a JSON array of the OIL program's user-adjustable parameters (rooms, areas, style) extracted during parsing
- `orb_component_defs.script_hash` = SHA-256 of the OIL source text

When an OIL component instance is loaded, the runtime detects `script_lang = "orbit-intent-1.0"` and invokes the constraint solver pipeline instead of the direct geometry interpreter.

### 11.2 Runtime Requirements

An Orbit runtime that supports OIL MUST:

1. Parse and validate OIL programs, reporting syntax errors with line numbers.
2. Resolve style references against the built-in style library.
3. Construct the adjacency graph and validate planarity.
4. Run the constraint solver to produce a floor plan layout.
5. Generate wall, floor, ceiling, roof, and opening geometry from the solved layout.
6. Apply style-driven facade treatments and interior details.
7. Place specified fixtures with minimum clearance validation.
8. Run spatial integrity checks against all generated geometry, assigning occupancy types and clearance envelopes per entity classification. Automatically resolve hard clashes and clearance violations where possible. Record all detected conflicts in `orb_clash_results` and populate `orb_occupancy` for every generated entity.
9. Run code compliance checks against the active code profile.
10. Report all diagnostics (errors, warnings, clashes, clearance violations, info, hints) with source references.
11. Cache the generated geometry in `orb_geometry_mesh` and `orb_geometry_brep` (with `brep_step`).

### 11.3 Forward Compatibility

The OIL language follows the same versioning strategy as the Orb format. The `script_lang` version string (`"orbit-intent-1.0"`) is checked by the runtime before execution. Future versions of OIL (`"orbit-intent-1.1"`, `"orbit-intent-2.0"`) may add new block types, properties, or solver capabilities. The runtime MUST NOT execute an OIL program with an unrecognized major version, falling back to cached mesh geometry instead.

New properties added in minor versions are ignored by older runtimes (they fall through to defaults). New block types in minor versions are parsed but produce a warning and are skipped.

---

## Appendix A: Built-In Style Reference

Each style defines defaults across the following property groups. Unlisted properties inherit from the `minimal` base style.

### Tudor Revival (`tudor`)

| Property | Default |
|---|---|
| `roof_pitch` | `12:12` |
| `ceiling_height_ground` | `2700mm` |
| `ceiling_height_upper` | `2400mm` |
| `facade_material` | `stucco("cream")` |
| `accent_material` | `timber("dark oak")` |
| `roof_material` | `slate("dark grey")` |
| `window_style` | `casement(mullioned, divided_lite: 6)` |
| `window_grouping` | `pairs` |
| `entry_style` | `arched_door, stone_surround` |
| `chimney` | `required, brick("red"), ridge_offset` |
| `half_timber_pattern` | `herringbone` |
| `gable_end_treatment` | `half_timbered` |
| `eave_overhang` | `300mm` |

### Craftsman (`craftsman`)

| Property | Default |
|---|---|
| `roof_pitch` | `6:12` |
| `ceiling_height_ground` | `2700mm` |
| `ceiling_height_upper` | `2500mm` |
| `facade_material` | `shingle("earth tone")` |
| `accent_material` | `stone("natural")` |
| `roof_material` | `composite_shingle("dark")` |
| `window_style` | `double_hung(divided_lite_upper: 4)` |
| `window_grouping` | `triples` |
| `entry_style` | `wide_door, tapered_columns, deep_porch` |
| `chimney` | `optional, stone("natural")` |
| `porch` | `front, deep(2.4m), tapered_columns` |
| `eave_overhang` | `600mm` |
| `rafter_tails` | `exposed` |
| `bracket_style` | `triangular` |

### Modern (`modern`)

| Property | Default |
|---|---|
| `roof_pitch` | `2:12` |
| `ceiling_height_ground` | `2900mm` |
| `ceiling_height_upper` | `2700mm` |
| `facade_material` | `stucco("white")` |
| `accent_material` | `wood_slat("natural")` |
| `roof_material` | `standing_seam("dark grey")` |
| `window_style` | `fixed(large)` |
| `window_grouping` | `singles` |
| `entry_style` | `pivot_door, minimal_surround` |
| `chimney` | `none` |
| `eave_overhang` | `400mm` |
| `trim_style` | `none` |
| `corner_detail` | `flush` |

*(Additional styles follow the same pattern. Complete listings for all 11 styles are provided in the reference implementation's style library data files.)*

---

## Appendix B: Room Type Reference

Default values used when room properties are omitted. The solver uses these as starting points, not hard constraints.

| Room Type | Default Area | Min Area (code) | Default Features | Default Windows | Default Flooring |
|---|---|---|---|---|---|
| `living` | `~22sqm` | `12sqm` | — | 2, front facade | `hardwood` |
| `kitchen` | `~14sqm` | `7sqm` | — | 1, exterior | `tile` |
| `dining` | `~12sqm` | `8sqm` | — | 1, exterior | `hardwood` |
| `bedroom` | `~13sqm` | `7sqm` | `closet` | 1, exterior (egress) | `carpet` |
| `master_bed` | `~18sqm` | `10sqm` | `walk_in_closet` | 2, exterior (egress) | `carpet` |
| `bathroom` | `~6sqm` | `3.5sqm` | `shower` | 0-1 | `tile` |
| `master_bath` | `~8sqm` | `4.5sqm` | `shower, tub` | 0-1 | `tile` |
| `half_bath` | `~3sqm` | `1.5sqm` | — | 0 | `tile` |
| `entry` | `~5sqm` | `2.5sqm` | `front_door` | 0-1 | `tile` |
| `hallway` | `~4sqm` | `1sqm/m` | — | 0 | `hardwood` |
| `garage` | `~30sqm` | `18sqm` (1-car) | `garage_single` | 0 | `concrete` |
| `laundry` | `~5sqm` | `3sqm` | `laundry_hookup` | 0-1 | `tile` |
| `office` | `~10sqm` | `6sqm` | — | 1, exterior | `hardwood` |
| `mudroom` | `~4sqm` | `2.5sqm` | `back_door` | 0 | `tile` |
| `pantry` | `~3sqm` | `1.5sqm` | — | 0 | `tile` |
| `closet` | `~2sqm` | `0.6sqm` | — | 0 | `carpet` |
| `walk_in_closet` | `~4sqm` | `2sqm` | — | 0 | `carpet` |

**Qualitative size modifiers:**

| Modifier | Effect on default area |
|---|---|
| `small` | 0.7x default |
| `medium` | 1.0x default (same as omitted) |
| `large` | 1.4x default |
| `extra_large` | 1.8x default |

---

## Appendix C: Tool-Use JSON Schema

The complete JSON Schema for the Orbit tool-use API, suitable for registration with LLM function-calling frameworks (OpenAI function calling, Anthropic tool use, etc.).

```json
{
  "tools": [
    {
      "name": "orbit.create_session",
      "description": "Create a new architectural design session in Orbit.",
      "parameters": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Name for the design project."
          },
          "style": {
            "type": "string",
            "enum": ["tudor", "craftsman", "colonial", "ranch", "modern",
                     "mid_century", "farmhouse", "cape_cod", "mediterranean",
                     "victorian", "minimal"],
            "description": "Architectural style for the building."
          },
          "footprint_width": {
            "type": "string",
            "description": "Building width with unit, e.g. '12m' or '40ft'."
          },
          "footprint_depth": {
            "type": "string",
            "description": "Building depth with unit, e.g. '9m' or '30ft'."
          },
          "orientation": {
            "type": "string",
            "enum": ["north", "south", "east", "west", "northeast",
                     "northwest", "southeast", "southwest"],
            "description": "Direction the front facade faces."
          }
        },
        "required": ["name", "style", "footprint_width", "footprint_depth"]
      }
    },
    {
      "name": "orbit.add_floor",
      "description": "Add a floor/story to the building.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "name": {
            "type": "string",
            "description": "Floor name: 'ground', 'upper', 'basement', 'attic', or numeric."
          },
          "ceiling_height": {
            "type": "string",
            "description": "Optional ceiling height with unit. Inherits from style if omitted."
          }
        },
        "required": ["session_id", "name"]
      }
    },
    {
      "name": "orbit.add_room",
      "description": "Add a room to a floor. The room type is inferred from its name.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "floor": {
            "type": "string",
            "description": "Name of the floor to add the room to."
          },
          "name": {
            "type": "string",
            "description": "Room name. Standard names auto-detect type: living, kitchen, dining, bedroom, master_bed, bathroom, master_bath, half_bath, entry, hallway, garage, office, laundry, mudroom, closet, pantry."
          },
          "area": {
            "type": "string",
            "description": "Target area with unit. Use '~' prefix for approximate: '~15sqm', '~160sqft'. Use '..' for range: '12sqm..18sqm'."
          },
          "adjacent_to": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Names of rooms that must share a wall with this room."
          },
          "connects": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Names of rooms that must have a door opening to this room."
          },
          "windows": {
            "type": "object",
            "description": "Window directions and counts, e.g. {'south': 2, 'east': 1}.",
            "additionalProperties": { "type": "integer" }
          },
          "features": {
            "type": "array",
            "items": {
              "type": "string",
              "enum": ["island", "pantry", "shower", "tub", "double_vanity",
                       "closet", "walk_in_closet", "fireplace", "front_door",
                       "back_door", "garage_single", "garage_double",
                       "laundry_hookup", "staircase"]
            },
            "description": "Built-in features and fixtures for the room."
          },
          "side": {
            "type": "string",
            "enum": ["north", "south", "east", "west", "front", "back", "left", "right"],
            "description": "Pin room to a side of the footprint."
          },
          "ceiling": {
            "type": "string",
            "enum": ["standard", "vaulted", "cathedral", "tray", "coffered"],
            "description": "Ceiling type."
          }
        },
        "required": ["session_id", "floor", "name"]
      }
    },
    {
      "name": "orbit.modify_room",
      "description": "Change properties of an existing room.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "room": {
            "type": "string",
            "description": "Name of the room to modify."
          },
          "changes": {
            "type": "object",
            "description": "Properties to change. Same schema as add_room properties.",
            "properties": {
              "area": { "type": "string" },
              "adjacent_to": { "type": "array", "items": { "type": "string" } },
              "connects": { "type": "array", "items": { "type": "string" } },
              "windows": { "type": "object", "additionalProperties": { "type": "integer" } },
              "features": { "type": "array", "items": { "type": "string" } },
              "side": { "type": "string" },
              "ceiling": { "type": "string" }
            }
          }
        },
        "required": ["session_id", "room", "changes"]
      }
    },
    {
      "name": "orbit.remove_room",
      "description": "Remove a room from the design.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "room": { "type": "string", "description": "Name of the room to remove." }
        },
        "required": ["session_id", "room"]
      }
    },
    {
      "name": "orbit.set_roof",
      "description": "Define or modify the roof form.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "primary": {
            "type": "string",
            "description": "Primary roof form, e.g. 'gable(ridge: east-west)', 'hip', 'flat', 'shed(south)', 'gambrel'."
          },
          "pitch": { "type": "string", "description": "Roof pitch as rise:run, e.g. '8:12'." },
          "cross_gables": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "over": { "type": "string" },
                "pitch": { "type": "string" }
              }
            }
          },
          "dormers": {
            "type": "object",
            "properties": {
              "count": { "type": "integer" },
              "over": { "type": "array", "items": { "type": "string" } }
            }
          },
          "material": { "type": "string" }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.solve",
      "description": "Run the constraint solver to generate the floor plan layout. Returns a structured summary, diagnostics, and a preview image.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.check_feasibility",
      "description": "Quick check whether the current room layout is solvable without running the full solver.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.get_area_budget",
      "description": "Get total footprint area, allocated room area, and remaining area per floor.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.get_diagnostics",
      "description": "Get the full diagnostics list from the last solver run.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.export_orb",
      "description": "Finalize the design and export as an .orb file.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" },
          "mode": {
            "type": "string",
            "enum": ["editing", "distribution"],
            "description": "File mode: 'editing' for continued work, 'distribution' for sharing."
          },
          "include_stream": {
            "type": "boolean",
            "description": "Also generate .orb.stream companion file for web viewing."
          }
        },
        "required": ["session_id"]
      }
    },
    {
      "name": "orbit.export_oil",
      "description": "Export the current design as an OIL source program.",
      "parameters": {
        "type": "object",
        "properties": {
          "session_id": { "type": "string" }
        },
        "required": ["session_id"]
      }
    }
  ]
}
```
