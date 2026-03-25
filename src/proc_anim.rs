use bevy::prelude::*;

#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    head: Entity,          //entity dynamic body is connected to.
    segments: Vec<Entity>, //vec length should be seg_count - 1
}
#[derive(Component)]
pub struct FabrikJoint {}

impl DynamicBody {
    pub fn new(seg_lens: Vec<f32>, segments: Vec<Entity>, head: Entity) -> Self {
        Self {
            seg_lengths: seg_lens,
            head: head,
            segments: segments,
        }
    }

    fn get_seg_len(&self) -> i32 {
        return self.seg_lengths.len() as i32;
    }
}

pub fn calc_segment_pos(dynamic_body: Single<&DynamicBody>, mut transforms: Query<&mut Transform>) {
    let segments = &dynamic_body.segments;
    let segment_lengths = &dynamic_body.seg_lengths;

    let mut last_vec = transforms.get(dynamic_body.head).unwrap().translation;
    transforms.get_mut(segments[0]).unwrap().translation = last_vec;

    for (i, segment) in segments.iter().skip(1).enumerate() {
        if let Ok(mut transform) = transforms.get_mut(segment.clone()) {
            let current_vec = transform.translation;
            let new_vec = distance_restraints(last_vec, current_vec, segment_lengths[i]);
            transform.translation = new_vec;
            last_vec = transform.translation;
        }
    }
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}
