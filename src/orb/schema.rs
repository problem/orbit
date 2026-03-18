use rusqlite::Connection;

pub const ORB_APPLICATION_ID: u32 = 0x4F524231; // ASCII "ORB1"
pub const ORB_FORMAT_VERSION: &str = "1.0.0";

/// Complete Orb v1.0 schema DDL.
pub const ORB_SCHEMA: &str = r#"
-- Document Metadata
CREATE TABLE orb_meta (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Layers
CREATE TABLE orb_layers (
    id         BLOB PRIMARY KEY NOT NULL,
    name       TEXT NOT NULL,
    color      TEXT,
    visible    INTEGER NOT NULL DEFAULT 1,
    locked     INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- Scene Graph
CREATE TABLE orb_entities (
    id          BLOB PRIMARY KEY NOT NULL,
    parent_id   BLOB REFERENCES orb_entities(id) ON DELETE CASCADE,
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

-- Spatial Index (R-tree)
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

-- Textures
CREATE TABLE orb_textures (
    id     BLOB PRIMARY KEY NOT NULL,
    name   TEXT,
    format TEXT NOT NULL,
    width  INTEGER NOT NULL,
    height INTEGER NOT NULL,
    data   BLOB NOT NULL
);

-- Materials
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

-- Mesh Geometry
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

-- B-Rep Geometry
CREATE TABLE orb_geometry_brep (
    entity_id   BLOB PRIMARY KEY NOT NULL
                REFERENCES orb_entities(id) ON DELETE CASCADE,
    kernel      TEXT NOT NULL,
    brep_data   BLOB NOT NULL,
    brep_format TEXT NOT NULL,
    brep_step   BLOB,
    tess_params TEXT
);

-- Component Definitions
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

-- Component Instances
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

-- Saved Views
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

-- Selection Sets
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

-- BIM Classifications
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

-- External References (reserved, non-functional in v1.0)
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

-- Spatial Occupancy
CREATE TABLE orb_occupancy (
    entity_id       BLOB PRIMARY KEY NOT NULL
                    REFERENCES orb_entities(id) ON DELETE CASCADE,
    occupancy_type  TEXT NOT NULL DEFAULT 'solid',
    clearance_data  BLOB,
    priority        INTEGER NOT NULL DEFAULT 100,
    system          TEXT
);

CREATE INDEX idx_occupancy_system ON orb_occupancy(system);

-- Clash Detection Results
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
"#;

/// Initialize a new .orb database with the full schema and required pragmas.
pub fn init_database(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("PRAGMA page_size = 4096;")?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(&format!(
        "PRAGMA application_id = {};",
        ORB_APPLICATION_ID
    ))?;
    conn.execute_batch(ORB_SCHEMA)?;
    Ok(())
}

/// Verify that a database is a valid .orb file.
pub fn verify_database(conn: &Connection) -> anyhow::Result<()> {
    // Check application_id
    let app_id: u32 = conn.pragma_query_value(None, "application_id", |row| row.get(0))?;
    if app_id != ORB_APPLICATION_ID {
        anyhow::bail!(
            "not an Orb file: application_id is 0x{:08X}, expected 0x{:08X}",
            app_id,
            ORB_APPLICATION_ID
        );
    }

    // Check format_version major version
    let version: String = conn.query_row(
        "SELECT value FROM orb_meta WHERE key = 'format_version'",
        [],
        |row| row.get(0),
    )?;
    let major: u32 = version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if major != 1 {
        anyhow::bail!(
            "unsupported Orb format major version: {} (expected 1)",
            major
        );
    }

    Ok(())
}
