use crate::proc_anim::{DynamicBody, FabrikJoint, NodeOffsetter, PivotEntity, SegmentFiller};
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

fn linear_downset(i: i32) -> Vec3 {
    return Vec3::new(0.0, 0.0 - (i as f32 * 0.01), 0.0);
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
    let seg_lens = vec![0.4; 10];
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
                    Mesh3d(meshes.add(Cylinder::new(0.15, seg_lens[i]))),
                    MeshMaterial3d(materials.add(Color::srgb_u8(255, 124, 144))),
                ))
                .id();
            midpoint_segments.push(midpoint_id);
        }
    }
    let segments_cloned = segments.clone();
    let segments_cloned2 = segments.clone();
    commands.spawn((
        DynamicBody::new(seg_lens, segments, 10.0 * std::f32::consts::PI / 180.0, 0.8, head_id),
        SegmentFiller::new(segments_cloned, midpoint_segments, Vec3::Y),
        NodeOffsetter::new(segments_cloned2, linear_downset),
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
        PivotEntity::new(head_id, Vec3::new(0.2, 0.0, 0.0), offset_entity),
        FabrikJoint::new_with_default(
            seg_lens,
            segments,
            0.5,
            0.7,
            Vec3::new(0.4, -0.2, 0.2),
            offset_entity,
        ),
    ));
}

/*

define mantis:
abdomen is composed of dynamic body segments, on a linear downward path
legs use fabrik algorithm, later add angle restrictions, and alternating step
thorax should have breathing effect, increase rate of breathing the faster the mantis moves
thorax movement:
head should point to the direction of the mouse cursor, or as close in a dir as possible
antenae should be composed of dynamic body segments (upward curve)
pinchers should also use fabrik, different target. they should move towards the mouse




*/
