//! White furnace test for Bevy Solari's pathtracer BRDF.
//!
//! Renders a grid of spheres with varying metallic (rows) and roughness (columns)
//! in a uniform white environment. A perfectly energy-conserving BRDF will display
//! all spheres at identical brightness — any darker sphere indicates energy loss.

use bevy::{
    camera::{CameraMainTextureUsages, Hdr},
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::render_resource::TextureUsages,
    solari::{
        pathtracer::{Pathtracer, PathtracingPlugin},
        prelude::{RaytracingMesh3d, SolariPlugins},
    },
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SolariPlugins, PathtracingPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight::NONE);

    let sphere = meshes.add(
        Sphere::new(0.4)
            .mesh()
            .build()
            .with_generated_tangents()
            .unwrap(),
    );

    let roughness_values = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0f32];
    let metallic_values = [0.0, 0.5, 1.0f32];
    let cols = roughness_values.len();
    let rows = metallic_values.len();
    let spacing = 1.1_f32;

    for (row, &metallic) in metallic_values.iter().enumerate() {
        for (col, &perceptual_roughness) in roughness_values.iter().enumerate() {
            let x = (col as f32 - (cols - 1) as f32 / 2.0) * spacing;
            let y = (row as f32 - (rows - 1) as f32 / 2.0) * spacing;
            commands.spawn((
                RaytracingMesh3d(sphere.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    metallic,
                    perceptual_roughness,
                    ..default()
                })),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }

    // sample_random_light divides by light_count, causing NaN with zero lights.
    // A zero-illuminance light keeps the buffer non-empty without contributing any radiance.
    commands.spawn(DirectionalLight {
        illuminance: 0.0,
        shadow_maps_enabled: false,
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
        Msaa::Off,
        Hdr,
        Tonemapping::None,
        Pathtracer::default(),
    ));
}
