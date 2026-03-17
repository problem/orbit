use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use crate::camera::orbit_camera::OrbitCameraPlugin;
use crate::ui::measurement_ui::{MeasurementUiPlugin, MeasurementUiState, PrimitiveType, CreatedPrimitive};
use crate::primitives::{Cube, Cylinder};
use crate::units::measurement::{Unit, Measurement, Dimensions3D};
use nalgebra::Vector3;

#[derive(Component)]
struct ModelComponent {
    pub model_type: String,
    pub vertex_indices: Vec<usize>,
    pub edge_indices: Vec<usize>,
    pub face_indices: Vec<usize>,
}

pub struct BevySketchupPlugin;

impl Plugin for BevySketchupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins)
            .add_plugins(EguiPlugin)
            .add_plugins(OrbitCameraPlugin)
            .add_plugins(MeasurementUiPlugin)
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (handle_primitive_creation, create_primitive_mesh))
            .add_systems(Update, bevy::window::close_on_esc);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a simple grid to help with orientation
    let grid_size = 10.0;
    let grid_divisions = 10;
    let grid_color = Color::rgb(0.5, 0.5, 0.5);

    for i in 0..=grid_divisions {
        let pos = -grid_size / 2.0 + i as f32 * grid_size / grid_divisions as f32;

        // X lines
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -grid_size / 2.0,
                max_x: grid_size / 2.0,
                min_y: 0.0,
                max_y: 0.01,
                min_z: pos - 0.005,
                max_z: pos + 0.005,
            })),
            material: materials.add(StandardMaterial {
                base_color: grid_color,
                ..default()
            }),
            ..default()
        });

        // Z lines
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: pos - 0.005,
                max_x: pos + 0.005,
                min_y: 0.0,
                max_y: 0.01,
                min_z: -grid_size / 2.0,
                max_z: grid_size / 2.0,
            })),
            material: materials.add(StandardMaterial {
                base_color: grid_color,
                ..default()
            }),
            ..default()
        });
    }

    // Add a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn handle_primitive_creation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<Input<KeyCode>>,
    ui_state: Res<MeasurementUiState>,
) {
    // For demonstration, let's create a cube when the space key is pressed
    if keys.just_pressed(KeyCode::Space) {
        // Create a default cube if no valid input is provided
        let width = ui_state.width_input.parse::<f32>().unwrap_or(1.0);
        let height = ui_state.height_input.parse::<f32>().unwrap_or(1.0);
        let depth = ui_state.depth_input.parse::<f32>().unwrap_or(1.0);
        
        // Create the dimensions using the current unit
        let width_measurement = Measurement::new(width, ui_state.active_unit).unwrap_or_else(|_| Measurement::new(1.0, Unit::Meters).unwrap());
        let height_measurement = Measurement::new(height, ui_state.active_unit).unwrap_or_else(|_| Measurement::new(1.0, Unit::Meters).unwrap());
        let depth_measurement = Measurement::new(depth, ui_state.active_unit).unwrap_or_else(|_| Measurement::new(1.0, Unit::Meters).unwrap());
        
        let dimensions = Dimensions3D::new(width_measurement, height_measurement, depth_measurement);
        
        // Create a cube at the origin
        let cube = Cube::new(dimensions, Vector3::new(0.0, 0.0, 0.0));
        let (vertices, edges, faces) = cube.generate_mesh_data();
        
        // Create a Bevy mesh from our vertices and faces
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        
        // Convert vertices to Bevy's format
        let bevy_positions: Vec<[f32; 3]> = vertices.iter()
            .map(|v| [v.position.x, v.position.y, v.position.z])
            .collect();
        
        // Extract indices from faces
        let indices: Vec<u32> = faces.iter()
            .flat_map(|face| face.vertex_indices.iter().map(|&i| i as u32))
            .collect();

        // Calculate normals for each vertex
        let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
        for face in &faces {
            if let Some(normal) = face.normal(&vertices) {
                let normal_array = [normal.x, normal.y, normal.z];
                // Apply the face normal to all vertices of this face
                for &vertex_idx in &face.vertex_indices {
                    normals[vertex_idx] = normal_array;
                }
            }
        }
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        
        // Create a material
        let material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..default()
        });
        
        // Spawn the mesh as a PbrBundle
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh),
                material,
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            ModelComponent {
                model_type: "Cube".to_string(),
                vertex_indices: (0..vertices.len()).collect(),
                edge_indices: (0..edges.len()).collect(),
                face_indices: (0..faces.len()).collect(),
            }
        ));
    }
}

