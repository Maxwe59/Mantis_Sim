use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, placeholder)
        .add_systems(Startup, add_plane)
        .add_systems(Update, user_input)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
) {
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


fn placeholder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
        // cube
    commands.spawn((
        MantisPhysics{ speed: 0.1 },
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}

fn add_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}


fn user_input(
    mut mantis: Single<(&mut Transform, &MantisPhysics)>,
    input: Res<ButtonInput<KeyCode>>,
){  
    let mut transform = (0.0, 0.0);
    if input.pressed(KeyCode::KeyW) {
        transform.1 += 1.0;

    }
    else if input.pressed(KeyCode::KeyS) {

        transform.1 -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        transform.0 -= 1.0;

    }
    else if input.pressed(KeyCode::KeyD) {
        transform.0 += 1.0;
    }
    let speed = mantis.1.speed;
    mantis.0.translation.x += transform.0 * speed;
    mantis.0.translation.z += transform.1 * speed;
    
}


#[derive(Component)]
struct MantisPhysics{
    speed: f32,

}