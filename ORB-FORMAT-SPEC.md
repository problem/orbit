# The Orb File Format

**An Open Specification for Parametric CAD Data**

| | |
|---|---|
| **Version** | 1.0.0 Draft |
| **Status** | Draft |
| **Author** | Godspeed Systems LLC |
| **Date** | March 2026 |
| **License** | This specification is released under a permissive open specification license. Implementors are encouraged to build conforming readers and writers without restriction. |

---

## Table of Contents

- [1. Abstract](#1-abstract)
- [2. Introduction and Motivation](#2-introduction-and-motivation)
  - [2.1 Problem Statement](#21-problem-statement)
  - [2.2 Design Principles](#22-design-principles)
- [3. Format Overview](#3-format-overview)
  - [3.1 Container: SQLite 3](#31-container-sqlite-3)
  - [3.2 File Identification](#32-file-identification)
  - [3.3 Encoding Conventions](#33-encoding-conventions)
  - [3.4 Editing Session Model](#34-editing-session-model)
- [4. Schema Specification](#4-schema-specification)
  - [4.1 Document Metadata: orb_meta](#41-document-metadata-orb_meta)
  - [4.2 Scene Graph: orb_entities](#42-scene-graph-orb_entities)
  - [4.3 Layers: orb_layers](#43-layers-orb_layers)
  - [4.4 Spatial Index: orb_spatial_index](#44-spatial-index-orb_spatial_index)
  - [4.5 Geometry: Dual Representation](#45-geometry-dual-representation)
  - [4.6 Materials: orb_materials and orb_textures](#46-materials-orb_materials-and-orb_textures)
  - [4.7 Parametric Components](#47-parametric-components)
  - [4.8 Saved Views: orb_saved_views](#48-saved-views-orb_saved_views)
  - [4.9 Selection Sets: orb_selection_sets](#49-selection-sets-orb_selection_sets)
  - [4.10 Annotations](#410-annotations)
  - [4.11 BIM Classifications: orb_classifications](#411-bim-classifications-orb_classifications)
  - [4.12 External References: orb_external_refs](#412-external-references-orb_external_refs)
  - [4.13 Spatial Occupancy: orb_occupancy](#413-spatial-occupancy-orb_occupancy)
  - [4.14 Clash Detection: orb_clash_results](#414-clash-detection-orb_clash_results)
- [5. Versioning and Extensibility](#5-versioning-and-extensibility)
  - [5.1 Semantic Versioning](#51-semantic-versioning)
  - [5.2 Forward Compatibility Rules](#52-forward-compatibility-rules)
  - [5.3 Application-Specific Extensions](#53-application-specific-extensions)
- [6. Encoding Details](#6-encoding-details)
  - [6.1 UUIDv7 Generation](#61-uuidv7-generation)
  - [6.2 Transformation Matrices](#62-transformation-matrices)
  - [6.3 Mesh Vertex Packing](#63-mesh-vertex-packing)
  - [6.4 Face Material Assignment](#64-face-material-assignment)
- [7. Web Streaming Protocol](#7-web-streaming-protocol)
  - [7.1 Architecture](#71-architecture)
  - [7.2 Loading Sequence](#72-loading-sequence)
  - [7.3 HTTP Requirements](#73-http-requirements)
  - [7.4 Companion Stream Format](#74-companion-stream-format)
- [8. Import and Export](#8-import-and-export)
  - [8.1 SketchUp (.skp) Import](#81-sketchup-skp-import)
  - [8.2 IFC Export](#82-ifc-export)
  - [8.3 Other Formats](#83-other-formats)
- [9. Performance Considerations](#9-performance-considerations)
  - [9.1 SQLite Configuration](#91-sqlite-configuration)
  - [9.2 Geometry Budget Guidelines](#92-geometry-budget-guidelines)
  - [9.3 Indexing Strategy](#93-indexing-strategy)
- [10. Conformance Requirements](#10-conformance-requirements)
  - [10.1 Conformance Level 1: Minimal Reader](#101-conformance-level-1-minimal-reader)
  - [10.2 Conformance Level 2: Component-Aware Reader](#102-conformance-level-2-component-aware-reader)
  - [10.3 Conformance Level 3: Full Reader/Writer](#103-conformance-level-3-full-readerwriter)
- [11. Security Considerations](#11-security-considerations)
  - [11.1 Script Sandboxing](#111-script-sandboxing)
  - [11.2 Resource Limits](#112-resource-limits)
  - [11.3 BLOB Validation](#113-blob-validation)
- [12. Future Directions](#12-future-directions)
- [Appendix A: Complete Schema Reference](#appendix-a-complete-schema-reference)
- [Appendix B: MIME Type and File Association](#appendix-b-mime-type-and-file-association)
- [Appendix C: Glossary](#appendix-c-glossary)

---

## 1. Abstract

The Orb file format (`.orb`) is an open specification for storing parametric computer-aided design (CAD) data. Designed as the native format for the Orbit CAD application, it addresses fundamental limitations in existing CAD file formats: proprietary encoding that prevents interoperability, monolithic structures that inhibit partial reads, and the absence of web-native streaming capabilities.

Orb uses SQLite as its container format, providing atomic transactions, partial random access, proven long-term archival stability, and tooling compatibility. The format stores geometry in a dual representation — boundary representation (B-Rep) for parametric editing and tessellated mesh for rendering — enabling lightweight viewers to display models without requiring a full geometry kernel. B-Rep data is stored in both a kernel-native fast format and a portable STEP AP214 encoding to ensure long-term readability independent of any specific geometry kernel. The specification supports progressive complexity: simple direct-modeled geometry, parametric component definitions with scripted behavior, and full Building Information Modeling (BIM) metadata with industry-standard classifications.

This document defines the complete Orb v1.0 format specification, including the SQLite schema, data encoding conventions, the editing session model, spatial indexing, extensibility mechanisms, the web streaming protocol with its companion stream format, and conformance requirements for readers and writers.

---

## 2. Introduction and Motivation

### 2.1 Problem Statement

The CAD industry suffers from a fragmentation of file formats that impedes collaboration, tool choice, and long-term data preservation. SketchUp's `.skp` format is proprietary and requires Trimble's C SDK to read. Autodesk's `.rvt` and `.dwg` formats are closed and version-locked. Even the open IFC standard, while valuable for exchange, is not suitable as a working document format due to its verbose structure and lack of editing-centric features like undo history and parametric definitions.

No existing format satisfies the following requirements simultaneously:

- Open and publicly specified, allowing any tool to read and write without licensing restrictions
- Structured for partial, random-access reads suitable for streaming over HTTP
- Capable of storing both precise analytical geometry (B-Rep) and display-ready mesh data, with portable B-Rep encoding for archival independence
- Extensible to carry parametric scripting definitions, BIM classifications, and application-specific metadata
- Inspectable with commodity tools (a database browser, not a hex editor)
- Usable as a live working document during editing sessions with crash recovery, not just a serialization snapshot

### 2.2 Design Principles

The Orb format is governed by five design principles that inform every schema and encoding decision.

**Openness.** The specification is public. Any individual or organization may build a conforming reader or writer without payment, registration, or license agreement. The format must be fully documentable such that a competent engineer could implement a reader from the specification alone, without reference to any particular implementation's source code. Critically, openness extends to geometric data: B-Rep geometry must be available in a kernel-independent portable encoding (STEP AP214) so that no reader is locked to a specific geometry kernel.

**Inspectability.** An Orb file should be explorable with general-purpose tools. Because the container is SQLite, any SQLite browser (DB Browser for SQLite, the `sqlite3` CLI, or programmatic bindings in any language) can open an `.orb` file and query its contents. This dramatically lowers the barrier for third-party tool developers, data migration scripts, and forensic analysis of corrupted files.

**Progressive Complexity.** The format must serve hobbyists modeling a bookshelf and architects documenting a hospital. Simple models should produce simple files. BIM metadata, parametric scripting, and advanced geometry are optional layers that do not impose overhead on basic use cases. A conforming minimal reader need only understand the scene graph and mesh tables.

**Streamability.** Large models must be viewable over the web without downloading the entire file. The format's structure must support HTTP range-request access patterns, progressive loading of geometry data, and prioritized fetching of visible content. For production deployments, an optional companion stream format provides optimized sequential-read access.

**Longevity.** Architectural models may be referenced for decades. The format must be forward-compatible (new features do not break old readers) and archival-grade. SQLite is recommended by the Library of Congress as a sustainable digital preservation format. The Orb format inherits this durability. B-Rep geometry is preserved in the ISO-standardized STEP format alongside kernel-native encodings, ensuring that exact analytical geometry remains accessible regardless of which geometry kernels exist in the future.

---

## 3. Format Overview

### 3.1 Container: SQLite 3

An Orb file is a SQLite 3 database with the file extension `.orb`. The SQLite file format is publicly documented, stable across versions, backward-compatible to its inception, and supported by bindings in virtually every programming language. The choice of SQLite over alternatives (ZIP archives, FlatBuffers, Protobuf, custom binary formats) is deliberate and based on specific technical advantages:

- **Atomic transactions.** SQLite's write-ahead logging (WAL) mode ensures that a crash or power loss during a save operation never leaves the file in a corrupt state. For CAD files representing months of design work, this is a critical safety property.
- **Partial random access.** SQLite reads data at page granularity (default 4096 bytes). A query that touches only the scene graph table does not read geometry data from disk. This enables the web viewer to fetch only the metadata and mesh data it needs via HTTP range requests against the SQLite page structure.
- **Schema extensibility.** New tables and columns can be added in future format versions without breaking readers that do not recognize them. SQLite's `ALTER TABLE` and `CREATE TABLE IF NOT EXISTS` semantics provide a natural migration path.
- **Commodity tooling.** Developers can explore `.orb` files using the `sqlite3` command-line tool, DB Browser for SQLite, or any SQLite library, without any Orbit-specific software.

### 3.2 File Identification

Orb files are identified by three mechanisms:

- **File extension:** `.orb`
- **MIME type:** `application/vnd.orbit.orb`
- **Application ID:** The SQLite `application_id` pragma is set to the 32-bit integer `0x4F524231` (ASCII `"ORB1"`). This allows identification of Orb files even when the file extension is missing or incorrect.

### 3.3 Encoding Conventions

The following encoding conventions apply throughout the format:

- **Identifiers.** All entity, component, material, and texture identifiers are UUIDv7 values stored as 16-byte BLOBs. UUIDv7 is preferred over UUIDv4 because its time-ordered property enables efficient B-tree indexing in SQLite and provides natural creation-order sorting.
- **Text.** All text values are UTF-8 encoded.
- **Floating point.** All geometric coordinates and transformation values are IEEE 754 double-precision (f64) unless otherwise noted. Mesh vertex positions are single-precision (f32) for compactness.
- **Units.** All geometric values are stored in millimeters. The document's display unit preference is recorded in metadata but does not affect stored values. Millimeters are chosen because they provide sub-millimeter precision with f64 values across architectural scales (a 1 km site to a 0.1 mm detail) without floating-point degradation, and they align with the internal representation of most geometry kernels.
- **Transforms.** Affine transformation matrices are stored as 16 packed f64 values (128 bytes) in column-major order, consistent with OpenGL/WebGPU conventions and the `nalgebra` library's memory layout.
- **Packed arrays.** Vertex positions, normals, indices, and edge data are stored as tightly-packed binary BLOBs with no padding or alignment between elements. The element type and count are inferrable from the BLOB length and the column's documented element size.
- **JSON.** Structured metadata that does not benefit from relational querying (e.g., BIM property sets, parameter values) is stored as JSON text. This avoids schema explosion while remaining human-readable and queryable via SQLite's JSON functions (`json_extract`, `json_each`).

### 3.4 Editing Session Model

The Orb format defines two distinct file states, and conforming writers must understand the distinction:

**Live editing mode.** During an active editing session, the `.orb` file is the live working document. The application opens the file in SQLite WAL mode, performs incremental reads and writes as the user models, and relies on SQLite's atomic transaction guarantees for crash recovery. In this mode, two sidecar files (`<filename>.orb-wal` and `<filename>.orb-shm`) may exist alongside the `.orb` file. These are SQLite implementation details, not part of the Orb format, but users and file management tools should be aware that moving or copying a `.orb` file during an active editing session requires including the sidecar files to avoid data loss.

Implementations SHOULD perform an automatic checkpoint (`PRAGMA wal_checkpoint(PASSIVE)`) at regular intervals during editing to keep the WAL file from growing unbounded. The recommended interval is after every user-initiated save or every 60 seconds of idle time, whichever comes first.

**Distribution mode.** When a file is saved for sharing, archiving, or publishing, the writer MUST finalize the file by performing a WAL checkpoint, running `VACUUM` to defragment the database, and running `ANALYZE` to update query planner statistics. This produces a single, self-contained `.orb` file with no sidecar files, optimized page layout for the web streaming protocol, and consolidated query statistics. Distribution-mode files SHOULD have `journal_mode` set back to `DELETE` (the SQLite default) so that no WAL infrastructure is created when the file is opened read-only.

The `x-orbit-file-mode` key in `orb_meta` MAY be set to `"editing"` or `"distribution"` to indicate the file's current state. Readers MUST NOT require this key — its absence implies no assumption about the mode.

---

## 4. Schema Specification

This section defines every table in the Orb v1.0 schema. Tables are grouped by function: document metadata, scene graph, spatial indexing, geometry, materials, parametric definitions, views, selection sets, annotations, BIM classifications, external references, spatial occupancy, and clash detection. All core tables use the `orb_` prefix to avoid collisions with application-specific extension tables.

### 4.1 Document Metadata: orb_meta

The `orb_meta` table is a key-value store for document-level settings and provenance information. It is the first table a reader should query after verifying the `application_id` pragma.

```sql
CREATE TABLE orb_meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
```

The following keys are defined by the specification. Implementations may store additional keys prefixed with `x-` for application-specific metadata.

| Key | Type | Description |
|-----|------|-------------|
| `format_version` | semver | The Orb format version. v1.0 files use `"1.0.0"`. Readers MUST reject files with an unrecognized major version. |
| `created_by` | string | Application name and version that created the file (e.g., `"Orbit 1.2.0"`). |
| `created_at` | ISO 8601 | UTC timestamp of initial file creation. |
| `modified_at` | ISO 8601 | UTC timestamp of last modification. Updated on every save. |
| `display_unit` | enum | User's preferred display unit: `"mm"`, `"cm"`, `"m"`, `"in"`, `"ft"`. Does not affect stored values (always mm). |
| `up_axis` | enum | The world up axis: `"z"` (default, engineering convention) or `"y"` (SketchUp/game convention). |
| `geo_origin_lat` | float | Optional WGS84 latitude for geo-located models. |
| `geo_origin_lon` | float | Optional WGS84 longitude for geo-located models. |
| `geo_origin_alt` | float | Optional altitude in meters above WGS84 ellipsoid. |
| `import_source` | string | If converted from another format, the original filename and format (e.g., `"house.skp (SketchUp 2024)"`). |
| `description` | string | Optional human-readable description of the model. |
| `thumbnail` | string | Optional Base64-encoded PNG thumbnail image for file browsers (recommended 512x512). |

### 4.2 Scene Graph: orb_entities

The scene graph is a tree of entities representing every object in the model. Each entity has an optional parent, forming a hierarchy of groups, component instances, and geometric bodies. The tree's root entities (those with a `NULL` `parent_id`) are direct children of the scene.

```sql
CREATE TABLE orb_entities (
    id          BLOB PRIMARY KEY NOT NULL,  -- UUIDv7 (16 bytes)
    parent_id   BLOB REFERENCES orb_entities(id)
                     ON DELETE CASCADE,
    name        TEXT,
    entity_type TEXT NOT NULL,               -- see enum below
    transform   BLOB NOT NULL,               -- 4x4 f64 column-major (128 bytes)
    visible     INTEGER NOT NULL DEFAULT 1,
    locked      INTEGER NOT NULL DEFAULT 0,
    layer_id    BLOB REFERENCES orb_layers(id),
    source_unit TEXT,                         -- original unit before mm conversion
    created_at  TEXT NOT NULL,               -- ISO 8601
    modified_at TEXT NOT NULL                 -- ISO 8601
);

CREATE INDEX idx_entities_parent ON orb_entities(parent_id);
CREATE INDEX idx_entities_layer  ON orb_entities(layer_id);
CREATE INDEX idx_entities_type   ON orb_entities(entity_type);
```

Entity types are defined as follows:

| `entity_type` | Description |
|----------------|-------------|
| `body` | A solid geometric body with associated B-Rep and/or mesh data. |
| `group` | A named collection of child entities. Groups have no geometry of their own. |
| `component_instance` | An instance of a parametric component definition. References a row in `orb_component_defs` and carries instance-specific parameter overrides. |
| `section_plane` | A clipping/section plane entity used for generating 2D documentation views. |
| `annotation` | A dimension, label, or other 2D annotation entity. See [Section 4.10](#410-annotations). |
| `guide` | Construction geometry (guide lines, guide points) that is not part of the built model. |

The `transform` column stores the entity's local-to-parent transformation matrix. To compute an entity's world-space position, a reader must multiply the chain of transforms from the root to the entity. Implementations should cache world transforms and invalidate them when any ancestor's transform changes.

The `ON DELETE CASCADE` constraint ensures that deleting a parent entity automatically removes all descendants, maintaining tree integrity at the database level. Implementations SHOULD be aware that cascading deletes on deeply-nested hierarchies with tens of thousands of descendants can be slow in SQLite. For bulk deletion of large subtrees, implementations MAY collect descendant IDs first and delete in batches within a single transaction.

The optional `source_unit` column records the original unit system of the entity's geometry before conversion to millimeters. Valid values are `"mm"`, `"cm"`, `"m"`, `"in"`, `"ft"`. This is primarily useful for imported geometry: a wall imported from an Imperial-unit SketchUp file can record `source_unit = "in"` so that implementations can display measurements as exact Imperial values (e.g., "8 feet") rather than millimeter-converted approximations (e.g., "2438.4 mm"). When `source_unit` is `NULL`, no source unit information is available and the display unit from `orb_meta` applies.

### 4.3 Layers: orb_layers

Layers provide an organizational mechanism orthogonal to the scene graph hierarchy. An entity belongs to at most one layer. Layers control visibility and selectability in the user interface.

```sql
CREATE TABLE orb_layers (
    id         BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name       TEXT NOT NULL,
    color      TEXT,                        -- hex RGB (e.g., "FF6B35")
    visible    INTEGER NOT NULL DEFAULT 1,
    locked     INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);
```

### 4.4 Spatial Index: orb_spatial_index

Efficient spatial queries (viewport frustum culling, click-to-select hit testing, proximity searches) require a spatial index. The scene graph's hierarchical structure is not sufficient for these operations at architectural scale, where models may contain hundreds of thousands of entities.

Orb uses SQLite's R-tree extension to provide axis-aligned bounding box (AABB) queries. Because the R-tree virtual table requires integer rowid keys, a mapping table translates between UUIDv7 entity identifiers and integer row IDs:

```sql
CREATE TABLE orb_entity_rowids (
    rowid     INTEGER PRIMARY KEY,
    entity_id BLOB NOT NULL UNIQUE
              REFERENCES orb_entities(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE orb_spatial_index USING rtree(
    id,                -- integer, references orb_entity_rowids.rowid
    min_x, max_x,
    min_y, max_y,
    min_z, max_z
);
```

The bounding box values are in world-space millimeters (not local-space). Writers MUST update the spatial index whenever an entity's geometry or world-space transform changes. This includes propagating updates to all descendants when a parent's transform is modified.

The spatial index stores bounds for entities with geometry (`body` and `component_instance` types). Group entities MAY have spatial index entries representing the union of their children's bounds, but this is optional — implementations can compute group bounds from children on demand.

**Usage patterns:**

- **Frustum culling (web viewer).** Query `orb_spatial_index` for entities whose AABB intersects the camera frustum. Only fetch `orb_geometry_mesh` for matching entities. This is the primary mechanism for progressive loading prioritization described in [Section 7.2](#72-loading-sequence).
- **Click hit testing.** Query `orb_spatial_index` for entities whose AABB contains the click ray intersection point, then perform precise mesh-ray intersection only on candidates.
- **Proximity search.** Find all entities within a bounding region for area-select operations.
- **Spatial integrity enforcement.** Query `orb_spatial_index` as a broad-phase filter before every geometry-modifying operation to detect potential occupancy conflicts and clearance violations. This is the enabling mechanism for the spatial occupancy engine described in [Section 4.13](#413-spatial-occupancy-orb_occupancy). The spatial index is not merely a query optimization — it is the enforcement layer for spatial integrity, analogous to how SQLite's foreign key indexes enforce referential integrity.

Writers MUST populate the spatial index for all `body` and `component_instance` entities. Readers that do not need spatial queries (e.g., batch export tools) MAY ignore the spatial index tables entirely.

### 4.5 Geometry: Dual Representation

Orb stores geometry in two parallel representations. This dual-storage design is the format's most important architectural decision and warrants detailed explanation.

#### 4.5.1 Rationale for Dual Storage

Boundary representation (B-Rep) geometry is the authoritative, high-fidelity representation used by the editing kernel. It stores exact analytical surfaces (planes, cylinders, NURBS), topological relationships (vertices, edges, faces, shells), and enables precise Boolean operations, filleting, and measurement. However, B-Rep data requires a geometry kernel to interpret and cannot be rendered directly by a GPU.

Tessellated mesh geometry is a triangulated approximation of the B-Rep, suitable for direct GPU rendering. It consists of vertex positions, normals, and triangle indices — the universal language of graphics hardware. Any renderer (desktop, web, mobile) can display mesh data without any kernel dependency.

By storing both, Orb decouples the editing pipeline from the viewing pipeline. The full Orbit application reads and writes B-Rep data. Lightweight viewers (the web viewer, mobile viewers, file preview generators) read only mesh data. An Orb file with mesh data but no B-Rep is a valid view-only file. An Orb file with B-Rep data but no mesh is a valid file that requires tessellation before display.

#### 4.5.2 Mesh Geometry: orb_geometry_mesh

```sql
CREATE TABLE orb_geometry_mesh (
    entity_id      BLOB PRIMARY KEY NOT NULL
                   REFERENCES orb_entities(id) ON DELETE CASCADE,
    positions      BLOB NOT NULL,  -- packed f32 vec3 (12 bytes per vertex)
    normals        BLOB NOT NULL,  -- packed f32 vec3 (12 bytes per vertex)
    indices        BLOB NOT NULL,  -- packed u32 (4 bytes per index)
    edges          BLOB,           -- packed u32 pairs (8 bytes per edge)
    uv0            BLOB,           -- packed f32 vec2 (8 bytes per vertex)
    lod_level      INTEGER NOT NULL DEFAULT 0,
    face_materials BLOB            -- indexed material assignment per face
);
```

The `positions` BLOB contains tightly-packed IEEE 754 single-precision floating-point triplets (x, y, z). The vertex count is inferred as `byte_length(positions) / 12`. The `normals` BLOB must have the same vertex count. The `indices` BLOB contains unsigned 32-bit triangle indices; the triangle count is `byte_length(indices) / 12`.

The `edges` BLOB stores pairs of vertex indices that define the visible hard edges of the model (the characteristic lines in SketchUp-style rendering). This data is separate from the triangle mesh because edge visibility is a modeling concept, not a rendering primitive — some edges of a triangle mesh are internal to a smooth surface and should not be drawn.

The optional `uv0` BLOB stores the primary UV texture coordinate set. Additional UV sets for lightmaps or detail textures may be added in future versions.

The `face_materials` BLOB maps each triangle to a material ID using an indexed encoding scheme defined in [Section 6.4](#64-face-material-assignment). Writers MUST sort triangles by material before writing to ensure optimal draw-call batching and compact face material encoding.

#### 4.5.3 B-Rep Geometry: orb_geometry_brep

```sql
CREATE TABLE orb_geometry_brep (
    entity_id    BLOB PRIMARY KEY NOT NULL
                 REFERENCES orb_entities(id) ON DELETE CASCADE,
    kernel       TEXT NOT NULL,       -- e.g., "truck-0.5", "occt-7.8"
    brep_data    BLOB NOT NULL,       -- kernel-native serialized B-Rep
    brep_format  TEXT NOT NULL,        -- "bincode", "json"
    brep_step    BLOB,                -- STEP AP214 portable encoding
    tess_params  TEXT                  -- JSON: tessellation settings used
);
```

The `kernel` column identifies the geometry kernel and version that produced the `brep_data`. This is essential for correct deserialization: a B-Rep serialized by `truck` v0.5 may not be readable by `truck` v0.6 if the internal format changed.

The `brep_format` column identifies the serialization format of `brep_data`. `"bincode"` is the default for `truck`'s native serialization. `"json"` is used for debugging and inspection.

**The `brep_step` column** stores a portable, kernel-independent encoding of the same B-Rep geometry in STEP AP214 format (ISO 10303-214). This column exists to ensure long-term archival readability. The native `brep_data` is optimized for fast deserialization by the specific kernel that produced it, but is inherently tied to that kernel's version and serialization format. The STEP encoding can be read by any geometry kernel that supports the STEP standard — which includes every major commercial and open-source kernel (OpenCASCADE, Parasolid, ACIS, `truck` via conversion).

Writers MUST populate `brep_step` for all B-Rep data. This is a firm requirement, not optional. The STEP encoding is larger than native formats (typically 3-5x for complex geometry), but this cost is justified by the format's longevity principle: an Orb file must remain fully readable decades after creation, regardless of which geometry kernels exist at that time. Readers that understand the native `kernel` and `brep_format` SHOULD prefer `brep_data` for performance. Readers that do not recognize the kernel MUST fall back to `brep_step`. If both are unreadable, the reader falls back to the mesh in `orb_geometry_mesh`.

The `tess_params` column stores the tessellation parameters (chord tolerance, angle tolerance, maximum edge length) that were used to generate the corresponding mesh in `orb_geometry_mesh`. This allows the application to detect when retessellation is needed (e.g., after a parameter change) versus when the cached mesh is still valid.

### 4.6 Materials: orb_materials and orb_textures

The material system uses a physically-based rendering (PBR) metallic-roughness model, consistent with glTF 2.0 and modern real-time renderers, extended with specification data to support architectural material schedules and BIM workflows.

```sql
CREATE TABLE orb_materials (
    id                     BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name                   TEXT NOT NULL,
    -- PBR rendering properties
    base_color             TEXT NOT NULL DEFAULT 'CCCCCC',  -- hex RGB
    base_color_tex         BLOB REFERENCES orb_textures(id),
    metallic               REAL NOT NULL DEFAULT 0.0,  -- 0.0 to 1.0
    roughness              REAL NOT NULL DEFAULT 0.5,  -- 0.0 to 1.0
    metallic_roughness_tex BLOB REFERENCES orb_textures(id),
    normal_tex             BLOB REFERENCES orb_textures(id),
    normal_scale           REAL NOT NULL DEFAULT 1.0,
    emissive_color         TEXT DEFAULT '000000',
    emissive_tex           BLOB REFERENCES orb_textures(id),
    opacity                REAL NOT NULL DEFAULT 1.0,  -- 0.0 to 1.0
    double_sided           INTEGER NOT NULL DEFAULT 0,
    -- Specification / product data
    spec_name              TEXT,         -- product or finish name
    spec_manufacturer      TEXT,         -- manufacturer name
    spec_product_id        TEXT,         -- SKU, model number, catalog code
    spec_url               TEXT,         -- manufacturer product page URL
    spec_properties        TEXT          -- JSON: thickness, fire rating, cost, etc.
);
```

```sql
CREATE TABLE orb_textures (
    id     BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name   TEXT,
    format TEXT NOT NULL,   -- "png", "webp", "jpg"
    width  INTEGER NOT NULL,
    height INTEGER NOT NULL,
    data   BLOB NOT NULL    -- compressed image bytes
);
```

Textures are stored as compressed image data (PNG, WebP, or JPEG) rather than raw pixel arrays. This keeps file sizes manageable for models with many textured materials. WebP is recommended for new files due to its superior compression ratio. Readers MUST support all three formats.

The material model intentionally omits advanced PBR features (clearcoat, subsurface scattering, anisotropy) in v1.0 to keep the specification focused. These may be added as optional properties in future versions via JSON extension fields.

#### 4.6.1 Specification Data

The `spec_*` columns carry product and architectural specification data that exists independently of rendering properties. This separation is deliberate: the PBR fields control how the material *looks*, while the spec fields describe what the material *is*. An architect can generate a material schedule from the spec fields without any understanding of PBR rendering.

The `spec_properties` column is a JSON object for domain-specific properties that vary by material type:

```json
{
  "thickness": { "type": "length", "value": 12.7, "unit": "mm" },
  "fire_rating": { "type": "string", "value": "Class A" },
  "cost_per_sqm": { "type": "float", "value": 45.00, "currency": "USD" },
  "acoustic_rating": { "type": "string", "value": "STC 55" },
  "sustainability": { "type": "string", "value": "FSC Certified" }
}
```

The spec fields are all optional. A hobbyist modeling furniture may never populate them. An architect producing construction documents relies on them for scheduling and specification sheets. This aligns with the progressive complexity principle.

### 4.7 Parametric Components

The parametric system is the bridge between Orb as a static geometry format and Orb as a living design document. Component definitions are reusable, parameterized geometry generators written in the Orbit scripting language. Instances of these definitions appear in the scene graph with per-instance parameter overrides.

#### 4.7.1 Component Definitions: orb_component_defs

```sql
CREATE TABLE orb_component_defs (
    id           BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name         TEXT NOT NULL,
    category     TEXT,              -- "doors", "windows", "furniture", etc.
    description  TEXT,
    script_lang  TEXT NOT NULL DEFAULT 'orbit-script-1.0',
    script       TEXT NOT NULL,     -- Orbit scripting language source
    script_hash  TEXT NOT NULL,     -- SHA-256 hex of script text
    param_schema TEXT NOT NULL,     -- JSON: parameter definitions
    thumbnail    BLOB,             -- PNG, recommended 256x256
    author       TEXT,
    license      TEXT,
    version      TEXT,             -- semver of the component definition
    created_at   TEXT NOT NULL,
    modified_at  TEXT NOT NULL
);
```

The `script_lang` column identifies the scripting language and version used to write the component script. The default value `"orbit-script-1.0"` refers to the first version of the Orbit scripting language. This field is critical for forward compatibility: when the scripting language evolves (new built-in functions, changed semantics, new syntax), the runtime can inspect `script_lang` to determine whether it can execute the script, attempt an automated migration, or fall back to the cached mesh geometry. Without this field, opening an old file with new software risks silently regenerating geometry with incorrect dimensions — the most dangerous class of bug in a CAD application, because the user has no indication that their door component just changed size.

Implementations MUST check `script_lang` before executing a component script. If the runtime does not support the specified language version, it MUST NOT execute the script and MUST instead use the cached mesh geometry from `orb_geometry_mesh` and present a clear warning to the user.

The `param_schema` column is a JSON array defining the component's parameters. Each parameter has a name, type, default value, and optional constraints:

```json
[
  {
    "name": "width",
    "type": "length",
    "default": 900,
    "min": 300,
    "max": 3000,
    "label": "Frame Width",
    "group": "Dimensions"
  },
  {
    "name": "panes",
    "type": "int",
    "default": 2,
    "min": 1,
    "max": 6,
    "label": "Number of Panes",
    "group": "Configuration"
  },
  {
    "name": "frame_material",
    "type": "enum",
    "options": ["pine", "oak", "aluminum", "upvc"],
    "default": "pine",
    "label": "Frame Material",
    "group": "Materials"
  }
]
```

Supported parameter types are: `length` (stored as mm), `angle` (stored as degrees), `int`, `float`, `bool`, `string`, `enum`, `color` (hex RGB), and `material_ref` (UUID reference to `orb_materials`).

The `script_hash` column enables cache invalidation: if the script changes, the hash changes, and all instances know their cached geometry is stale and must be regenerated.

#### 4.7.2 Component Instances: orb_component_instances

```sql
CREATE TABLE orb_component_instances (
    entity_id    BLOB PRIMARY KEY NOT NULL
                 REFERENCES orb_entities(id) ON DELETE CASCADE,
    def_id       BLOB NOT NULL
                 REFERENCES orb_component_defs(id),
    param_values TEXT NOT NULL,  -- JSON object of overridden values
    cache_valid  INTEGER NOT NULL DEFAULT 0,
    cache_hash   TEXT             -- SHA-256 of (script_hash + param_values)
);

CREATE INDEX idx_comp_inst_def ON orb_component_instances(def_id);
```

The `param_values` column stores only the parameters that differ from the definition's defaults. An empty JSON object `{}` means all defaults apply. This keeps the common case compact — most instances of a standard door differ only in width and height.

The `cache_valid` flag and `cache_hash` enable lazy regeneration. When the application opens a file, it compares each instance's `cache_hash` against a fresh hash of `(script_hash + param_values)`. If they match, the cached mesh in `orb_geometry_mesh` is used directly. If not, the script is re-executed to regenerate geometry. This means `.orb` files store pre-computed geometry that can be displayed immediately, with regeneration only when parameters or scripts have changed.

### 4.8 Saved Views: orb_saved_views

Saved views (named camera positions) are a core feature of every CAD application. Architects use them to define presentation viewpoints, documentation sheet views, and working orientations. The `orb_saved_views` table is a first-class schema table, not an extension, because saved views are needed by all conformance levels — including the web viewer, which must be able to offer a list of named viewpoints to the user.

```sql
CREATE TABLE orb_saved_views (
    id          BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    -- Camera state
    eye_x       REAL NOT NULL,              -- camera position (mm)
    eye_y       REAL NOT NULL,
    eye_z       REAL NOT NULL,
    target_x    REAL NOT NULL,              -- look-at point (mm)
    target_y    REAL NOT NULL,
    target_z    REAL NOT NULL,
    up_x        REAL NOT NULL DEFAULT 0.0,  -- up vector
    up_y        REAL NOT NULL DEFAULT 0.0,
    up_z        REAL NOT NULL DEFAULT 1.0,
    fov         REAL,                       -- field of view (degrees); NULL = ortho
    ortho_width REAL,                       -- ortho view width (mm); NULL = perspective
    -- Optional state
    layer_visibility TEXT,                  -- JSON: {layer_id_hex: bool, ...}
    section_plane_id BLOB REFERENCES orb_entities(id),
    description      TEXT,
    thumbnail        BLOB                   -- PNG, recommended 512x512
);
```

A view is either perspective (`fov` is set, `ortho_width` is `NULL`) or orthographic (`ortho_width` is set, `fov` is `NULL`). Exactly one of the two MUST be non-NULL.

The optional `layer_visibility` column stores per-layer visibility overrides for this view as a JSON object mapping hex-encoded layer UUIDs to boolean values. Layers not listed in the object use their default visibility from `orb_layers`. This enables presentation views that show or hide specific layers without affecting the model's default layer state.

The optional `section_plane_id` references an `orb_entities` row of type `section_plane` that is active in this view.

### 4.9 Selection Sets: orb_selection_sets

Selection sets provide named, non-hierarchical collections of entities. Unlike groups (which are spatial and affect the scene tree), selection sets are organizational bookmarks — they do not appear in the scene graph and have no transform or visibility implications. Typical uses include bid packages, construction phases, design options, and review sets.

```sql
CREATE TABLE orb_selection_sets (
    id          BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name        TEXT NOT NULL,
    description TEXT,
    color       TEXT,                        -- hex RGB for highlight color
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE orb_selection_set_members (
    set_id    BLOB NOT NULL REFERENCES orb_selection_sets(id) ON DELETE CASCADE,
    entity_id BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    PRIMARY KEY (set_id, entity_id)
);

CREATE INDEX idx_selset_entity ON orb_selection_set_members(entity_id);
```

An entity may belong to any number of selection sets. The `idx_selset_entity` index enables efficient lookup of which sets a given entity belongs to (useful when displaying set membership in a properties panel).

### 4.10 Annotations

Entities with `entity_type = "annotation"` represent dimensions, labels, leaders, text notes, and other 2D documentation elements. The annotation domain is large and complex — a full annotation specification encompasses dimension styles, tolerancing, associativity with referenced geometry, text formatting, and symbol libraries.

**In v1.0, annotation content is opaque.** The Orb format defines that annotation entities exist in the scene graph (with position, visibility, and layer assignment), but their internal data — the specific dimension value, referenced geometry, text content, and display style — is stored in the application-specific extension table `x_orbit_annotations`. Other readers MUST treat annotation entities as non-interactive overlay geometry: they SHOULD display the annotation's cached mesh representation from `orb_geometry_mesh` (which contains the tessellated text and leader lines as renderable geometry), but they MUST NOT attempt to interpret, edit, or regenerate annotation content.

This intentional opacity prevents the dangerous situation where a third-party reader misinterprets annotation data and displays incorrect dimensions. Incorrect geometry is visible; incorrect dimensions are silent and can cause construction errors.

Future versions of the Orb specification (anticipated in v1.2) will define a full annotation schema as a core table, including associative dimension types, text styling, tolerance notation, and reference geometry binding. The annotation data format will be designed to be interoperable with IFC annotation entities and compatible with standard documentation workflows.

### 4.11 BIM Classifications: orb_classifications

BIM data is stored as entity-level classifications that map to industry-standard taxonomies. This table is entirely optional — a valid Orb file need not contain any BIM data. The progressive disclosure principle applies: hobbyist files have no classifications, architectural files may have IFC classes, and construction documentation files carry full property sets.

```sql
CREATE TABLE orb_classifications (
    entity_id   BLOB NOT NULL
                REFERENCES orb_entities(id) ON DELETE CASCADE,
    system      TEXT NOT NULL,      -- "IFC4", "UniFormat", "OmniClass"
    class       TEXT NOT NULL,      -- e.g., "IfcWall", "IfcDoor"
    pset_name   TEXT,               -- property set name
    properties  TEXT NOT NULL,      -- JSON object of typed properties
    PRIMARY KEY (entity_id, system, pset_name)
);

CREATE INDEX idx_class_system ON orb_classifications(system, class);
```

An entity may have multiple classification rows — one per classification system and property set. For example, a wall entity might have:

- An IFC4 classification with class `"IfcWallStandardCase"` and a `Pset_WallCommon` property set containing `fire_rating`, `is_external`, and `thermal_transmittance`
- A UniFormat classification with class `"B2010"` (Exterior Walls)
- A custom property set with project-specific metadata

The `properties` column is a JSON object where each key is a property name and each value includes both the data and its type:

```json
{
  "fire_rating": { "type": "string", "value": "2HR" },
  "is_external": { "type": "bool", "value": true },
  "thermal_transmittance": {
    "type": "float",
    "value": 0.28,
    "unit": "W/(m2*K)"
  },
  "load_bearing": { "type": "bool", "value": true }
}
```

### 4.12 External References: orb_external_refs

Real architectural projects are rarely contained in a single file. A site model may reference a building model, which references furniture families from a shared library, which reference material definitions from a manufacturer catalog. The Orb format reserves the external reference table to support these workflows in future versions.

```sql
CREATE TABLE orb_external_refs (
    id          BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    name        TEXT NOT NULL,              -- display name
    ref_type    TEXT NOT NULL,              -- "linked", "embedded", "library"
    uri         TEXT NOT NULL,              -- file path, URL, or content hash
    format      TEXT NOT NULL,              -- "orb", "skp", "ifc", etc.
    transform   BLOB,                       -- 4x4 f64 placement in host model
    status      TEXT NOT NULL DEFAULT 'unresolved',  -- "resolved", "unresolved", "broken"
    last_synced TEXT,                        -- ISO 8601: last successful load
    properties  TEXT                         -- JSON: ref-type-specific metadata
);
```

**In v1.0, external references are defined but non-functional.** Writers MAY create `orb_external_refs` rows to record that a reference relationship exists (e.g., when importing a SketchUp file that referenced external components), but readers MUST NOT attempt to resolve or load external references. The resolution protocol — how references are discovered, loaded, authenticated, version-matched, and merged into the host model — will be specified in a future version (anticipated in v1.2 or v2.0).

The table is included in v1.0 to prevent third-party implementors from designing extension tables for linked files that would conflict with the eventual core specification. By reserving the table name and basic schema shape now, the format maintains a clean migration path.

Reference types are defined as:

| `ref_type` | Description |
|-----------|-------------|
| `linked` | A live reference to an external file. Changes to the external file are reflected when the reference is reloaded. Analogous to Revit's linked models or ArchiCAD's hotlinked modules. |
| `embedded` | A snapshot of an external file's content embedded within this file. The original source is recorded for provenance but the data is self-contained. |
| `library` | A reference to a component library or material catalog. The library may be a local file, a network path, or a URL to a shared repository. |

### 4.13 Spatial Occupancy: orb_occupancy

The Orb format treats spatial integrity as a kernel-level property, not an after-the-fact analysis feature. Just as SQLite enforces referential integrity through foreign key constraints — preventing the creation of orphaned references — Orbit enforces spatial integrity through the occupancy engine, preventing the creation of physically impossible geometry intersections without explicit acknowledgment.

This design eliminates the need for separate clash detection tools (such as Autodesk Navisworks) for single-discipline and small-team workflows. The traditional CAD industry workflow — author geometry in isolation, federate models in a clash detection tool, generate a report with thousands of clashes, resolve them manually in a spreadsheet — exists because no modeling tool enforces spatial constraints at the point of creation. Orbit does.

The `orb_occupancy` table associates each entity with its spatial occupancy classification, clearance requirements, building system assignment, and conflict resolution priority.

```sql
CREATE TABLE orb_occupancy (
    entity_id       BLOB PRIMARY KEY NOT NULL
                    REFERENCES orb_entities(id) ON DELETE CASCADE,
    occupancy_type  TEXT NOT NULL DEFAULT 'solid',
    clearance_data  BLOB,
    priority        INTEGER NOT NULL DEFAULT 100,
    system          TEXT
);

CREATE INDEX idx_occupancy_system ON orb_occupancy(system);
```

#### 4.13.1 Occupancy Types

The `occupancy_type` column classifies how an entity occupies space:

| `occupancy_type` | Description |
|---|---|
| `solid` | The entity physically exists and nothing else can occupy its space. This is the default for walls, beams, columns, floors, and all standard geometry. |
| `penetrable` | The entity exists but can be legally penetrated with appropriate detailing. A wall can have a pipe sleeve punched through it; a floor can have a chase cut into it. Penetrations are recorded as acknowledged exceptions in `orb_clash_results`. |
| `reservation` | The entity does not physically exist yet, but the space is reserved. This is how engineers "claim" routing corridors before finalizing sizes. Other systems must route around reservations. |

#### 4.13.2 Clearance Envelopes

The optional `clearance_data` BLOB stores one or more clearance volumes — the functional space an entity needs beyond its physical geometry. Clearance volumes are distinct from the entity's geometry: a door's geometry is the door panel, but its clearance envelope includes the swing arc and approach zones on both sides.

Clearance envelopes are stored as a packed array of simple geometric primitives (axis-aligned boxes, oriented boxes, cylinders, half-cylinders, extruded 2D shapes) rather than full B-Rep. This keeps intersection tests fast — clearance checks must be responsive enough for interactive modeling.

The encoding format for `clearance_data` is:

```
[envelope_count (u16)]
[envelope_0: type (u8) + parameters (variable)]
[envelope_1: type (u8) + parameters (variable)]
...
```

Envelope types:

| Type ID | Name | Parameters |
|---|---|---|
| `0x01` | Axis-aligned box | min_x, min_y, min_z, max_x, max_y, max_z (6 x f64) |
| `0x02` | Oriented box | center (3 x f64), half_extents (3 x f64), rotation (4 x f64 quaternion) |
| `0x03` | Cylinder | base_center (3 x f64), axis (3 x f64), radius (f64), height (f64) |
| `0x04` | Half-cylinder | base_center (3 x f64), axis (3 x f64), normal (3 x f64), radius (f64), height (f64) |

All clearance dimensions are in millimeters, in the entity's local coordinate space. The entity's world transform applies to clearance volumes just as it does to geometry.

Clearance envelopes are determined by entity classification. Standard clearances are defined by the runtime for common entity types:

| Entity Classification | Clearance Description |
|---|---|
| `IfcDoor` | Half-cylinder swing arc on each side (radius = door width), plus 300mm rectangular approach zone |
| `IfcSanitaryTerminal` (toilet) | 500mm frontal clearance, 200mm lateral clearance |
| `IfcSanitaryTerminal` (sink/vanity) | 600mm frontal clearance |
| `IfcFlowTerminal` (electrical panel) | 900mm x 900mm x 2000mm clear zone in front (per NEC) |
| Kitchen island | 1050mm perimeter clearance on all accessible sides |
| Staircase | Clear headroom envelope (2000mm minimum) along entire run |

Writers SHOULD populate `clearance_data` for all entities with known functional clearance requirements. Readers that do not perform spatial integrity checks MAY ignore `clearance_data`.

#### 4.13.3 System Assignment

The `system` column categorizes the entity by building system, enabling system-versus-system clash analysis:

| `system` | Description |
|---|---|
| `structural` | Load-bearing elements: beams, columns, foundations, structural slabs |
| `architectural` | Walls, floors, ceilings, roofs, doors, windows, stairs |
| `mechanical` | HVAC ducts, air handlers, diffusers |
| `plumbing` | Pipes, fixtures, drains, water heaters |
| `electrical` | Conduit, panels, outlets, lighting fixtures |
| `fire_protection` | Sprinkler pipes, heads, risers, fire dampers |
| `furniture` | Movable furnishings, equipment, casework |

When `system` is `NULL`, the entity is unclassified and participates in all-versus-all clash checking.

#### 4.13.4 Priority and Conflict Resolution

The `priority` column determines which entity takes precedence when two entities conflict. Lower values mean higher priority. The recommended priority scale is:

| Priority Range | System | Rationale |
|---|---|---|
| 10-19 | Structural | Structure cannot move; everything else routes around it |
| 20-29 | Architectural | Walls and floors define the spatial envelope |
| 30-39 | Fire protection | Life safety systems have regulatory priority |
| 40-49 | Plumbing | Gravity-dependent; limited routing flexibility |
| 50-59 | Mechanical | Most routing flexibility but large cross-sections |
| 60-69 | Electrical | Small cross-sections; most routing flexibility |
| 90-99 | Furniture | Easily moved; lowest spatial priority |
| 100 | Default | Unclassified entities |

When the spatial integrity engine detects a conflict between two entities, the entity with the higher priority (lower number) is assumed to be correctly placed, and the resolution suggestion targets the lower-priority entity. This mirrors the real-world construction coordination hierarchy.

#### 4.13.5 Spatial Integrity Check Protocol

Conforming Level 3 writers MUST enforce spatial integrity through the following protocol on every geometry-modifying operation (entity creation, transform change, geometry edit):

**Step 1: Broad-phase.** Query `orb_spatial_index` for all entities whose AABB intersects the modified entity's AABB (including clearance envelope bounds). This is O(log n) via the R-tree.

**Step 2: Narrow-phase occupancy.** For each broad-phase candidate, test actual geometry intersection. Use GJK/SAT algorithms on convex decompositions of the meshes. This identifies hard clashes (solid-vs-solid intersections).

**Step 3: Clearance check.** For each broad-phase candidate, test the modified entity's geometry against the candidate's clearance envelopes, and the candidate's geometry against the modified entity's clearance envelopes. This identifies functional violations.

**Step 4: Classification.** Each detected intersection is classified by severity:

| Clash Type | Severity | Example | Default Action |
|---|---|---|---|
| Solid vs. solid (same system) | `error` | Two walls overlapping | Block placement |
| Solid vs. solid (different system) | `error` | Beam through duct | Block placement |
| Solid vs. penetrable | `warning` | Pipe through wall | Allow, record penetration |
| Solid vs. reservation | `warning` | Wall through MEP corridor | Allow with acknowledgment |
| Clearance vs. solid | `warning` | Door swing hits counter | Report, suggest adjustment |
| Clearance vs. clearance | `info` | Two door swings overlap | Report only |

**Step 5: Resolution.** The response depends on the operation context:

- **Interactive modeling**: Present the diagnostic immediately. Highlight clashing entities. Offer resolution suggestions based on priority (suggest moving the lower-priority entity).
- **Solver/script execution**: Accumulate clashes. The solver MAY attempt automatic resolution (re-route, shift, resize) before falling back to diagnostic reporting.
- **Import**: Accumulate all clashes silently and present a clash report after import completes.

Level 1 and Level 2 readers MAY ignore `orb_occupancy` entirely. They are not required to perform spatial integrity checks.

### 4.14 Clash Detection: orb_clash_results

The `orb_clash_results` table records detected spatial conflicts and their resolution state. This table serves the same function as a Navisworks clash report, but is embedded in the model file — when you open an `.orb` file, you see the spatial integrity state alongside the geometry. Unresolved clashes travel with the file.

```sql
CREATE TABLE orb_clash_results (
    id                   BLOB PRIMARY KEY NOT NULL,  -- UUIDv7
    entity_a             BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    entity_b             BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    clash_type           TEXT NOT NULL,     -- "hard", "clearance", "penetration"
    severity             TEXT NOT NULL,     -- "error", "warning", "info"
    system_a             TEXT,
    system_b             TEXT,
    intersection_point_x REAL,
    intersection_point_y REAL,
    intersection_point_z REAL,
    distance             REAL,             -- penetration depth (hard) or clearance
                                           -- shortfall (clearance violation), in mm
    status               TEXT NOT NULL DEFAULT 'active',
    resolved_by          TEXT,             -- description of resolution action
    detected_at          TEXT NOT NULL,    -- ISO 8601
    resolved_at          TEXT,             -- ISO 8601
    UNIQUE(entity_a, entity_b, clash_type)
);

CREATE INDEX idx_clash_status ON orb_clash_results(status);
CREATE INDEX idx_clash_entity_a ON orb_clash_results(entity_a);
CREATE INDEX idx_clash_entity_b ON orb_clash_results(entity_b);
```

Clash types are:

| `clash_type` | Description |
|---|---|
| `hard` | Two solid entities occupy the same physical space. This is a physical impossibility that must be resolved. |
| `clearance` | An entity's geometry violates another entity's clearance envelope. The space is functionally unusable as designed. |
| `penetration` | A solid entity passes through a penetrable entity. This is recorded as an intentional exception that may require detailing (e.g., a pipe sleeve through a wall). |

Status values are:

| `status` | Description |
|---|---|
| `active` | The clash has been detected and has not been addressed. |
| `resolved` | The clash has been resolved by modifying one or both entities. The `resolved_by` field describes the action taken. |
| `approved` | The clash has been reviewed and explicitly approved as acceptable (e.g., an intentional intersection for a design feature). |
| `ignored` | The clash has been reviewed and marked as not requiring action (e.g., a cosmetic overlap below construction tolerance). |

The `distance` column stores the penetration depth for hard clashes (how far the entities overlap, in mm) or the clearance shortfall for clearance violations (how much less than the required clearance is available, in mm). This value is useful for prioritizing clash resolution — a 200mm penetration is more urgent than a 5mm one.

Writers MUST update `orb_clash_results` whenever spatial integrity checks detect new clashes or when previously-detected clashes are resolved by geometry changes. Writers SHOULD remove `resolved` entries whose resolution has been verified (both entities moved apart). Writers MUST NOT silently delete `active` or `approved` entries — these represent state that may have been reviewed by a user.

The `UNIQUE(entity_a, entity_b, clash_type)` constraint ensures that the same clash is not recorded twice. When entity A and entity B have both a hard clash and a clearance violation, these are recorded as two separate rows.

Readers that do not perform clash analysis MAY ignore `orb_clash_results`. Level 1 readers (web viewers) MAY read the table to display clash markers as visual overlays without performing clash detection themselves.

---

## 5. Versioning and Extensibility

### 5.1 Semantic Versioning

The `format_version` key in `orb_meta` follows semantic versioning (`MAJOR.MINOR.PATCH`):

- **Major version change** (e.g., 1.x.x to 2.0.0): Incompatible schema changes. A v1 reader cannot open a v2 file. Major version changes should be extremely rare and accompanied by migration tooling.
- **Minor version change** (e.g., 1.0.x to 1.1.0): New tables or columns added. A v1.0 reader can open a v1.1 file but may not understand all data. It MUST NOT reject the file.
- **Patch version change** (e.g., 1.0.0 to 1.0.1): Clarifications to the specification, no schema changes.

### 5.2 Forward Compatibility Rules

To ensure forward compatibility, all conforming readers MUST follow these rules:

1. A reader MUST check `format_version` before processing any other data.
2. A reader MUST reject files with an unrecognized major version and present a clear error message suggesting the user update their software.
3. A reader MUST ignore (not reject) unrecognized tables. Future minor versions may add tables; existing readers should silently skip them.
4. A reader MUST ignore unrecognized columns in known tables. SQLite's dynamic typing ensures that unknown columns do not prevent reading known columns.
5. A reader MUST ignore unrecognized keys in `orb_meta`.
6. A reader MUST ignore unrecognized values in JSON columns (`properties`, `param_values`, etc.) unless those values are required for correct interpretation of the data.

### 5.3 Application-Specific Extensions

Applications may store custom data in two ways:

- **Custom tables.** Applications may create tables with the prefix `x_` followed by an application identifier (e.g., `x_orbit_undo_history`, `x_orbit_annotations`). Other readers MUST ignore these tables.
- **Custom meta keys.** Applications may store custom keys in `orb_meta` with the prefix `x-` (e.g., `"x-orbit-last-camera-position"`, `"x-orbit-file-mode"`). Other readers MUST ignore these keys.

Extension data MUST NEVER be required for correct interpretation of the core model. A reader that ignores all `x_` tables and `x-` keys must still be able to display the model geometry, materials, and scene hierarchy correctly.

---

## 6. Encoding Details

### 6.1 UUIDv7 Generation

Identifiers are UUIDv7 as specified by RFC 9562. The first 48 bits encode a Unix millisecond timestamp, followed by version and variant bits, then random data. The BLOB storage is the raw 16-byte binary representation (not the 36-character hyphenated string form). This provides:

- Temporal ordering, useful for B-tree indexing
- Global uniqueness without a central authority, essential for offline and collaborative use
- Compact 16-byte storage

### 6.2 Transformation Matrices

Transforms are 4x4 affine transformation matrices stored as 128 bytes (16 x f64). The layout is column-major:

```
Byte offset:  0   8  16  24  32  40  48  56  64  72  80  88  96 104 112 120
Matrix cell: m00 m10 m20 m30 m01 m11 m21 m31 m02 m12 m22 m32 m03 m13 m23 m33

Where the matrix is interpreted as:
| m00 m01 m02 m03 |   | Xx Yx Zx Tx |
| m10 m11 m21 m13 | = | Xy Yy Zy Ty |
| m20 m21 m22 m23 |   | Xz Yz Zz Tz |
| m30 m31 m32 m33 |   |  0  0  0  1 |
```

The identity transform is 128 bytes of: `1.0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 1.0, 0, 0, 0, 0, 1.0` (as f64 little-endian). All transforms are little-endian, matching the native byte order of x86, ARM, and WASM targets.

### 6.3 Mesh Vertex Packing

Vertex data in `orb_geometry_mesh` is packed without any padding or interleaving. Each attribute is stored in its own BLOB column:

- `positions`: `[x0, y0, z0, x1, y1, z1, ...]` as f32 little-endian. Total: `vertex_count x 12` bytes.
- `normals`: `[nx0, ny0, nz0, nx1, ny1, nz1, ...]` as f32 little-endian. Total: `vertex_count x 12` bytes.
- `indices`: `[i0, i1, i2, i3, i4, i5, ...]` as u32 little-endian. Total: `triangle_count x 12` bytes.
- `edges`: `[a0, b0, a1, b1, ...]` as u32 little-endian. Total: `edge_count x 8` bytes.
- `uv0`: `[u0, v0, u1, v1, ...]` as f32 little-endian. Total: `vertex_count x 8` bytes.

This separated layout (structure of arrays) is preferred over interleaved (array of structures) because it allows the web viewer to fetch only the attributes it needs. A wireframe preview, for example, needs only `positions` and `edges` — not normals or UVs.

### 6.4 Face Material Assignment

The `face_materials` BLOB uses an indexed encoding scheme that avoids storing 16-byte UUIDs per triangle. The encoding has two parts: a material palette (mapping local indices to UUIDs) followed by a per-triangle index array.

**Writers MUST sort triangles by material before writing the index buffer.** This is required for two reasons: it ensures the face material array is maximally compressible, and it enables renderers to batch draw calls by material without re-sorting at load time.

```
Encoding layout:
  [palette_count (u16)]
  [material_uuid_0 (16 bytes)]
  [material_uuid_1 (16 bytes)]
  ...
  [material_uuid_N (16 bytes)]
  [tri_0_material_index (u16)]
  [tri_1_material_index (u16)]
  ...
  [tri_M_material_index (u16)]

Example: 10,000 triangles using 3 materials:
  palette_count = 3
  palette: [wood_uuid, glass_uuid, steel_uuid]  ->  48 bytes
  per-triangle indices: 10,000 x 2 bytes          ->  20,000 bytes
  Total: 2 + 48 + 20,000 = 20,050 bytes
```

The `palette_count` is a u16 (maximum 65,535 materials per mesh, which is far beyond practical use). Each `material_index` is a u16 referencing a position in the palette. The palette maps local indices to global material UUIDs in `orb_materials`.

If `face_materials` is `NULL`, all faces use the entity's default material (determined by the first material referenced in the entity or its nearest ancestor with a material assignment). A single-material body has no `face_materials` BLOB, saving space in the common case.

Because triangles are sorted by material, the per-triangle index array will consist of contiguous runs of the same index. Implementations MAY apply additional compression (e.g., SQLite's built-in zlib compression via the `ZIPVFS` extension) but this is not required by the specification.

---

## 7. Web Streaming Protocol

A primary design goal of the Orb format is enabling browser-based model viewing without downloading the entire file. This section defines the protocol by which a web viewer can progressively load an Orb file over HTTP.

### 7.1 Architecture

The Orb web viewer is a WebAssembly module compiled from Rust that runs in the browser. It uses the WebGPU API (via `wgpu`) for rendering and accesses the remote `.orb` file via HTTP range requests against the SQLite database pages.

The key enabling technology is the SQLite VFS (Virtual File System) layer. SQLite's I/O is abstracted through a VFS interface that can be implemented to read pages from any backing store. For the web viewer, this VFS issues HTTP Range requests to fetch individual 4096-byte pages from the remote `.orb` file. Only the pages touched by a given SQL query are fetched.

### 7.2 Loading Sequence

The web viewer follows a defined loading sequence to achieve the fastest possible first paint:

**Phase 1: Metadata** (1-2 requests, ~8 KB). Fetch the SQLite header page and the `orb_meta` table. This gives the viewer the format version, display units, up axis, and thumbnail. The viewer can display the thumbnail and model name immediately.

**Phase 2: Scene structure and spatial index** (1-5 requests, ~50 KB typical). Fetch the `orb_entities` table, `orb_layers` table, `orb_entity_rowids`, and `orb_spatial_index`. The viewer now knows the full scene hierarchy, entity transforms, visibility states, and world-space bounding boxes. It can render a bounding-box wireframe of the entire model at this point, providing spatial context while geometry loads. The spatial index enables immediate frustum-based prioritization for the next phase.

**Phase 3: Materials** (1-3 requests, variable). Fetch the `orb_materials` table. Texture data is deferred; initial rendering uses flat-shaded material colors.

**Phase 4: Mesh geometry** (bulk, progressive). Fetch `orb_geometry_mesh` rows, prioritized by visual importance. The viewer uses the spatial index and a priority queue ordered by: (a) whether the entity's bounding box intersects the camera frustum, (b) the entity's screen-space projected area (larger objects first), and (c) distance from the camera. Entities outside the frustum are deferred until the user pans or zooms to reveal them.

**Phase 5: Textures** (background, on-demand). Fetch `orb_textures` rows as materials are applied to visible geometry. Large textures may be downsampled server-side if a tile server is available, but the base protocol assumes the full texture is fetched.

**Phase 6: Saved views** (optional, on-demand). Fetch `orb_saved_views` to populate the view selector UI. This is low-priority and can happen after initial geometry is visible.

The viewer NEVER fetches `orb_geometry_brep`, `orb_component_defs` scripts, or `orb_classifications` data. These are editing-mode and BIM-mode data that a view-only client does not need.

### 7.3 HTTP Requirements

The server hosting `.orb` files MUST support:

- **Range requests.** The server must return `206 Partial Content` responses for `Range` headers. This is standard for nginx, Apache, S3, CloudFront, and all major CDNs and static file hosts.
- **CORS headers.** If the viewer is hosted on a different origin than the `.orb` file, the server must include `Access-Control-Allow-Origin` and `Access-Control-Allow-Headers: Range`.
- **Content-Length.** The server must return the total file size in the `Content-Length` header (or in `Content-Range` for partial responses) so the SQLite VFS can determine the database size without fetching the entire file.

No server-side processing, API, or special software is required. An Orb file served as a static file from any standards-compliant HTTP server is viewable in the web viewer.

### 7.4 Companion Stream Format

The SQLite-over-HTTP approach described above is flexible and requires no server-side tooling, but it has a fundamental performance limitation: each SQL query may trigger multiple non-sequential page fetches, each incurring HTTP roundtrip latency. For a 100 MB model on a 50 ms latency connection, fetching 200 non-sequential 4 KB pages means 200 sequential HTTP requests — potentially 10 seconds of latency before meaningful geometry appears.

For production deployments where viewing performance is critical, Orb defines an optional companion format: the `.orb.stream` file. This is a pre-linearized, pre-sorted binary blob optimized for sequential reading. The canonical `.orb` file remains the authoritative source; the `.orb.stream` file is a derived cache.

**Generation.** The stream file is produced by an export tool:

```
orbit export-stream model.orb -> model.orb.stream
```

**Structure.** The stream file is organized in the exact order of the loading sequence:

```
[Header: magic, version, section offsets (fixed size)]
[Section 1: orb_meta key-value pairs]
[Section 2: scene graph (entities, layers, spatial index)]
[Section 3: material palette]
[Section 4: mesh geometry, sorted by spatial priority]
[Section 5: texture data]
[Section 6: saved views]
```

Each section is independently addressable via the header's offset table. A viewer can fetch the header (one request), then stream sections sequentially. Because the data is pre-sorted and pre-linearized, large sections can be fetched in a single HTTP range request, dramatically reducing latency.

**Discovery.** The web viewer attempts to load `<url>.stream` before falling back to range-requesting the `.orb` file directly. If the `.stream` file is not found (404), the viewer transparently falls back to the SQLite range-request protocol with no user-visible impact.

**Staleness.** The stream file header includes the `modified_at` timestamp from the source `.orb` file. The viewer compares this against the `orb_meta` `modified_at` value (fetched via range request from the `.orb` file) to detect stale stream files. If the timestamps differ, the viewer discards the stream file and falls back to direct SQLite access.

The `.orb.stream` format is a performance optimization, not a data format. It carries no data that is not present in the `.orb` file. Implementations MUST NOT write `.orb.stream` files as a substitute for `.orb` files, and MUST NOT require `.orb.stream` files for any operation.

---

## 8. Import and Export

### 8.1 SketchUp (.skp) Import

SketchUp import is a first-class requirement. The conversion pipeline uses Trimble's free SketchUp C SDK to read `.skp` files and maps SketchUp concepts to Orb entities:

| SketchUp Concept | Orb Mapping |
|---|---|
| Group | `orb_entities` with `entity_type = "group"` |
| Component Definition | `orb_component_defs` (script is empty; geometry-only) |
| Component Instance | `orb_entities` with `entity_type = "component_instance"` |
| Face / Edge geometry | Tessellated to `orb_geometry_mesh`; no B-Rep conversion |
| Material | `orb_materials` with PBR defaults (`metallic=0`, `roughness=0.8`) |
| Texture | `orb_textures` (re-encoded to WebP) |
| Layer / Tag | `orb_layers` |
| Scene (saved view) | `orb_saved_views` |
| Section Plane | `orb_entities` with `entity_type = "section_plane"` |
| Dimension / Label | `orb_entities` with `entity_type = "annotation"` |
| Guide Line / Point | `orb_entities` with `entity_type = "guide"` |

The `import_source` meta key records the original `.skp` filename and SketchUp version for provenance tracking. SketchUp's up axis is Y-up; the importer applies a coordinate transformation if the Orb file uses Z-up (the default). All imported entities are tagged with `source_unit = "in"` (SketchUp's native unit) to enable accurate Imperial display.

Imported `.skp` geometry does not generate B-Rep data (and therefore has no `brep_step` encoding), since SketchUp's internal representation (a boundary mesh, not true B-Rep with analytical surfaces) cannot be losslessly converted. The imported mesh geometry is directly editable through Orbit's direct modeling tools, which generate B-Rep data (with both native and STEP encodings) on first modification.

### 8.2 IFC Export

Orb files with BIM classifications can be exported to IFC4 (ISO 16739-1:2018). The export maps `orb_classifications` `system="IFC4"` rows to IFC entity types and property sets. Geometry is exported as exact B-Rep from `orb_geometry_brep.brep_step` (preferred, since it is already in an ISO-standard encoding), or as tessellated B-Rep (`IfcFacetedBrep`) from `orb_geometry_mesh` if no B-Rep data exists. Material specification data from the `spec_*` columns in `orb_materials` is mapped to IFC material properties where applicable.

### 8.3 Other Formats

The following import/export capabilities are planned for v1.x but are not part of the v1.0 specification:

- **glTF 2.0 export.** For rendering, visualization, and AR/VR consumption. The PBR material model maps directly to glTF's metallic-roughness model.
- **STEP AP214 import/export.** For mechanical CAD interoperability. Requires B-Rep geometry. Import can populate `orb_geometry_brep.brep_step` directly.
- **DXF/DWG import.** For legacy AutoCAD data. DXF is open; DWG requires the Open Design Alliance libraries.
- **STL/OBJ/3MF export.** For 3D printing and fabrication workflows.

---

## 9. Performance Considerations

### 9.1 SQLite Configuration

Orb writers SHOULD configure the following SQLite pragmas when creating `.orb` files for optimal read performance:

```sql
PRAGMA page_size = 4096;              -- optimal for range-request alignment
PRAGMA journal_mode = WAL;            -- write-ahead logging for crash safety
PRAGMA foreign_keys = ON;             -- enforce referential integrity
PRAGMA application_id = 0x4F524231;   -- "ORB1" identification
```

When finalizing a file for distribution (see [Section 3.4](#34-editing-session-model)), writers SHOULD perform a WAL checkpoint, run `VACUUM` to defragment the database, run `ANALYZE` to update query planner statistics, and set `journal_mode` back to `DELETE`. This significantly improves the web viewer's fetch efficiency by consolidating related data onto adjacent pages.

### 9.2 Geometry Budget Guidelines

While the format imposes no hard limits on geometry size, the following guidelines help implementations provide good performance:

- Single entity meshes should stay under 500,000 triangles. Larger meshes should be split into multiple entities for efficient frustum culling via the spatial index and progressive loading.
- Total file size should be considered relative to the web viewing use case. A 100 MB `.orb` file is viable for desktop editing but may strain web viewing on slow connections. The companion stream format ([Section 7.4](#74-companion-stream-format)) mitigates this significantly, but initial load time is still affected by total scene complexity.
- Texture resolution should be capped at 4096x4096 pixels for individual textures. Higher resolutions provide diminishing visual returns in architectural visualization and dramatically increase file and memory usage.
- Component instancing should be used aggressively. A model with 500 identical chairs should have one component definition and 500 lightweight instances (each ~200 bytes), not 500 copies of the geometry (each ~500 KB).

### 9.3 Indexing Strategy

The schema defines several indexes that are critical for query performance. Writers MUST NOT omit these indexes:

- `idx_entities_parent` enables efficient tree traversal (finding all children of a group).
- `idx_entities_layer` enables layer-based visibility toggling without a full table scan.
- `idx_entities_type` enables type-filtered queries (e.g., finding all `component_instance` entities).
- `idx_comp_inst_def` enables finding all instances of a given component definition for batch updates.
- `idx_class_system` enables BIM queries filtered by classification system and class.
- `idx_selset_entity` enables lookup of selection set membership for a given entity.
- `idx_occupancy_system` enables system-versus-system clash analysis (e.g., querying all structural entities to check against a new mechanical entity).
- `idx_clash_status` enables efficient retrieval of active/unresolved clashes for display and reporting.
- `idx_clash_entity_a` and `idx_clash_entity_b` enable efficient lookup of all clashes involving a selected entity.
- `orb_spatial_index` (R-tree) enables bounding box queries for frustum culling, hit testing, proximity search, and spatial integrity enforcement. This is the most performance-critical index for interactive applications — it serves both the rendering pipeline (frustum culling) and the modeling pipeline (clash detection broad-phase).

---

## 10. Conformance Requirements

This section defines the minimum requirements for conforming Orb readers and writers. Conformance levels allow lightweight tools (viewers, converters) to implement a subset of the specification without claiming full support.

### 10.1 Conformance Level 1: Minimal Reader

A Level 1 reader can display the visual representation of an Orb file. It MUST:

1. Verify the SQLite `application_id` pragma equals `0x4F524231`.
2. Read and validate `format_version` from `orb_meta`, rejecting unrecognized major versions.
3. Read the `orb_entities` table and reconstruct the scene graph tree.
4. Compute world-space transforms by multiplying the parent chain.
5. Read `orb_geometry_mesh` and render triangle meshes with vertex positions and normals.
6. Read `orb_materials` and apply `base_color` to meshes.
7. Respect entity and layer visibility flags.
8. Read `orb_saved_views` and present named viewpoints to the user.
9. Read `orb_spatial_index` for frustum culling and hit testing, if performing interactive rendering.
10. Display annotation entities using their cached mesh representation from `orb_geometry_mesh` without attempting to interpret annotation content.
11. MAY read `orb_clash_results` to display clash markers as visual overlays (highlighting entities with active clashes), without performing clash detection itself.
12. Ignore all unrecognized tables, columns, and meta keys.

The Orb web viewer is a Level 1 reader.

### 10.2 Conformance Level 2: Component-Aware Reader

A Level 2 reader adds parametric component support. In addition to Level 1, it MUST:

1. Read `orb_component_defs` and `orb_component_instances`.
2. Check `script_lang` before executing any component script. If the language version is not supported, fall back to cached mesh geometry and present a warning.
3. Display the parameter panel for selected component instances.
4. Regenerate mesh geometry when parameters change by executing the component script.
5. Update `cache_valid` and `cache_hash` after regeneration.
6. Read and apply `orb_textures` to materials.
7. Read and display `orb_selection_sets`.

### 10.3 Conformance Level 3: Full Reader/Writer

A Level 3 implementation is a full CAD application. In addition to Level 2, it MUST:

1. Read and write `orb_geometry_brep` for exact geometry editing, including both `brep_data` (kernel-native) and `brep_step` (portable STEP AP214) columns.
2. Retessellate B-Rep geometry to update `orb_geometry_mesh` when geometry changes.
3. Maintain the `orb_spatial_index` and `orb_entity_rowids` tables, updating bounding boxes whenever entity geometry or transforms change.
4. Read and write `orb_classifications` for BIM data.
5. Support SketchUp (`.skp`) import as defined in [Section 8.1](#81-sketchup-skp-import).
6. Support IFC4 export for classified models, preferring `brep_step` data for geometry.
7. Maintain referential integrity across all tables.
8. Set the `application_id` pragma and write all required `orb_meta` keys on save.
9. Support both editing and distribution file modes as described in [Section 3.4](#34-editing-session-model).
10. Generate `.orb.stream` companion files when exporting for web distribution.
11. Enforce spatial integrity as described in [Section 4.13.5](#4135-spatial-integrity-check-protocol). Every geometry-modifying operation MUST run the broad-phase/narrow-phase/clearance check protocol against the spatial index and occupancy data.
12. Read and write `orb_occupancy` for all entities, populating `occupancy_type`, `system`, and `priority` based on entity classification. Populate `clearance_data` for all entities with known functional clearance requirements.
13. Maintain `orb_clash_results`, recording detected clashes and updating their status as geometry changes resolve or introduce conflicts.

The Orbit desktop application is a Level 3 reader/writer.

---

## 11. Security Considerations

Because Orb files contain executable script code (in `orb_component_defs`) and may be received from untrusted sources, implementations must consider the following security aspects.

### 11.1 Script Sandboxing

Component scripts execute in a sandboxed environment with no access to the file system, network, or system APIs. The Rhai scripting runtime used by the reference implementation provides this sandboxing by default: scripts can only call functions explicitly registered by the host application. The registered API is limited to geometry construction, mathematical operations, and parameter access. Scripts cannot read or write files, make network requests, execute system commands, or access memory outside their sandbox.

Implementations that use a different scripting runtime MUST provide equivalent sandboxing guarantees. A script in an Orb file must never be able to affect anything outside the model it belongs to.

### 11.2 Resource Limits

Malicious scripts could attempt denial-of-service by generating excessive geometry or entering infinite loops. Implementations MUST enforce:

- **Execution time limit.** Script execution should be capped at a configurable timeout (recommended default: 5 seconds per component instance).
- **Memory limit.** Script execution should be capped at a configurable memory budget (recommended default: 256 MB per component instance).
- **Geometry output limit.** The maximum number of vertices and triangles produced by a single script execution should be bounded (recommended default: 2 million triangles).

### 11.3 BLOB Validation

Readers MUST validate BLOB data before processing:

1. Verify that `positions` BLOB length is divisible by 12 (3 x f32).
2. Verify that `normals` BLOB length equals `positions` BLOB length.
3. Verify that all indices in the `indices` BLOB are less than the vertex count.
4. Verify that edge indices are less than the vertex count.
5. Verify that transform BLOBs are exactly 128 bytes.
6. Verify that texture data BLOBs begin with valid PNG, WebP, or JPEG headers.
7. Verify that `face_materials` palette count is consistent with the BLOB length and triangle count.

Failure to validate may result in out-of-bounds reads, GPU driver crashes, or memory corruption.

---

## 12. Future Directions

The following features are explicitly deferred from v1.0 but anticipated in future minor versions. The schema has been designed with these extensions in mind, and implementors should be aware of them when making architectural decisions.

**Collaboration and Change Tracking.** Future versions may add tables for tracking change history, supporting undo/redo persistence across sessions, and enabling branch-merge workflows for collaborative design. The use of UUIDv7 identifiers for all entities is a prerequisite for meaningful diff and merge operations, as two independently-created entities will never share an ID.

**Annotation Schema (v1.2).** The full annotation specification will define associative dimension types (linear, angular, radial, ordinate), text notes, leaders, dimension styles, tolerance notation, and reference geometry binding. Annotation data will be stored in core tables rather than application-specific extensions, enabling interoperable construction documentation workflows and IFC annotation export.

**External Reference Resolution (v1.2+).** The `orb_external_refs` table defined in [Section 4.12](#412-external-references-orb_external_refs) will be activated with a full resolution protocol: discovery, version matching, authentication, conflict handling, and incremental synchronization. This is the prerequisite for multi-file project workflows.

**Level of Detail.** The `lod_level` column in `orb_geometry_mesh` anticipates multiple tessellation levels per entity. A future version may define the LOD selection protocol, allowing the web viewer to request coarser meshes for distant objects and refine as the user zooms in.

**Advanced PBR Materials.** The material model may be extended with clearcoat, subsurface scattering, anisotropy, and transmission properties, aligning with the glTF 2.0 extensions ecosystem (`KHR_materials_clearcoat`, `KHR_materials_transmission`, etc.).

**Geospatial Integration.** The `geo_origin` fields in `orb_meta` provide a foundation for future GIS integration. A georeferenced Orb model could be placed on a map, aligned with terrain data, or combined with other geo-located models in a site context.

**Animation and Phasing.** Construction sequencing and design option phasing may be supported through a timeline table that associates entities with temporal visibility ranges. This would enable 4D construction visualization directly from the Orb format.

**Multi-Discipline Clash Federation.** The v1.0 spatial integrity engine operates within a single `.orb` file. Future versions, in conjunction with the external reference resolution protocol ([Section 4.12](#412-external-references-orb_external_refs)), will extend clash detection across federated multi-file models — enabling the architect's model to be checked against the structural engineer's model and the MEP engineer's model without requiring a separate aggregation tool. The `orb_clash_results` table schema is designed to accommodate cross-file clash references in future versions.

**Expanded Clearance Libraries.** Future versions may include jurisdiction-specific clearance envelope libraries (ADA accessibility clearances, IBC egress clearances, NEC electrical clearances) that automatically populate `clearance_data` based on entity classification and the active code profile.

---

## Appendix A: Complete Schema Reference

The following is the complete SQL schema for an Orb v1.0 file, suitable for direct execution against a new SQLite database.

```sql
-- ============================================================
-- Orb File Format v1.0 Schema
-- Copyright (c) 2026 Godspeed Systems LLC
-- Released under open specification license
-- ============================================================

PRAGMA page_size = 4096;
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA application_id = 0x4F524231;  -- ASCII "ORB1"

-- ------------------------------------------------------------
-- Document Metadata
-- ------------------------------------------------------------
CREATE TABLE orb_meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- ------------------------------------------------------------
-- Layers
-- ------------------------------------------------------------
CREATE TABLE orb_layers (
    id         BLOB PRIMARY KEY NOT NULL,
    name       TEXT NOT NULL,
    color      TEXT,
    visible    INTEGER NOT NULL DEFAULT 1,
    locked     INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- ------------------------------------------------------------
-- Scene Graph
-- ------------------------------------------------------------
CREATE TABLE orb_entities (
    id          BLOB PRIMARY KEY NOT NULL,
    parent_id   BLOB REFERENCES orb_entities(id)
                     ON DELETE CASCADE,
    name        TEXT,
    entity_type TEXT NOT NULL,
    transform   BLOB NOT NULL,
    visible     INTEGER NOT NULL DEFAULT 1,
    locked      INTEGER NOT NULL DEFAULT 0,
    layer_id    BLOB REFERENCES orb_layers(id),
    source_unit TEXT,
    created_at  TEXT NOT NULL,
    modified_at TEXT NOT NULL
);

CREATE INDEX idx_entities_parent ON orb_entities(parent_id);
CREATE INDEX idx_entities_layer  ON orb_entities(layer_id);
CREATE INDEX idx_entities_type   ON orb_entities(entity_type);

-- ------------------------------------------------------------
-- Spatial Index (R-tree)
-- ------------------------------------------------------------
CREATE TABLE orb_entity_rowids (
    rowid     INTEGER PRIMARY KEY,
    entity_id BLOB NOT NULL UNIQUE
              REFERENCES orb_entities(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE orb_spatial_index USING rtree(
    id,
    min_x, max_x,
    min_y, max_y,
    min_z, max_z
);

-- ------------------------------------------------------------
-- Textures
-- ------------------------------------------------------------
CREATE TABLE orb_textures (
    id     BLOB PRIMARY KEY NOT NULL,
    name   TEXT,
    format TEXT NOT NULL,
    width  INTEGER NOT NULL,
    height INTEGER NOT NULL,
    data   BLOB NOT NULL
);

-- ------------------------------------------------------------
-- Materials
-- ------------------------------------------------------------
CREATE TABLE orb_materials (
    id                     BLOB PRIMARY KEY NOT NULL,
    name                   TEXT NOT NULL,
    base_color             TEXT NOT NULL DEFAULT 'CCCCCC',
    base_color_tex         BLOB REFERENCES orb_textures(id),
    metallic               REAL NOT NULL DEFAULT 0.0,
    roughness              REAL NOT NULL DEFAULT 0.5,
    metallic_roughness_tex BLOB REFERENCES orb_textures(id),
    normal_tex             BLOB REFERENCES orb_textures(id),
    normal_scale           REAL NOT NULL DEFAULT 1.0,
    emissive_color         TEXT DEFAULT '000000',
    emissive_tex           BLOB REFERENCES orb_textures(id),
    opacity                REAL NOT NULL DEFAULT 1.0,
    double_sided           INTEGER NOT NULL DEFAULT 0,
    spec_name              TEXT,
    spec_manufacturer      TEXT,
    spec_product_id        TEXT,
    spec_url               TEXT,
    spec_properties        TEXT
);

-- ------------------------------------------------------------
-- Mesh Geometry (display-ready)
-- ------------------------------------------------------------
CREATE TABLE orb_geometry_mesh (
    entity_id      BLOB PRIMARY KEY NOT NULL
                   REFERENCES orb_entities(id) ON DELETE CASCADE,
    positions      BLOB NOT NULL,
    normals        BLOB NOT NULL,
    indices        BLOB NOT NULL,
    edges          BLOB,
    uv0            BLOB,
    lod_level      INTEGER NOT NULL DEFAULT 0,
    face_materials BLOB
);

-- ------------------------------------------------------------
-- B-Rep Geometry (editing kernel)
-- ------------------------------------------------------------
CREATE TABLE orb_geometry_brep (
    entity_id   BLOB PRIMARY KEY NOT NULL
                REFERENCES orb_entities(id) ON DELETE CASCADE,
    kernel      TEXT NOT NULL,
    brep_data   BLOB NOT NULL,
    brep_format TEXT NOT NULL,
    brep_step   BLOB,
    tess_params TEXT
);

-- ------------------------------------------------------------
-- Component Definitions
-- ------------------------------------------------------------
CREATE TABLE orb_component_defs (
    id           BLOB PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    category     TEXT,
    description  TEXT,
    script_lang  TEXT NOT NULL DEFAULT 'orbit-script-1.0',
    script       TEXT NOT NULL,
    script_hash  TEXT NOT NULL,
    param_schema TEXT NOT NULL,
    thumbnail    BLOB,
    author       TEXT,
    license      TEXT,
    version      TEXT,
    created_at   TEXT NOT NULL,
    modified_at  TEXT NOT NULL
);

-- ------------------------------------------------------------
-- Component Instances
-- ------------------------------------------------------------
CREATE TABLE orb_component_instances (
    entity_id    BLOB PRIMARY KEY NOT NULL
                 REFERENCES orb_entities(id) ON DELETE CASCADE,
    def_id       BLOB NOT NULL
                 REFERENCES orb_component_defs(id),
    param_values TEXT NOT NULL,
    cache_valid  INTEGER NOT NULL DEFAULT 0,
    cache_hash   TEXT
);

CREATE INDEX idx_comp_inst_def ON orb_component_instances(def_id);

-- ------------------------------------------------------------
-- Saved Views
-- ------------------------------------------------------------
CREATE TABLE orb_saved_views (
    id               BLOB PRIMARY KEY NOT NULL,
    name             TEXT NOT NULL,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    eye_x            REAL NOT NULL,
    eye_y            REAL NOT NULL,
    eye_z            REAL NOT NULL,
    target_x         REAL NOT NULL,
    target_y         REAL NOT NULL,
    target_z         REAL NOT NULL,
    up_x             REAL NOT NULL DEFAULT 0.0,
    up_y             REAL NOT NULL DEFAULT 0.0,
    up_z             REAL NOT NULL DEFAULT 1.0,
    fov              REAL,
    ortho_width      REAL,
    layer_visibility TEXT,
    section_plane_id BLOB REFERENCES orb_entities(id),
    description      TEXT,
    thumbnail        BLOB
);

-- ------------------------------------------------------------
-- Selection Sets
-- ------------------------------------------------------------
CREATE TABLE orb_selection_sets (
    id          BLOB PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT,
    color       TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE orb_selection_set_members (
    set_id    BLOB NOT NULL REFERENCES orb_selection_sets(id) ON DELETE CASCADE,
    entity_id BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    PRIMARY KEY (set_id, entity_id)
);

CREATE INDEX idx_selset_entity ON orb_selection_set_members(entity_id);

-- ------------------------------------------------------------
-- BIM Classifications
-- ------------------------------------------------------------
CREATE TABLE orb_classifications (
    entity_id  BLOB NOT NULL
               REFERENCES orb_entities(id) ON DELETE CASCADE,
    system     TEXT NOT NULL,
    class      TEXT NOT NULL,
    pset_name  TEXT,
    properties TEXT NOT NULL,
    PRIMARY KEY (entity_id, system, pset_name)
);

CREATE INDEX idx_class_system ON orb_classifications(system, class);

-- ------------------------------------------------------------
-- External References (reserved, non-functional in v1.0)
-- ------------------------------------------------------------
CREATE TABLE orb_external_refs (
    id          BLOB PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    ref_type    TEXT NOT NULL,
    uri         TEXT NOT NULL,
    format      TEXT NOT NULL,
    transform   BLOB,
    status      TEXT NOT NULL DEFAULT 'unresolved',
    last_synced TEXT,
    properties  TEXT
);

-- ------------------------------------------------------------
-- Spatial Occupancy
-- ------------------------------------------------------------
CREATE TABLE orb_occupancy (
    entity_id       BLOB PRIMARY KEY NOT NULL
                    REFERENCES orb_entities(id) ON DELETE CASCADE,
    occupancy_type  TEXT NOT NULL DEFAULT 'solid',
    clearance_data  BLOB,
    priority        INTEGER NOT NULL DEFAULT 100,
    system          TEXT
);

CREATE INDEX idx_occupancy_system ON orb_occupancy(system);

-- ------------------------------------------------------------
-- Clash Detection Results
-- ------------------------------------------------------------
CREATE TABLE orb_clash_results (
    id                   BLOB PRIMARY KEY NOT NULL,
    entity_a             BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    entity_b             BLOB NOT NULL REFERENCES orb_entities(id) ON DELETE CASCADE,
    clash_type           TEXT NOT NULL,
    severity             TEXT NOT NULL,
    system_a             TEXT,
    system_b             TEXT,
    intersection_point_x REAL,
    intersection_point_y REAL,
    intersection_point_z REAL,
    distance             REAL,
    status               TEXT NOT NULL DEFAULT 'active',
    resolved_by          TEXT,
    detected_at          TEXT NOT NULL,
    resolved_at          TEXT,
    UNIQUE(entity_a, entity_b, clash_type)
);

CREATE INDEX idx_clash_status ON orb_clash_results(status);
CREATE INDEX idx_clash_entity_a ON orb_clash_results(entity_a);
CREATE INDEX idx_clash_entity_b ON orb_clash_results(entity_b);
```

---

## Appendix B: MIME Type and File Association

| Property | Value |
|---|---|
| File extension | `.orb` |
| MIME type | `application/vnd.orbit.orb` |
| UTI (macOS/iOS) | `com.godspeed.orbit.orb` |
| SQLite application_id | `0x4F524231` (`"ORB1"`) |
| Magic bytes | Standard SQLite header: `"SQLite format 3\000"` at offset 0 |
| Companion stream extension | `.orb.stream` |
| Companion stream MIME type | `application/vnd.orbit.orb-stream` |

Operating systems identify Orb files first by extension, then by MIME type. The SQLite magic bytes provide a third identification layer. The `application_id` pragma provides definitive identification once the file is opened by a SQLite-aware tool.

---

## Appendix C: Glossary

**B-Rep (Boundary Representation).** A method for representing 3D shapes using their boundary surfaces (faces, edges, vertices) and the topological relationships between them. Supports exact analytical geometry (planes, cylinders, NURBS surfaces).

**BIM (Building Information Modeling).** A process for creating and managing digital representations of physical and functional characteristics of buildings. BIM data in Orb is carried by the `orb_classifications` table.

**Clearance Envelope.** The functional space around an entity beyond its physical geometry, required for the entity to be usable. A door's clearance envelope includes its swing arc; a toilet's includes frontal approach space.

**Clash (Hard Clash).** A spatial conflict where two solid entities occupy the same physical space — a physical impossibility that must be resolved. Distinguished from clearance violations, which are functional rather than physical conflicts.

**CSG (Constructive Solid Geometry).** A technique for creating complex shapes by combining primitive shapes using Boolean operations (union, intersection, difference).

**ECS (Entity Component System).** A software architectural pattern common in game engines that composes objects from independent data components rather than class hierarchies.

**GJK (Gilbert-Johnson-Keerthi).** An algorithm for computing the minimum distance between two convex shapes, used in the narrow-phase of spatial integrity checking.

**IFC (Industry Foundation Classes).** An ISO standard (16739) for BIM data exchange. Orb supports IFC4 classification and export.

**LOD (Level of Detail).** Multiple geometric representations of the same object at different fidelity levels, used to optimize rendering performance.

**PBR (Physically Based Rendering).** A rendering approach that models the physical interaction of light with materials. Orb uses the metallic-roughness PBR model.

**R-tree.** A tree data structure for spatial indexing of multi-dimensional data. Used by `orb_spatial_index` for axis-aligned bounding box queries.

**STEP (Standard for the Exchange of Product Model Data).** An ISO standard (10303) for representing 3D product data. Orb uses STEP AP214 encoding in the `brep_step` column for kernel-independent B-Rep storage.

**Tessellation.** The process of converting analytical B-Rep geometry into a triangulated mesh suitable for GPU rendering.

**UUIDv7.** A universally unique identifier format (RFC 9562) that encodes a Unix millisecond timestamp in its most significant bits, providing time-ordered uniqueness.

**VFS (Virtual File System).** SQLite's I/O abstraction layer that can be implemented to read database pages from arbitrary backing stores, including HTTP range requests.

**WAL (Write-Ahead Logging).** A SQLite journaling mode that provides atomic transactions and crash recovery by writing changes to a separate log file before modifying the main database.

**WASM (WebAssembly).** A binary instruction format for a stack-based virtual machine, enabling near-native performance for code running in web browsers.

**wgpu.** A Rust-native graphics library that implements the WebGPU API, providing a unified rendering interface across desktop (Vulkan, Metal, DX12) and web (WebGPU) targets.
