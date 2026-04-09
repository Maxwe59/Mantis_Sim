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
/*
Both FabrikJoint and DynamicBody assume the first segment[0] will be anchored to a "head" entity,
revolving around the head entity. (Assuming both need the component OffSetter)


*/
#[derive(Component)]
pub struct DynamicBody {
    seg_lengths: Vec<f32>, //length between segments, vec length should be seg_count - 1
    nodes: Vec<Entity>,    //vec length should be seg_count - 1
    angle_constraints: f32,
    lerp_speed: f32,
    anchor_entity: Entity,
    slope_func: fn(i32, Vec3) -> Vec3,
}

#[derive(Component)]
pub struct PivotEntity {
    head: Entity,
    offset: Vec3,
    child: Entity,
}

#[derive(Component)]
pub struct SegmentFiller {
    nodes: Vec<Entity>,
    midpoints: Vec<Entity>,
    vec_dir_segment: Vec3, //defines which vector direction the segment points in when the midpoint filler calculates rotation
}

#[derive(Component)]
pub struct FabrikJoint {
    seg_lengths: Vec<f32>,
    nodes: Vec<Entity>,
    max_target_dist: f32, //max distance target (foot) can get from target_pos (global space)
    lerp_speed: f32,
    target_offset: Vec3,   //relative to anchor position (segments[0]),
    anchor_entity: Entity, //entity the fabrik joint is anchored to.
    fabrik_iterations: i32,
    //interal variables used to calculate states
    stepping: bool,
    new_target_pos: Vec3, //used to lerp between the old target_pos and new_target_pos, when stepping is true.
    curr_target_pos: Vec3, //used to track foot location
    t_val: f32,
}

impl_new!(SegmentFiller, nodes: Vec<Entity>, midpoints: Vec<Entity>, vec_dir_segment: Vec3);
impl_new!(PivotEntity, head: Entity, offset: Vec3, child: Entity);
impl_new!(DynamicBody, seg_lengths: Vec<f32>, nodes: Vec<Entity>, angle_constraints: f32, lerp_speed: f32, anchor_entity: Entity, slope_func: fn(i32, Vec3) -> Vec3);
impl_new!(FabrikJoint, seg_lengths: Vec<f32>, nodes: Vec<Entity>, max_target_dist: f32, lerp_speed: f32, target_offset: Vec3, anchor_entity: Entity, fabrik_iterations: i32, stepping: bool, new_target_pos: Vec3, curr_target_pos: Vec3, t_val: f32);

impl FabrikJoint {
    pub fn new_with_default(
        seg_lengths: Vec<f32>,
        nodes: Vec<Entity>,
        max_target_dist: f32,
        lerp_speed: f32,
        target_offset: Vec3,
        anchor_entity: Entity,
    ) -> Self {
        return Self {
            seg_lengths: seg_lengths,
            nodes: nodes,
            max_target_dist: max_target_dist,
            lerp_speed: lerp_speed,
            target_offset: target_offset,
            anchor_entity: anchor_entity,
            fabrik_iterations: 5,
            stepping: false,
            new_target_pos: Vec3::ZERO,
            curr_target_pos: Vec3::ZERO,
            t_val: 0.0,
        };
    }
}

pub fn setup_offset(
    pivot_query: Query<&PivotEntity>,
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
) {
    for pivotter in pivot_query.iter() {
        //first set child/parent relationship
        commands
            .entity(pivotter.child)
            .insert(ChildOf(pivotter.head));
        //transform child to parent 0
        transforms.get_mut(pivotter.child).unwrap().translation = Vec3::ZERO;
        //apply offset
        transforms.get_mut(pivotter.child).unwrap().translation = pivotter.offset;
        //transforms.get_mut(pivotter.child).unwrap().translation.y += 0.5; //temporary, should be based on center of mass
    }
}

/*
prev_vector -> previous vector in the change, vector being measured to current vector
front_pos/backpos -> positions of 2 segments, they make up the current vector
angle_constraint -> max angle between prev_vector and current_vec

returns (vector,position)
*/

fn calc_angle_constraints(
    prev_vec: Vec3,
    front_pos: Vec3,
    back_pos: Vec3,
    angle_constraint: f32,
    lerp_speed: f32,
    segment_length: f32,
) -> (Vec3, Vec3) {
    let current_vec = (back_pos - front_pos).normalize();
    let angle = prev_vec.angle_between(current_vec);
    if angle > angle_constraint {
        let axis = current_vec.cross(prev_vec).normalize();
        let new_vec = Quat::from_axis_angle(axis, angle - angle_constraint) * current_vec;
        let new_pos = front_pos + (new_vec * segment_length);
        let final_lerp = back_pos.lerp(new_pos, lerp_speed);
        return (new_vec, final_lerp);
    } else {
        return (current_vec, back_pos);
    }
}

