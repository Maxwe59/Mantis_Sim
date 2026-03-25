use bevy::prelude::*;
mod mantis;
use mantis::create_mantis;
mod controls;
mod proc_anim;
use controls::{auto_movement, keyboard_controls, switch_movement_mode, ArcLengthCurve, lemniscate};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldOptions {
            movement_mode: MovementMode::Auto,
            auto_scale: 3.0,
            auto_t: -std::f32::consts::PI,
            arc_length: ArcLengthCurve::new(lemniscate, 100)
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, create_mantis)
        .add_systems(Startup, add_plane)
        .add_systems(Update, keyboard_controls)
        .add_systems(Update, proc_anim::calc_segment_pos)
        .add_systems(Update, switch_movement_mode)
        .add_systems(Update, auto_movement)
        .run();
}

#[derive(PartialEq, Debug)]
enum MovementMode {
    Mouse,
    Keyboard,
    Auto,
}

#[derive(Resource)]
struct WorldOptions {
    movement_mode: MovementMode,
    arc_length: ArcLengthCurve,
    auto_scale: f32,
    auto_t: f32,
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn add_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}
