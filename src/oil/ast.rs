use super::types::*;

/// Top-level program: either a house or a furniture definition.
#[derive(Debug, Clone)]
pub enum Program {
    House(HouseBlock),
    Furniture(FurnitureBlock),
}

#[derive(Debug, Clone)]
pub struct HouseBlock {
    pub name: Option<String>,
    pub site: Option<SiteBlock>,
    pub style: Option<StyleBlock>,
    pub floors: Vec<FloorBlock>,
    pub roof: Option<RoofBlock>,
    pub facades: Vec<FacadeBlock>,
    pub landscape: Option<LandscapeBlock>,
}

#[derive(Debug, Clone)]
pub struct SiteBlock {
    pub footprint: Option<(Dimension, Dimension)>,
    pub orientation: Option<Cardinal>,
    pub setbacks: Vec<(String, Dimension)>,
    pub slope: Option<String>,
    pub garage_access: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub name: String,
    pub parent: Option<String>,
    pub overrides: Vec<StyleProperty>,
}

/// A style property: key + structured value.
#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub key: String,
    pub value: StyleValue,
}

/// Typed style values — we parse what we can, keep the rest as text.
#[derive(Debug, Clone)]
pub enum StyleValue {
    Material(MaterialSpec),
    Pitch(Pitch),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct FloorBlock {
    pub name: String,
    pub ceiling_height: Option<Dimension>,
    pub rooms: Vec<RoomBlock>,
}

#[derive(Debug, Clone)]
pub struct RoomBlock {
    pub name: String,
    pub area: Option<ApproxValue>,
    pub aspect: Option<ApproxValue>,
    pub adjacent_to: Vec<String>,
    pub connects: Vec<String>,
    pub windows: Vec<WindowSpec>,
    pub features: Vec<Feature>,
    pub side: Option<Cardinal>,
    pub ceiling: Option<CeilingType>,
    pub flooring: Option<MaterialSpec>,
    pub purpose: Option<RoomType>,
}

#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub direction: Cardinal,
    pub count: u32,
}

/// Structured roof block with parsed sub-elements.
#[derive(Debug, Clone)]
pub struct RoofBlock {
    pub primary: Option<RoofPrimary>,
    pub cross_gable: Option<CrossGableSpec>,
    pub dormers: Option<DormerSpec>,
    pub material: Option<MaterialSpec>,
    pub pitch: Option<Pitch>,
    pub overhang: Option<Dimension>,
}

/// Primary roof form with optional parameters.
#[derive(Debug, Clone)]
pub struct RoofPrimary {
    pub form: RoofForm,
    pub params: Vec<(String, String)>,
}

/// Cross-gable specification.
#[derive(Debug, Clone)]
pub struct CrossGableSpec {
    pub over: Option<String>,
    pub pitch: Option<Pitch>,
}

/// Dormer specification.
#[derive(Debug, Clone)]
pub struct DormerSpec {
    pub count: u32,
    pub over: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FacadeBlock {
    pub side: String,
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct LandscapeBlock {
    pub properties: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct FurnitureBlock {
    pub name: String,
    pub properties: Vec<(String, String)>,
}
