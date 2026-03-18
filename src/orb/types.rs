use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::transform::Transform;
use super::uuid::OrbId;

// --- Entity Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Body,
    Group,
    ComponentInstance,
    SectionPlane,
    Annotation,
    Guide,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Body => write!(f, "body"),
            Self::Group => write!(f, "group"),
            Self::ComponentInstance => write!(f, "component_instance"),
            Self::SectionPlane => write!(f, "section_plane"),
            Self::Annotation => write!(f, "annotation"),
            Self::Guide => write!(f, "guide"),
        }
    }
}

impl FromStr for EntityType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "body" => Ok(Self::Body),
            "group" => Ok(Self::Group),
            "component_instance" => Ok(Self::ComponentInstance),
            "section_plane" => Ok(Self::SectionPlane),
            "annotation" => Ok(Self::Annotation),
            "guide" => Ok(Self::Guide),
            _ => Err(format!("unknown entity type: {s}")),
        }
    }
}

// --- Display Unit ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayUnit {
    Mm,
    Cm,
    M,
    In,
    Ft,
}

impl fmt::Display for DisplayUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mm => write!(f, "mm"),
            Self::Cm => write!(f, "cm"),
            Self::M => write!(f, "m"),
            Self::In => write!(f, "in"),
            Self::Ft => write!(f, "ft"),
        }
    }
}

impl FromStr for DisplayUnit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mm" => Ok(Self::Mm),
            "cm" => Ok(Self::Cm),
            "m" => Ok(Self::M),
            "in" => Ok(Self::In),
            "ft" => Ok(Self::Ft),
            _ => Err(format!("unknown display unit: {s}")),
        }
    }
}

// --- Up Axis ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpAxis {
    Z,
    Y,
}

impl fmt::Display for UpAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Z => write!(f, "z"),
            Self::Y => write!(f, "y"),
        }
    }
}

impl FromStr for UpAxis {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "z" => Ok(Self::Z),
            "y" => Ok(Self::Y),
            _ => Err(format!("unknown up axis: {s}")),
        }
    }
}

// --- Occupancy ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyType {
    Solid,
    Penetrable,
    Reservation,
}

impl fmt::Display for OccupancyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Solid => write!(f, "solid"),
            Self::Penetrable => write!(f, "penetrable"),
            Self::Reservation => write!(f, "reservation"),
        }
    }
}

impl FromStr for OccupancyType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "solid" => Ok(Self::Solid),
            "penetrable" => Ok(Self::Penetrable),
            "reservation" => Ok(Self::Reservation),
            _ => Err(format!("unknown occupancy type: {s}")),
        }
    }
}

// --- Building System ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingSystem {
    Structural,
    Architectural,
    Mechanical,
    Plumbing,
    Electrical,
    FireProtection,
    Furniture,
}

impl fmt::Display for BuildingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structural => write!(f, "structural"),
            Self::Architectural => write!(f, "architectural"),
            Self::Mechanical => write!(f, "mechanical"),
            Self::Plumbing => write!(f, "plumbing"),
            Self::Electrical => write!(f, "electrical"),
            Self::FireProtection => write!(f, "fire_protection"),
            Self::Furniture => write!(f, "furniture"),
        }
    }
}

impl FromStr for BuildingSystem {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "structural" => Ok(Self::Structural),
            "architectural" => Ok(Self::Architectural),
            "mechanical" => Ok(Self::Mechanical),
            "plumbing" => Ok(Self::Plumbing),
            "electrical" => Ok(Self::Electrical),
            "fire_protection" => Ok(Self::FireProtection),
            "furniture" => Ok(Self::Furniture),
            _ => Err(format!("unknown building system: {s}")),
        }
    }
}

// --- Clash Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClashType {
    Hard,
    Clearance,
    Penetration,
}

impl fmt::Display for ClashType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hard => write!(f, "hard"),
            Self::Clearance => write!(f, "clearance"),
            Self::Penetration => write!(f, "penetration"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClashSeverity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for ClashSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClashStatus {
    Active,
    Resolved,
    Approved,
    Ignored,
}

impl fmt::Display for ClashStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Resolved => write!(f, "resolved"),
            Self::Approved => write!(f, "approved"),
            Self::Ignored => write!(f, "ignored"),
        }
    }
}

// --- Data Structs ---

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: OrbId,
    pub parent_id: Option<OrbId>,
    pub name: Option<String>,
    pub entity_type: EntityType,
    pub transform: Transform,
    pub visible: bool,
    pub locked: bool,
    pub layer_id: Option<OrbId>,
    pub source_unit: Option<String>,
    pub created_at: String,
    pub modified_at: String,
}

impl Entity {
    pub fn new(entity_type: EntityType) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: OrbId::new(),
            parent_id: None,
            name: None,
            entity_type,
            transform: Transform::identity(),
            visible: true,
            locked: false,
            layer_id: None,
            source_unit: None,
            created_at: now.clone(),
            modified_at: now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: OrbId,
    pub name: String,
    pub color: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub id: OrbId,
    pub name: String,
    pub base_color: String,
    pub metallic: f64,
    pub roughness: f64,
    pub opacity: f64,
    pub double_sided: bool,
}

impl Material {
    pub fn new(name: &str, base_color: &str) -> Self {
        Self {
            id: OrbId::new(),
            name: name.to_string(),
            base_color: base_color.to_string(),
            metallic: 0.0,
            roughness: 0.5,
            opacity: 1.0,
            double_sided: false,
        }
    }

    /// Parse hex RGB to [f32; 3] in 0..1 range.
    pub fn base_color_rgb(&self) -> [f32; 3] {
        let hex = &self.base_color;
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(204) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(204) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(204) as f32 / 255.0;
        [r, g, b]
    }
}
