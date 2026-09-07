//! Illustrates rectangular area lights and how surface roughness and anisotropy affect
//! their appearance.
//!
//! The floor is anisotropic, which puts it on the anisotropic LTC path; the sphere is
//! isotropic, so both paths are on screen at once. Press Enter to toggle the floor between
//! them.

use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            }),
            FreeCameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_gizmos, update_floor))
        .run();
}

#[derive(Resource)]
struct FloorMaterial(Handle<StandardMaterial>);

#[derive(Component)]
struct RoughnessDisplay;

/// Anisotropy strength the floor uses when enabled.
///
/// Deliberately below 1.0: at full strength the larger roughness is pinned to 1.0 while the
/// smaller collapses toward zero, which lands in the corner of the LUT with the least
/// angular resolution.
const FLOOR_ANISOTROPY: f32 = 0.75;

/// Simple scene with a sphere on a reflective floor, lit by two rectangular area lights
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        metallic: 1.0,
        perceptual_roughness: 0.6,
        anisotropy_strength: FLOOR_ANISOTROPY,
        anisotropy_rotation: 0.0,
        ..default()
    });
    commands.insert_resource(FloorMaterial(floor_material.clone()));
    commands.spawn((
        // Anisotropy is defined relative to the surface tangent, so the floor needs tangents.
        Mesh3d(meshes.add(
            Mesh::from(Plane3d::default().mesh().size(20.0, 20.0))
                .with_generated_tangents()
                .expect("plane mesh should have UVs and normals"),
        )),
        MeshMaterial3d(floor_material),
    ));

    // Left isotropic on purpose, as an in-frame reference for the other LTC path.
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));

    // Lights
    commands.spawn((
        RectLight {
            color: Color::srgb(1.0, 0.3, 0.2),
            intensity: 100_000.0,
            width: 2.0,
            height: 1.0,
            range: 20.0,
        },
        ShowLightGizmo::default(),
        Transform::from_xyz(1.0, 3.0, 1.0).looking_at(Vec3::Y, Vec3::Y),
    ));

    commands.spawn((
        RectLight {
            color: Color::srgb(0.5, 0.7, 1.0),
            intensity: 800_000.0,
            width: 1.5,
            height: 4.0,
            range: 20.0,
        },
        ShowLightGizmo::default(),
        Transform::from_xyz(-2.0, 1.5, -3.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 5.0, 8.0).looking_at(Vec3::Y, Vec3::Y),
        FreeCamera::default(),
    ));

    commands.spawn((
        Text::new(status_text(0.6, FLOOR_ANISOTROPY, 0.0)),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        RoughnessDisplay,
    ));
}

/// Builds the help/status readout.
fn status_text(roughness: f32, anisotropy: f32, rotation: f32) -> String {
    format!(
        "Controls\n\
         Arrow Up/Down: Adjust floor roughness\n\
         Arrow Left/Right: Rotate floor anisotropy direction\n\
         Enter: Toggle floor anisotropy\n\
         G: Toggle light gizmos\n\n\
         Roughness: {roughness:.2}\n\
         Anisotropy: {}\n\
         Direction: {:.0} deg",
        if anisotropy > 0.0 { "on" } else { "off" },
        rotation.to_degrees(),
    )
}

/// Drives the floor's roughness, anisotropy strength and anisotropy direction.
///
/// Toggling the strength to zero drops the material off the anisotropic path entirely, so
/// the A key A/Bs the anisotropic LTC LUT against the isotropic one on the same surface.
fn update_floor(
    keys: Res<ButtonInput<KeyCode>>,
    floor_material: Res<FloorMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut text_query: Query<&mut Text, With<RoughnessDisplay>>,
) {
    let roughness_delta = if keys.pressed(KeyCode::ArrowUp) {
        0.005
    } else if keys.pressed(KeyCode::ArrowDown) {
        -0.005
    } else {
        0.0
    };

    let rotation_delta = if keys.pressed(KeyCode::ArrowRight) {
        0.02
    } else if keys.pressed(KeyCode::ArrowLeft) {
        -0.02
    } else {
        0.0
    };

    // Not a letter key: the free camera owns W/A/S/D/E/Q, Shift, M and Ctrl. Enter also
    // matches the anisotropy toggle in the `anisotropy` example.
    let toggle_anisotropy = keys.just_pressed(KeyCode::Enter);

    // Avoid touching the asset unless something actually changed, so we don't mark the
    // material dirty and re-upload it every frame.
    if roughness_delta == 0.0 && rotation_delta == 0.0 && !toggle_anisotropy {
        return;
    }

    let Some(mut material) = materials.get_mut(&floor_material.0) else {
        return;
    };

    material.perceptual_roughness =
        (material.perceptual_roughness + roughness_delta).clamp(0.0, 1.0);
    material.anisotropy_rotation =
        (material.anisotropy_rotation + rotation_delta).rem_euclid(core::f32::consts::TAU);

    if toggle_anisotropy {
        material.anisotropy_strength = if material.anisotropy_strength > 0.0 {
            0.0
        } else {
            FLOOR_ANISOTROPY
        };
    }

    if let Ok(mut text) = text_query.single_mut() {
        **text = status_text(
            material.perceptual_roughness,
            material.anisotropy_strength,
            material.anisotropy_rotation,
        );
    }
}

fn toggle_gizmos(keys: Res<ButtonInput<KeyCode>>, mut config_store: ResMut<GizmoConfigStore>) {
    if keys.just_pressed(KeyCode::KeyG) {
        let (config, light_config) = config_store.config_mut::<LightGizmoConfigGroup>();
        light_config.draw_all = false;
        config.enabled = !config.enabled;
    }
}
