use std::fmt;
use std::str::FromStr;

/// Length/area unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Mm,
    Cm,
    M,
    In,
    Ft,
    Sqm,
    Sqft,
    /// Unitless value (aspect ratios, counts).
    Unitless,
}

impl Unit {
    pub fn is_area(&self) -> bool {
        matches!(self, Self::Sqm | Self::Sqft)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mm => write!(f, "mm"),
            Self::Cm => write!(f, "cm"),
            Self::M => write!(f, "m"),
            Self::In => write!(f, "in"),
            Self::Ft => write!(f, "ft"),
            Self::Sqm => write!(f, "sqm"),
            Self::Sqft => write!(f, "sqft"),
            Self::Unitless => write!(f, ""),
        }
    }
}

impl FromStr for Unit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mm" => Ok(Self::Mm),
            "cm" => Ok(Self::Cm),
            "m" => Ok(Self::M),
            "in" => Ok(Self::In),
            "ft" => Ok(Self::Ft),
            "sqm" => Ok(Self::Sqm),
            "sqft" => Ok(Self::Sqft),
            _ => Err(format!("unknown unit: {s}")),
        }
    }
}

/// A dimensional value with unit.
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    pub value: f64,
    pub unit: Unit,
}

impl Dimension {
    pub fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }

    /// Convert to millimeters.
    pub fn to_mm(&self) -> f64 {
        match self.unit {
            Unit::Mm => self.value,
            Unit::Cm => self.value * 10.0,
            Unit::M => self.value * 1000.0,
            Unit::In => self.value * 25.4,
            Unit::Ft => self.value * 304.8,
            Unit::Sqm => self.value * 1_000_000.0,
            Unit::Sqft => self.value * 92_903.04,
            Unit::Unitless => self.value,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

/// An approximate, range, or exact value.
#[derive(Debug, Clone)]
pub enum ApproxValue {
    /// `~25sqm` — target with +/-20% tolerance
    Approximate(f64, Unit),
    /// `20sqm..30sqm` — hard bounds
    Range(f64, f64, Unit),
    /// `25sqm` — exact constraint
    Exact(f64, Unit),
    /// `large`, `small`, etc. — resolved per room type
    Qualitative(String),
}

impl fmt::Display for ApproxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Approximate(v, u) => write!(f, "~{v}{u}"),
            Self::Range(lo, hi, u) => write!(f, "{lo}{u}..{hi}{u}"),
            Self::Exact(v, u) => write!(f, "{v}{u}"),
            Self::Qualitative(q) => write!(f, "{q}"),
        }
    }
}

/// Cardinal direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinal {
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

impl fmt::Display for Cardinal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::North => write!(f, "north"),
            Self::South => write!(f, "south"),
            Self::East => write!(f, "east"),
            Self::West => write!(f, "west"),
            Self::Northeast => write!(f, "northeast"),
            Self::Northwest => write!(f, "northwest"),
            Self::Southeast => write!(f, "southeast"),
            Self::Southwest => write!(f, "southwest"),
        }
    }
}

impl FromStr for Cardinal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            "east" => Ok(Self::East),
            "west" => Ok(Self::West),
            "northeast" => Ok(Self::Northeast),
            "northwest" => Ok(Self::Northwest),
            "southeast" => Ok(Self::Southeast),
            "southwest" => Ok(Self::Southwest),
            _ => Err(format!("unknown cardinal: {s}")),
        }
    }
}

/// Recognized room type (inferred from name or explicit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Living,
    Kitchen,
    Dining,
    Bedroom,
    MasterBed,
    Bathroom,
    MasterBath,
    HalfBath,
    Entry,
    Hallway,
    Garage,
    Office,
    Laundry,
    Mudroom,
    Closet,
    Pantry,
    Studio,
    Nursery,
    Family,
    Porch,
    General,
}

