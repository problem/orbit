use crate::units::measurement::{Unit, Measurement, MeasurementError};

/// Convert a value from one unit to another
pub fn convert_value(value: f32, from_unit: Unit, to_unit: Unit) -> f32 {
    // First convert to meters (base unit)
    let meters = match from_unit {
        Unit::Meters => value,
        Unit::Centimeters => value / 100.0,
        Unit::Feet => value * 0.3048,
        Unit::Inches => value * 0.0254,
    };

    // Then convert from meters to target unit
    match to_unit {
        Unit::Meters => meters,
        Unit::Centimeters => meters * 100.0,
        Unit::Feet => meters / 0.3048,
        Unit::Inches => meters / 0.0254,
    }
}

/// Parse a string like "12.5 cm" or "3 ft" into a Measurement
pub fn parse_measurement(input: &str) -> Result<Measurement, MeasurementError> {
    let input = input.trim();

    // Try to split on space
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(MeasurementError::InvalidValue(format!("Invalid format: {}", input)));
    }

    // Parse value
    let value = match parts[0].parse::<f32>() {
        Ok(v) => v,
        Err(_) => return Err(MeasurementError::InvalidValue(format!("Invalid number: {}", parts[0]))),
    };

    // Parse unit
    let unit = match parts[1].to_lowercase().as_str() {
        "cm" | "centimeter" | "centimeters" => Unit::Centimeters,
        "m" | "meter" | "meters" => Unit::Meters,
        "ft" | "foot" | "feet" => Unit::Feet,
        "in" | "inch" | "inches" => Unit::Inches,
        _ => return Err(MeasurementError::InvalidValue(format!("Unknown unit: {}", parts[1]))),
    };

    Measurement::new(value, unit)
}

/// Format a measurement with its unit
pub fn format_measurement(measurement: &Measurement) -> String {
    format!("{:.2} {}", measurement.value(), measurement.display_unit())
}
