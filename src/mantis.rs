use crate::proc_anim::DynamicBody;
use bevy::prelude::*;

#[derive(Component)]
pub struct Mantis {
    pub speed: f32,
}

pub fn create_mantis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //center of mass placeholder
    let head_id = commands
        .spawn((
            Mantis { speed: 5.0 },
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .id();

    //create dynamic body
    let seg_lens = vec![0.5, 0.5, 0.5];
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
    ));
}