impl RoomType {
    pub fn infer_from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "living" | "living_room" => Self::Living,
            "kitchen" => Self::Kitchen,
            "dining" | "dining_room" => Self::Dining,
            s if s.starts_with("master_bed") || s.starts_with("master bed") => Self::MasterBed,
            s if s.starts_with("bedroom") || s == "bed" => Self::Bedroom,
            s if s.starts_with("master_bath") || s.starts_with("master bath") => Self::MasterBath,
            s if s.starts_with("half_bath") || s.starts_with("half bath") => Self::HalfBath,
            s if s.starts_with("bathroom") || s.starts_with("full_bath") || s == "bath" => {
                Self::Bathroom
            }
            "entry" | "foyer" | "entryway" => Self::Entry,
            "hallway" | "hall" | "corridor" => Self::Hallway,
            "garage" => Self::Garage,
            "office" | "study" => Self::Office,
            "laundry" | "laundry_room" => Self::Laundry,
            "mudroom" | "mud_room" => Self::Mudroom,
            "closet" => Self::Closet,
            "pantry" => Self::Pantry,
            "studio" => Self::Studio,
            "nursery" => Self::Nursery,
            "family" | "family_room" => Self::Family,
            "porch" => Self::Porch,
            _ => Self::General,
        }
    }
}

/// Built-in fixture/feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Island,
    Pantry,
    Shower,
    Tub,
    DoubleVanity,
    Closet,
    WalkInCloset,
    Fireplace,
    FrontDoor,
    BackDoor,
    GarageSingle,
    GarageDouble,
    LaundryHookup,
    Staircase,
}

impl FromStr for Feature {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "island" => Ok(Self::Island),
            "pantry" => Ok(Self::Pantry),
            "shower" => Ok(Self::Shower),
            "tub" => Ok(Self::Tub),
            "double_vanity" => Ok(Self::DoubleVanity),
            "closet" => Ok(Self::Closet),
            "walk_in_closet" => Ok(Self::WalkInCloset),
            "fireplace" => Ok(Self::Fireplace),
            "front_door" => Ok(Self::FrontDoor),
            "back_door" => Ok(Self::BackDoor),
            "garage_single" => Ok(Self::GarageSingle),
            "garage_double" => Ok(Self::GarageDouble),
            "laundry_hookup" => Ok(Self::LaundryHookup),
            "staircase" => Ok(Self::Staircase),
            _ => Err(format!("unknown feature: {s}")),
        }
    }
}

/// Ceiling type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeilingType {
    Standard,
    Vaulted,
    Cathedral,
    Tray,
    Coffered,
}

impl FromStr for CeilingType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" => Ok(Self::Standard),
            "vaulted" => Ok(Self::Vaulted),
            "cathedral" => Ok(Self::Cathedral),
            "tray" => Ok(Self::Tray),
            "coffered" => Ok(Self::Coffered),
            _ => Err(format!("unknown ceiling type: {s}")),
        }
    }
}

/// Roof form with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoofForm {
    Gable,
    Hip,
    Flat,
    Shed,
    Gambrel,
    Mansard,
}

impl FromStr for RoofForm {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gable" => Ok(Self::Gable),
            "hip" => Ok(Self::Hip),
            "flat" => Ok(Self::Flat),
            "shed" => Ok(Self::Shed),
            "gambrel" => Ok(Self::Gambrel),
            "mansard" => Ok(Self::Mansard),
            _ => Err(format!("unknown roof form: {s}")),
        }
    }
}

/// Roof pitch as rise:run ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    pub rise: f64,
    pub run: f64,
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.rise, self.run)
    }
}

/// Material specification like `stucco("cream")`.
#[derive(Debug, Clone)]
pub struct MaterialSpec {
    pub material_type: String,
    pub color: Option<String>,
}

impl fmt::Display for MaterialSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.color {
            Some(c) => write!(f, "{}(\"{}\")", self.material_type, c),
            None => write!(f, "{}", self.material_type),
        }
    }
}
