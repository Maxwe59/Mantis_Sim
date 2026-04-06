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

//must run after dynamicbody, and after all range/angle constraints.
#[derive(Component)]
pub struct NodeOffsetter {
    nodes: Vec<Entity>,
    function: fn(i32) -> Vec3, //maps node translations given an i to a vec3
}

impl_new!(NodeOffsetter, nodes: Vec<Entity>, function: fn(i32) -> Vec3);
impl_new!(SegmentFiller, nodes: Vec<Entity>, midpoints: Vec<Entity>, vec_dir_segment: Vec3);
impl_new!(PivotEntity, head: Entity, offset: Vec3, child: Entity);
impl_new!(DynamicBody, seg_lengths: Vec<f32>, nodes: Vec<Entity>, angle_constraints: f32, lerp_speed: f32);
impl_new!(FabrikJoint, seg_lengths: Vec<f32>, nodes: Vec<Entity>, max_target_dist: f32, lerp_speed: f32, target_offset: Vec3, anchor_entity: Entity, fabrik_iterations: i32, stepping: bool, new_target_pos: Vec3, curr_target_pos: Vec3, t_val: f32);

impl DynamicBody {
    fn get_seg_len(&self) -> i32 {
        return self.seg_lengths.len() as i32;
    }
}

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

pub fn calc_segment_pos(
    dynamic_body_query: Query<&DynamicBody>,
    mut transforms: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
) {
    for dynamic_body in dynamic_body_query.iter() {
        let nodes = &dynamic_body.nodes;
        let segment_lengths = &dynamic_body.seg_lengths;

        let mut last_vec = global_transforms
            .get(dynamic_body.nodes[0])
            .unwrap()
            .translation();
        for (i, segment) in nodes.iter().skip(1).enumerate() {
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
        let mut last_vec = -1.0
            * (*global_transforms
                .get(dynamic_body.nodes[0])
                .unwrap()
                .forward());
        let nodes = &dynamic_body.nodes;
        let segment_lengths = &dynamic_body.seg_lengths;

        for i in 0..segment_lengths.len() {
            let front_pos = global_transforms
                .get(dynamic_body.nodes[i])
                .unwrap()
                .translation();
            let back_pos = global_transforms
                .get(dynamic_body.nodes[i + 1])
                .unwrap()
                .translation();

            let current_vec = (back_pos - front_pos).normalize();
            let angle = last_vec.angle_between(current_vec);
            let segment_to_change = nodes[i + 1].clone();
            let past_segment = nodes[i].clone();
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

        for i in 0..fabrik_joint.fabrik_iterations {
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

fn node_mutator(node_mutator_query: Query<&NodeOffsetter>, mut transforms: Query<&mut Transform>) {
    for node_mutator in node_mutator_query.iter() {
        let nodes = &node_mutator.nodes;
        let function = node_mutator.function;
        for (i, node) in nodes.iter().enumerate() {
            let offset = function(i as i32);
            let mut node_transform = transforms.get_mut(node.clone()).unwrap();
            node_transform.translation += offset;
        }
    }
}

fn distance_restraints(vec_static: Vec3, vec_to_move: Vec3, distance: f32) -> Vec3 {
    let dir = (vec_to_move - vec_static).normalize() * distance;
    return dir + vec_static;
}

pub fn procedural_animation_plugin(app: &mut App) {
    app.add_systems(PostStartup, setup_offset)
        .add_systems(
            Update,
            (angle_constraints, node_mutator, calc_segment_pos).chain(),
        )
        .add_systems(Update, fabrik_calculator)
        .add_systems(Update, midpoint_filler);
}
