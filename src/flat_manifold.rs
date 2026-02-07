use bevy::transform::components::Transform;

use crate::{collisions::CollisionDetails, flat_body::FlatBody};

pub struct FlatManifold {
    pub transform_a: Transform,
    pub transform_b: Transform,
    pub flat_body_a: FlatBody,
    pub flat_body_b: FlatBody,
    pub collision_info: CollisionDetails,
}
