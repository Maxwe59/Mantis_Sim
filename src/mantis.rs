use crate::proc_anim::DynamicBody;
use bevy::prelude::*;

#[derive(Component)]
pub struct Mantis {
    pub speed: f32,
    init_center_of_mass: Vec3,
    //include color, and scale factors here later
}

pub fn create_mantis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //center of mass placeholder
    let center_of_mass = Vec3::new(0.0, 0.5, 0.0);
    let head_id = commands
        .spawn((
            Mantis {
                speed: 5.0,
                init_center_of_mass: center_of_mass,
            },
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
            Transform::from_xyz(center_of_mass.x, center_of_mass.y, center_of_mass.z),
        ))
        .id();

    //create dynamic body
    let seg_lens = vec![0.2, 0.2];
    let mut segments = Vec::new();
    for i in 0..seg_lens.len() + 1 {
        let segment_id = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                Transform::from_xyz(i as f32, 0.5, 0.0),
            ))
            .id();
        segments.push(segment_id);
    }
    commands.spawn(DynamicBody::new(
        seg_lens,
        segments,
        head_id,
        Vec3::new(0.0, 0.0, 0.5),
        30.0 * std::f32::consts::PI / 180.0,
        0.8,
    ));
}