pub fn dynamic_body_calculator(
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    dynamic_body_query: Query<&DynamicBody>,
) {
    for dynamic_body in dynamic_body_query.iter() {
        let anchor_entity_pos = global_transforms.get(dynamic_body.anchor_entity).unwrap();
        let nodes = &dynamic_body.nodes;
        let segment_lengths = &dynamic_body.seg_lengths;

        let mut first_node = transforms.get_mut(nodes[0]).unwrap();
        first_node.translation = anchor_entity_pos.translation();
        first_node.rotation = anchor_entity_pos.rotation();
        let mut last_vec = -1.0
            * (*global_transforms
                .get(dynamic_body.nodes[0])
                .unwrap()
                .forward());

        let mut last_node_pos = global_transforms
            .get(dynamic_body.nodes[0])
            .unwrap()
            .translation();

        let transform_func = dynamic_body.slope_func;

        for i in 0..segment_lengths.len() {
            //angle restrictions

            let front_pos = global_transforms
                .get(dynamic_body.nodes[i])
                .unwrap()
                .translation();
            let back_pos = global_transforms
                .get(dynamic_body.nodes[i + 1])
                .unwrap()
                .translation();

            let (new_vec, new_pos) = calc_angle_constraints(
                last_vec,
                front_pos,
                back_pos,
                dynamic_body.angle_constraints,
                dynamic_body.lerp_speed,
                segment_lengths[i],
            );
            transforms.get_mut(nodes[i + 1]).unwrap().translation = new_pos;
            last_vec = new_vec;

            //apply segment offset
            let offset = transform_func(i as i32, last_node_pos);
            let mut node_transform = transforms.get_mut(nodes[i + 1]).unwrap();
            if offset.x != 0.0 {
                node_transform.translation.x = offset.x;
            }
            if offset.y != 0.0 {
                node_transform.translation.y = offset.y;
            }
            if offset.z != 0.0 {
                node_transform.translation.z = offset.z;
            }

            //apply distance constraints LAST
            let mut transform = transforms.get_mut(nodes[i + 1]).unwrap();
            let new_vec =
                distance_restraints(last_node_pos, transform.translation, segment_lengths[i]);
            transform.translation = new_vec;
            last_node_pos = transform.translation;
        }
    }
}

pub fn fabrik_calculator(
    mut fabrik_query: Query<&mut FabrikJoint>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for mut fabrik_joint in fabrik_query.iter_mut() {
        let rotation_of_anchor = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .rotation();
        let updated_target = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .translation()
            + (rotation_of_anchor * fabrik_joint.target_offset);
        let anchor_pos = global_transforms
            .get(fabrik_joint.anchor_entity)
            .unwrap()
            .translation();

        if fabrik_joint.max_target_dist < fabrik_joint.curr_target_pos.distance(updated_target) {
            //implement lerping logic
            fabrik_joint.stepping = true;
            fabrik_joint.new_target_pos = updated_target;
            fabrik_joint.t_val = 0.0;
        }

        if fabrik_joint.stepping {
            //recalculate currentmost target (because teh entire body is moving, using old target will result in incomplete step)
            fabrik_joint.new_target_pos = updated_target;
            fabrik_joint.t_val += fabrik_joint.lerp_speed;
            fabrik_joint.curr_target_pos = fabrik_joint
                .curr_target_pos
                .lerp(fabrik_joint.new_target_pos, fabrik_joint.t_val);
            //reset stepping to false
            if fabrik_joint.t_val >= 1.0 {
                fabrik_joint.stepping = false;
            }
        }

        for _i in 0..fabrik_joint.fabrik_iterations {
            //backpass
            transforms
                .get_mut(fabrik_joint.nodes.last().unwrap().clone())
                .unwrap()
                .translation = fabrik_joint.curr_target_pos;
            for i in (0..(fabrik_joint.nodes.len() - 1)).rev() {
                let point1 = transforms
                    .get(fabrik_joint.nodes[i].clone())
                    .unwrap()
                    .translation;
                let point2 = transforms
                    .get(fabrik_joint.nodes[i + 1].clone())
                    .unwrap()
                    .translation;
                let new_vec = (point1 - point2).normalize() * fabrik_joint.seg_lengths[i];
                transforms
                    .get_mut(fabrik_joint.nodes[i].clone())
                    .unwrap()
                    .translation = point2 + new_vec;
            }
            //frontpass
            transforms
                .get_mut(fabrik_joint.nodes[0].clone())
                .unwrap()
                .translation = anchor_pos;
            for i in 1..fabrik_joint.nodes.len() {
                let point1 = transforms
                    .get(fabrik_joint.nodes[i].clone())
                    .unwrap()
                    .translation;
                let point2 = transforms
                    .get(fabrik_joint.nodes[i - 1].clone())
                    .unwrap()
                    .translation;
                let new_vec = (point1 - point2).normalize() * fabrik_joint.seg_lengths[i - 1];
                transforms
                    .get_mut(fabrik_joint.nodes[i].clone())
                    .unwrap()
                    .translation = point2 + new_vec;
            }
        }
    }
}

fn midpoint_filler(
    segment_fillers: Query<&SegmentFiller>,
    global_transforms: Query<&GlobalTransform>,
    mut transforms: Query<&mut Transform>,
) {
    for segment_filler in segment_fillers.iter() {
        let entity_list = &segment_filler.nodes;
        let midpoint_entity_list = &segment_filler.midpoints; //will be len(entity_list)-1 length
        for i in 0..(midpoint_entity_list.len()) {
            let pos1 = global_transforms
                .get(entity_list[i].clone())
                .unwrap()
                .translation();
            let pos2 = global_transforms
                .get(entity_list[i + 1].clone())
                .unwrap()
                .translation();
            let midpoint = (pos1 + pos2) / 2.0;
            let dir = (pos1 - pos2).normalize();
            let mut midpoint_entity = transforms.get_mut(midpoint_entity_list[i].clone()).unwrap();

            //set midpoint entity to midpoint between pos1 and pos2
            midpoint_entity.translation = midpoint;

            //set rotation to be pointing in the direction of dir, moves from vector to "to" vec direction
            midpoint_entity.rotation = Quat::from_rotation_arc(segment_filler.vec_dir_segment, dir);
        }
    }
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}

pub fn procedural_animation_plugin(app: &mut App) {
    app.add_systems(PostStartup, setup_offset)
        .add_systems(Update, dynamic_body_calculator)
        .add_systems(Update, fabrik_calculator)
        .add_systems(Update, midpoint_filler);
}
