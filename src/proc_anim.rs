use bevy::prelude::*;

#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    head: Entity,          //entity dynamic body is connected to.
    segments: Vec<Entity>, //vec length should be seg_count - 1
    offset_head: Vec3,
}
#[derive(Component)]
pub struct FabrikJoint {}

impl DynamicBody {
    pub fn new(seg_lens: Vec<f32>, segments: Vec<Entity>, head: Entity, offset: Vec3) -> Self {
        Self {
            seg_lengths: seg_lens,
            head: head,
            segments: segments,
            offset_head: offset,
        }
    }

    fn get_seg_len(&self) -> i32 {
        return self.seg_lengths.len() as i32;
    }
}

pub fn setup_dynamic_body(
    dynamic_body: Single<&DynamicBody>,
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
) {
    let first_segment = dynamic_body.segments[0];
    let head_transform = transforms.get_mut(dynamic_body.head).unwrap().translation;
    let mut first_seg_transform = transforms.get_mut(first_segment).unwrap();

    first_seg_transform.translation = head_transform;
    commands
        .entity(first_segment)
        .set_parent_in_place(dynamic_body.head);
    first_seg_transform.translation += dynamic_body.offset_head;
}

pub fn calc_segment_pos(
    dynamic_body: Single<&DynamicBody>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    let segments = &dynamic_body.segments;
    let segment_lengths = &dynamic_body.seg_lengths;

    let mut last_vec = global_transforms
        .get(dynamic_body.segments[0])
        .unwrap()
        .translation();
    for (i, segment) in segments.iter().skip(1).enumerate() {
        if let Ok(mut transform) = transforms.get_mut(segment.clone()) {
            let current_vec = transform.translation;
            let new_vec = distance_restraints(last_vec, current_vec, segment_lengths[i]);
            transform.translation = new_vec;
            last_vec = transform.translation;
        }
    }

    let test_vec = transforms
        .get(dynamic_body.segments[0])
        .unwrap()
        .translation;
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}
