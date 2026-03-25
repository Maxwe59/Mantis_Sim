use bevy::prelude::*;
use crate::proc_anim::DynamicBody;

#[derive(Component)]
pub struct Mantis{
    pub speed: f32,

}

pub fn create_mantis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    //center of mass placeholder
    let head_id = commands.spawn((
        Mantis{ speed: 0.1 },
        Mesh3d(meshes.add(Sphere::new(0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    )).id();

    //create dynamic body
    let seg_lens = vec![0.5, 0.5, 0.5];
    let mut segments = Vec::new();
    for i in 0..seg_lens.len() + 1 {
        let segment_id = commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(i as f32, 0.5, 0.0),
        )).id();
        segments.push(segment_id);
    }
    commands.spawn(DynamicBody::new(seg_lens, segments, head_id));

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