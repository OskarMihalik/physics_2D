use bevy::math::Vec2;

pub struct CircleCollision {
    pub penetration_depth: f32,
    pub collision_normal: Vec2,
}

pub fn intersect_circle_circle(
    pos_a: Vec2,
    radius_a: f32,
    pos_b: Vec2,
    radius_b: f32,
) -> Option<CircleCollision> {
    let distance = pos_a.distance(pos_b);
    let radius_sum = radius_a + radius_b;

    if distance >= radius_sum {
        return None;
    }

    let normal = (pos_b - pos_a).normalize();

    let depth = radius_sum - distance;

    Some(CircleCollision {
        penetration_depth: depth,
        collision_normal: normal,
    })
}
