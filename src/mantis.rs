use crate::proc_anim::{DynamicBody, FabrikJoint, OffSetter, SegmentFiller};
use bevy::prelude::*;

#[derive(Component)]
pub struct Mantis {
    pub speed: f32,
    init_center_of_mass: Vec3,
    //include color, and scale factors here later
}

impl Mantis {
    pub fn init_bundle(&self) -> impl Bundle {
        return (
            /*
            Mantis {
                speed: 5.0,
                init_center_of_mass: self.init_center_of_mass,
            },
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 255, 255))),
            Transform::from_xyz(center_of_mass.x, center_of_mass.y, center_of_mass.z),
             */
        );
    }
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
    let seg_lens = vec![0.2, 0.2, 0.2, 0.2, 0.2];
    let mut segments = Vec::new();
    let mut midpoint_segments = Vec::new();
    for i in 0..seg_lens.len() + 1 {
        let segment_id = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                Transform::from_xyz(i as f32, 0.5, 0.0),
            ))
            .id();
        segments.push(segment_id);

        if (i < seg_lens.len()) {
            let midpoint_id = commands
                .spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.2, seg_lens[i] / 2.0))),
                    MeshMaterial3d(materials.add(Color::srgb_u8(255, 124, 144))),
                ))
                .id();
            midpoint_segments.push(midpoint_id);
        }
    }
    let offset_entity = segments[0].clone();
    let segments_cloned = segments.clone();
    commands.spawn((
        DynamicBody::new(seg_lens, segments, 30.0 * std::f32::consts::PI / 180.0, 0.8),
        OffSetter::new(head_id, Vec3::new(0.0, 0.0, 0.2), offset_entity),
        SegmentFiller::new(segments_cloned, midpoint_segments, Vec3::Y),
    ));

    //create fabrik joinnt
    let seg_lens = vec![0.2, 0.2, 0.2];
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
    let offset_entity = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        ))
        .id();
    commands.spawn((
        OffSetter::new(head_id, Vec3::new(0.2, 0.0, 0.0), offset_entity),
        FabrikJoint::new_with_default(
            seg_lens,
            segments,
            0.5,
            0.7,
            Vec3::new(0.4, 0.0, 0.2),
            offset_entity,
            Vec3::new(0.4, 0.0, 0.0),
        ),
    ));

    /*
    pub fn new_with_default(
        seg_lengths: Vec<f32>,
        segments: Vec<Entity>,
        max_target_dist: f32,
        lerp_speed: f32,
        target_offset: Vec3,
        anchor_entity: Entity,
        init_target: Vec3,
    )


     */
}
