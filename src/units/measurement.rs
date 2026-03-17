use std::fmt;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Meters,
    Centimeters,
    Feet,
    Inches,
}

impl Default for Unit {
    fn default() -> Self {
        Unit::Meters
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Meters => write!(f, "m"),
            Unit::Centimeters => write!(f, "cm"),
            Unit::Feet => write!(f, "ft"),
            Unit::Inches => write!(f, "in"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Value in meters (our internal base unit)
    value_in_meters: f32,
    /// Display unit for user interaction
    display_unit: Unit,
}

#[derive(Error, Debug)]
pub enum MeasurementError {
    #[error("Invalid measurement value: {0}")]
    InvalidValue(String),
}

impl Measurement {
    /// Create a new measurement in the specified unit
    pub fn new(value: f32, unit: Unit) -> Result<Self, MeasurementError> {
        if value.is_nan() || value.is_infinite() {
            return Err(MeasurementError::InvalidValue(format!("Value {} is not valid", value)));
        }

        // Convert to meters for internal storage
        let value_in_meters = match unit {
            Unit::Meters => value,
            Unit::Centimeters => value / 100.0,
            Unit::Feet => value * 0.3048,
            Unit::Inches => value * 0.0254,
        };

        Ok(Self {
            value_in_meters,
            display_unit: unit,
        })
    }

    /// Get the value in the current display unit
    pub fn value(&self) -> f32 {
        match self.display_unit {
            Unit::Meters => self.value_in_meters,
            Unit::Centimeters => self.value_in_meters * 100.0,
            Unit::Feet => self.value_in_meters / 0.3048,
            Unit::Inches => self.value_in_meters / 0.0254,
        }
    }

    /// Get the value in meters (internal unit)
    pub fn value_in_meters(&self) -> f32 {
        self.value_in_meters
    }

    /// Change the display unit
    pub fn set_display_unit(&mut self, unit: Unit) {
        self.display_unit = unit;
    }

    /// Get the current display unit
    pub fn display_unit(&self) -> Unit {
        self.display_unit
    }
}

/// Special struct for representing dimensions with specific display units
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dimensions3D {
    pub width: Measurement,
    pub height: Measurement,
    pub depth: Measurement,
}

impl Dimensions3D {
    pub fn new(width: Measurement, height: Measurement, depth: Measurement) -> Self {
        Self { width, height, depth }
    }

    pub fn cuboid(size: f32, unit: Unit) -> Result<Self, MeasurementError> {
        let measurement = Measurement::new(size, unit)?;
        Ok(Self {
            width: measurement,
            height: measurement,
            depth: measurement,
        })
    }

    pub fn set_all_units(&mut self, unit: Unit) {
        self.width.set_display_unit(unit);
        self.height.set_display_unit(unit);
        self.depth.set_display_unit(unit);
    }
}
