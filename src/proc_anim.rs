use bevy::prelude::*;



macro_rules! impl_new {
    ($t:ty, $($field:ident : $ftype:ty),*) => {
        impl $t {
            pub fn new($($field: $ftype),*) -> Self {
                Self {
                    $($field),*
                }
            }
        }
    };
}

#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    segments: Vec<Entity>, //vec length should be seg_count - 1
    angle_constraints: f32,
    lerp_speed: f32,
}

#[derive(Component)]
pub struct OffSetter{
    head: Entity,
    offset: Vec3,
    child: Entity,
}


#[derive(Component)]
pub struct FabrikJoint {}


impl_new!(OffSetter, head: Entity, offset: Vec3, child: Entity);
impl_new!(DynamicBody, seg_lengths: Vec<f32>, segments: Vec<Entity>, angle_constraints: f32, lerp_speed: f32);

impl DynamicBody {

    fn get_seg_len(&self) -> i32 {
        return self.seg_lengths.len() as i32;
    }
}

pub fn setup_offset(offset_query: Query<&OffSetter>, mut commands: Commands, mut transforms: Query<&mut Transform>){
    for offset in offset_query.iter(){
        //first set child/parent relationship
        commands.entity(offset.child).set_parent_in_place(offset.head);
        //transform child to parent 0
        transforms.get_mut(offset.child).unwrap().translation = Vec3::ZERO;
        //apply offset
        transforms.get_mut(offset.child).unwrap().translation = offset.offset;
        //transforms.get_mut(offset.child).unwrap().translation.y += 0.5; //temporary, should be based on center of mass
    }
}



pub fn calc_segment_pos(
    dynamic_body_query: Query<&DynamicBody>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for dynamic_body in dynamic_body_query.iter() {
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
}

pub fn angle_constraints(
    dynamic_body_query: Query<&DynamicBody>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for dynamic_body in dynamic_body_query.iter() {
        //need to get the opposite of forward vector because each vector points backward
        let mut last_vec = -1.0 * (*global_transforms.get(dynamic_body.segments[0]).unwrap().forward()); 
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
                let new_vec = Quat::from_axis_angle(axis, angle - dynamic_body.angle_constraints)
                    * current_vec;
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
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}

pub fn procedural_animation_plugin(app: &mut App) {
    app.add_systems(PostStartup, setup_offset)
        .add_systems(Update, (angle_constraints, calc_segment_pos).chain());
}

//todo: - midpoint object spawner, generalize offset function, fabrik joint component
