use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec3},
};

use crate::flat_body::{BoxParams, CircleParams};

#[derive(Component)]
pub enum Collider {
    Box(BoxParams),
    Circle(CircleParams),
}

pub fn compute_area(collider: &Collider) -> f32 {
    return match collider {
        Collider::Circle(radius) => std::f32::consts::PI * radius.radius * radius.radius,
        Collider::Box(box_params) => box_params.width * box_params.height,
    };
}

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

pub fn intersects_polygons(vertices_a: &Vec<Vec3>, vertices_b: &Vec<Vec3>) -> bool {
    for i in 0..vertices_a.len() {
        let va = vertices_a[i];
        let vb = vertices_a[(i + 1) % vertices_a.len()];

        let edge = vb - va;
        let axis = Vec3::new(-edge.x, edge.y, 0.);
        let (min_a, max_a) = project_vertices(vertices_a, &axis);
        let (min_b, max_b) = project_vertices(vertices_b, &axis);

        if min_a >= max_b || min_b >= max_a {
            return false;
        }
    }

    for i in 0..vertices_b.len() {
        let va = vertices_b[i];
        let vb = vertices_b[(i + 1) % vertices_b.len()];

        let edge = vb - va;
        let axis = Vec3::new(-edge.x, edge.y, 0.);
        let (min_a, max_a) = project_vertices(vertices_a, &axis);
        let (min_b, max_b) = project_vertices(vertices_b, &axis);

        if min_a >= max_b || min_b >= max_a {
            return false;
        }
    }

    return true;
}

fn project_vertices(vertices: &Vec<Vec3>, axis: &Vec3) -> (f32, f32) {
    let mut max = f32::MAX;
    let mut min = f32::MIN;

    for vertex in vertices {
        let projektion = axis.dot(*vertex);
        if projektion < min {
            min = projektion
        }
        if projektion > max {
            max = projektion
        }
    }
    return (min, max);
}
