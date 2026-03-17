use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::units::measurement::{Unit, Measurement, Dimensions3D};
use nalgebra::Vector3;

pub struct MeasurementUiPlugin;

impl Plugin for MeasurementUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MeasurementUiState>()
            .init_resource::<CreatedPrimitive>()
            .add_systems(Update, measurement_ui_system);
    }
}

#[derive(Default, Resource)]
pub struct MeasurementUiState {
    pub active_unit: Unit,
    pub width_input: String,
    pub height_input: String,
    pub depth_input: String,
    pub error_message: Option<String>,
    pub selected_primitive: PrimitiveType,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    #[default]
    Cube,
    Cylinder,
    Pyramid,
    Sphere,
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Cube => write!(f, "Cube"),
            PrimitiveType::Cylinder => write!(f, "Cylinder"),
            PrimitiveType::Pyramid => write!(f, "Pyramid"),
            PrimitiveType::Sphere => write!(f, "Sphere"),
        }
    }
}

#[derive(Resource, Default)]
pub struct CreatedPrimitive {
    pub should_create: bool,
    pub primitive_type: PrimitiveType,
    pub dimensions: Option<Dimensions3D>,
    pub position: Vector3<f32>,
}

fn measurement_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<MeasurementUiState>,
    mut created_primitive: ResMut<CreatedPrimitive>,
) {
    egui::Window::new("Bevy SketchUp")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Measurement Tools");

            ui.horizontal(|ui| {
                ui.label("Unit:");
                if ui.radio_value(&mut ui_state.active_unit, Unit::Centimeters, "cm").clicked() {
                    update_unit_display(&mut ui_state, Unit::Centimeters);
                }
                if ui.radio_value(&mut ui_state.active_unit, Unit::Meters, "m").clicked() {
                    update_unit_display(&mut ui_state, Unit::Meters);
                }
                if ui.radio_value(&mut ui_state.active_unit, Unit::Feet, "ft").clicked() {
                    update_unit_display(&mut ui_state, Unit::Feet);
                }
                if ui.radio_value(&mut ui_state.active_unit, Unit::Inches, "in").clicked() {
                    update_unit_display(&mut ui_state, Unit::Inches);
                }
            });

            ui.separator();
            ui.heading("Create Primitive");

            egui::ComboBox::from_label("Primitive Type")
                .selected_text(format!("{}", ui_state.selected_primitive))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut ui_state.selected_primitive, PrimitiveType::Cube, "Cube");
                    ui.selectable_value(&mut ui_state.selected_primitive, PrimitiveType::Cylinder, "Cylinder");
                    ui.selectable_value(&mut ui_state.selected_primitive, PrimitiveType::Pyramid, "Pyramid");
                    ui.selectable_value(&mut ui_state.selected_primitive, PrimitiveType::Sphere, "Sphere");
                });

            ui.separator();

            match ui_state.selected_primitive {
                PrimitiveType::Cube => {
                    ui.heading("Cube Dimensions");
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.text_edit_singleline(&mut ui_state.width_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.text_edit_singleline(&mut ui_state.height_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Depth:");
                        ui.text_edit_singleline(&mut ui_state.depth_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                },
                PrimitiveType::Cylinder => {
                    ui.heading("Cylinder Dimensions");
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.text_edit_singleline(&mut ui_state.width_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.text_edit_singleline(&mut ui_state.height_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                },
                PrimitiveType::Pyramid => {
                    ui.heading("Pyramid Dimensions");
                    ui.horizontal(|ui| {
                        ui.label("Base Width:");
                        ui.text_edit_singleline(&mut ui_state.width_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Base Depth:");
                        ui.text_edit_singleline(&mut ui_state.depth_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.text_edit_singleline(&mut ui_state.height_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                },
                PrimitiveType::Sphere => {
                    ui.heading("Sphere Dimensions");
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.text_edit_singleline(&mut ui_state.width_input);
                        ui.label(format!("{}", ui_state.active_unit));
                    });
                },
            }

            if ui.button("Create").clicked() {
                create_primitive(&mut ui_state, &mut created_primitive);
            }

            if let Some(error) = &ui_state.error_message {
                ui.separator();
                ui.colored_label(egui::Color32::RED, error);
            }
        });
}

fn update_unit_display(ui_state: &mut MeasurementUiState, new_unit: Unit) {
    // This function would update the display of current measurements when the unit changes
    ui_state.active_unit = new_unit;

    // Here we would convert the current values to the new unit
    // For now, just clear the inputs to avoid confusion
    ui_state.width_input.clear();
    ui_state.height_input.clear();
    ui_state.depth_input.clear();
}

fn create_primitive(ui_state: &mut MeasurementUiState, created_primitive: &mut CreatedPrimitive) {
    ui_state.error_message = None;

    // Parse dimensions
    let width_result = match ui_state.width_input.parse::<f32>() {
        Ok(w) => Measurement::new(w, ui_state.active_unit),
        Err(_) => {
            ui_state.error_message = Some("Invalid width value".to_string());
            return;
        }
    };

    let height_result = match ui_state.height_input.parse::<f32>() {
        Ok(h) => Measurement::new(h, ui_state.active_unit),
        Err(_) => {
            ui_state.error_message = Some("Invalid height value".to_string());
            return;
        }
    };

    let depth_result = if ui_state.selected_primitive == PrimitiveType::Cube {
        match ui_state.depth_input.parse::<f32>() {
            Ok(d) => Some(Measurement::new(d, ui_state.active_unit)),
            Err(_) => {
                ui_state.error_message = Some("Invalid depth value".to_string());
                return;
            }
        }
    } else {
        None
    };

    // If any measurements failed to create, show the error
    let (width, height, depth) = match (width_result, height_result, depth_result) {
        (Ok(w), Ok(h), None) => (w, h, None),
        (Ok(w), Ok(h), Some(Ok(d))) => (w, h, Some(d)),
        _ => {
            ui_state.error_message = Some("Invalid measurement values".to_string());
            return;
        }
    };

    // Create the dimensions based on primitive type
    let dimensions = match ui_state.selected_primitive {
        PrimitiveType::Cube => {
            if let Some(d) = depth {
                Some(Dimensions3D::new(width, height, d))
            } else {
                ui_state.error_message = Some("Depth is required for cubes".to_string());
                None
            }
        },
        PrimitiveType::Cylinder => Some(Dimensions3D::new(width, height, width)), // Use width as radius
        _ => {
            ui_state.error_message = Some("Primitive type not yet implemented".to_string());
            None
        }
    };

    if let Some(dims) = dimensions {
        // Calculate position before moving dims
        let position_y = dims.height.value_in_meters() / 2.0;
        
        // Set the creation request
        created_primitive.should_create = true;
        created_primitive.primitive_type = ui_state.selected_primitive;
        created_primitive.dimensions = Some(dims);
        created_primitive.position = Vector3::new(0.0, position_y, 0.0); // Place on the grid
        
        // Clear the inputs
        ui_state.width_input.clear();
        ui_state.height_input.clear();
        ui_state.depth_input.clear();
    }
}
