use bevy::prelude::*;

#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    head: Entity,          //entity dynamic body is connected to.
    segments: Vec<Entity>, //vec length should be seg_count - 1
    offset_head: Vec3,
    angle_constraints: f32,
    lerp_speed: f32,
}
#[derive(Component)]
pub struct FabrikJoint {}

impl DynamicBody {
    pub fn new(
        seg_lens: Vec<f32>,
        segments: Vec<Entity>,
        head: Entity,
        offset: Vec3,
        angle: f32,
        lerp: f32,
    ) -> Self {
        Self {
            seg_lengths: seg_lens,
            head: head,
            segments: segments,
            offset_head: offset,
            angle_constraints: angle,
            lerp_speed: lerp,
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
}

pub fn angle_constraints(
    dynamic_body: Single<&DynamicBody>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&mut GlobalTransform>,
) {
    let head_position = global_transforms
        .get(dynamic_body.head)
        .unwrap()
        .translation();
    let first_seg_pos = global_transforms
        .get(dynamic_body.segments[0])
        .unwrap()
        .translation();
    let mut last_vec = (first_seg_pos - head_position).normalize();
    let segments = &dynamic_body.segments;
    let segment_lengths = &dynamic_body.seg_lengths;

    for i in 0..segment_lengths.len() {
        let front_pos = global_transforms
            .get(dynamic_body.segments[i])
            .unwrap()
            .translation();
        let back_pos = global_transforms
            .get(dynamic_body.segments[i + 1])
            .unwrap()
            .translation();

        let current_vec = (back_pos - front_pos).normalize();
        let angle = last_vec.angle_between(current_vec);
        let segment_to_change = segments[i + 1].clone();
        let past_segment = segments[i].clone();
        if (angle > dynamic_body.angle_constraints) {
            let axis = current_vec.cross(last_vec).normalize();
            let new_vec =
                Quat::from_axis_angle(axis, angle - dynamic_body.angle_constraints) * current_vec;
            let new_pos = global_transforms.get(past_segment).unwrap().translation()
                + (new_vec * segment_lengths[i]);
            let final_lerp = transforms
                .get(segment_to_change)
                .unwrap()
                .translation
                .lerp(new_pos, dynamic_body.lerp_speed);
            transforms.get_mut(segment_to_change).unwrap().translation = final_lerp;
            last_vec = new_vec;
        } else {
            last_vec = current_vec;
        }
    }
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}

pub fn procedural_animation_plugin(app: &mut App) {
    app.add_systems(PostStartup, setup_dynamic_body)
        .add_systems(Update, (angle_constraints, calc_segment_pos).chain());
}
