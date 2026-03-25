use bevy::prelude::*;


#[derive(Component)]
pub struct Mantis{
    pub speed: f32,

}

pub fn center_of_mass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
        // cube
    commands.spawn((
        Mantis{ speed: 0.1 },
        Mesh3d(meshes.add(Sphere::new(0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}

pub fn user_input(
    mut mantis: Single<(&mut Transform, &Mantis)>,
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