fn create_primitive_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut created_primitive: ResMut<CreatedPrimitive>,
) {
    if !created_primitive.should_create {
        return;
    }

    if let Some(dimensions) = created_primitive.dimensions.take() {
        match created_primitive.primitive_type {
            PrimitiveType::Cube => {
                let cube = Cube::new(dimensions, created_primitive.position);
                let (vertices, edges, faces) = cube.generate_mesh_data();
                
                // Create a Bevy mesh from our vertices and faces
                let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
                
                // Convert vertices to Bevy's format
                let bevy_positions: Vec<[f32; 3]> = vertices.iter()
                    .map(|v| [v.position.x, v.position.y, v.position.z])
                    .collect();
                
                // Extract indices from faces
                let indices: Vec<u32> = faces.iter()
                    .flat_map(|face| face.vertex_indices.iter().map(|&i| i as u32))
                    .collect();

                // Calculate normals for each vertex
                let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
                for face in &faces {
                    if let Some(normal) = face.normal(&vertices) {
                        let normal_array = [normal.x, normal.y, normal.z];
                        // Apply the face normal to all vertices of this face
                        for &vertex_idx in &face.vertex_indices {
                            normals[vertex_idx] = normal_array;
                        }
                    }
                }
                
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
                mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
                
                // Create a material
                let material = materials.add(StandardMaterial {
                    base_color: Color::rgb(0.8, 0.7, 0.6),
                    ..default()
                });
                
                // Convert nalgebra Vector3 to Bevy Vec3
                let position = Vec3::new(
                    created_primitive.position.x,
                    created_primitive.position.y,
                    created_primitive.position.z,
                );
                
                // Spawn the mesh as a PbrBundle
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(mesh),
                        material,
                        transform: Transform::from_translation(position),
                        ..default()
                    },
                    ModelComponent {
                        model_type: "Cube".to_string(),
                        vertex_indices: (0..vertices.len()).collect(),
                        edge_indices: (0..edges.len()).collect(),
                        face_indices: (0..faces.len()).collect(),
                    }
                ));
            },
            PrimitiveType::Cylinder => {
                // Create a cylinder with the given dimensions
                let radius = dimensions.width; // Using width as radius
                let height = dimensions.height;
                let segments = 32; // Default to 32 segments for smooth circles

                match Cylinder::new(radius, height, created_primitive.position, segments) {
                    Ok(cylinder) => {
                        let (vertices, edges, faces) = cylinder.generate_mesh_data();
                        
                        // Create a Bevy mesh from our vertices and faces
                        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
                        
                        // Convert vertices to Bevy's format
                        let bevy_positions: Vec<[f32; 3]> = vertices.iter()
                            .map(|v| [v.position.x, v.position.y, v.position.z])
                            .collect();
                        
                        // Extract indices from faces
                        let indices: Vec<u32> = faces.iter()
                            .flat_map(|face| face.vertex_indices.iter().map(|&i| i as u32))
                            .collect();
                        
                        // Calculate normals for each vertex
                        let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];
                        for face in &faces {
                            if let Some(normal) = face.normal(&vertices) {
                                let normal_array = [normal.x, normal.y, normal.z];
                                for &vertex_idx in &face.vertex_indices {
                                    normals[vertex_idx] = normal_array;
                                }
                            }
                        }
                        
                        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_positions);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
                        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
                        
                        // Create a material
                        let material = materials.add(StandardMaterial {
                            base_color: Color::rgb(0.7, 0.8, 0.6),
                            ..default()
                        });
                        
                        // Convert nalgebra Vector3 to Bevy Vec3
                        let position = Vec3::new(
                            created_primitive.position.x,
                            created_primitive.position.y,
                            created_primitive.position.z,
                        );
                        
                        // Spawn the mesh as a PbrBundle
                        commands.spawn((
                            PbrBundle {
                                mesh: meshes.add(mesh),
                                material,
                                transform: Transform::from_translation(position),
                                ..default()
                            },
                            ModelComponent {
                                model_type: "Cylinder".to_string(),
                                vertex_indices: (0..vertices.len()).collect(),
                                edge_indices: (0..edges.len()).collect(),
                                face_indices: (0..faces.len()).collect(),
                            }
                        ));
                    },
                    Err(err) => {
                        // Log the error and fall back to a cube for now
                        eprintln!("Failed to create cylinder: {:?}", err);
                        // Create a cube with the same dimensions as a fallback
                        let cube = Cube::new(dimensions, created_primitive.position);
                        let (vertices, edges, faces) = cube.generate_mesh_data();
                        
                        // Create a Bevy mesh from our vertices and faces
                        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
                        
                        // Convert vertices to Bevy's format
                        let bevy_positions: Vec<[f32; 3]> = vertices.iter()
                            .map(|v| [v.position.x, v.position.y, v.position.z])
                            .collect();
                        
                        // Extract indices from faces
                        let indices: Vec<u32> = faces.iter()
                            .flat_map(|face| face.vertex_indices.iter().map(|&i| i as u32))
                            .collect();
                        
                        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_positions);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; vertices.len()]);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
                        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
                        
                        // Create a material
                        let material = materials.add(StandardMaterial {
                            base_color: Color::rgb(0.8, 0.7, 0.6),
                            ..default()
                        });
                        
                        // Convert nalgebra Vector3 to Bevy Vec3
                        let position = Vec3::new(
                            created_primitive.position.x,
                            created_primitive.position.y,
                            created_primitive.position.z,
                        );
                        
                        // Spawn the mesh as a PbrBundle
                        commands.spawn((
                            PbrBundle {
                                mesh: meshes.add(mesh),
                                material,
                                transform: Transform::from_translation(position),
                                ..default()
                            },
                            ModelComponent {
                                model_type: "Cube".to_string(),
                                vertex_indices: (0..vertices.len()).collect(),
                                edge_indices: (0..edges.len()).collect(),
                                face_indices: (0..faces.len()).collect(),
                            }
                        ));
                    }
                }
            },
            _ => {
                // Other primitive types not implemented yet
            }
        }
    }

    // Reset the creation request
    created_primitive.should_create = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_ui_state() {
        let mut app = App::new();
        app.insert_resource(MeasurementUiState {
            active_unit: Unit::Meters,
            width_input: String::new(),
            height_input: String::new(),
            depth_input: String::new(),
            error_message: None,
            selected_primitive: PrimitiveType::Cube,
        });

        assert!(app.world.contains_resource::<MeasurementUiState>());
    }

    #[test]
    fn test_cube_creation() {
        let width = 1.0;
        let height = 2.0;
        let depth = 3.0;
        
        let width_measurement = Measurement::new(width, Unit::Meters).unwrap();
        let height_measurement = Measurement::new(height, Unit::Meters).unwrap();
        let depth_measurement = Measurement::new(depth, Unit::Meters).unwrap();
        
        let dimensions = Dimensions3D::new(width_measurement, height_measurement, depth_measurement);
        let cube = Cube::new(dimensions, Vector3::new(0.0, 0.0, 0.0));
        
        let (vertices, _edges, faces) = cube.generate_mesh_data();
        
        // A cube should have 8 vertices and 6 faces
        assert_eq!(vertices.len(), 8);
        assert_eq!(faces.len(), 6);
    }

    #[test]
    fn test_measurement_conversion() {
        let meters = Measurement::new(1.0, Unit::Meters).unwrap();
        let centimeters = Measurement::new(100.0, Unit::Centimeters).unwrap();
        
        assert_eq!(meters.value_in_meters(), centimeters.value_in_meters());
    }
}
