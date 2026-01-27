use core::f32;
use std::i16::MAX;

use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec3},
    ui::ValNum,
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

pub struct CollisionDetails {
    pub penetration_depth: f32,
    pub collision_normal: Vec2,
}

fn find_closes_point_on_polygon(circle_center: &Vec2, vertices: &[Vec2; 4]) -> Option<usize> {
    let mut result = None;
    let mut min_distance = f32::MAX;

    for (i, vertex) in vertices.iter().enumerate() {
        let distance = vertex.distance(*circle_center);
        if distance < min_distance {
            min_distance = distance;
            result = Some(i);
        }
    }
    result
}

fn project_circle(center: &Vec2, radius: f32, axis: &Vec2) -> (f32, f32) {
    let direction = axis.normalize();
    let direction_and_radius = direction * radius;

    let p1 = center + direction_and_radius;
    let p2 = center - direction_and_radius;

    let mut min = p1.dot(*axis);
    let mut max = p2.dot(*axis);

    if min > max {
        let temp = min;
        min = max;
        max = temp;
    }

    return (min, max);
}

pub fn intersect_circle_polygon(
    circle_center: &Vec2,
    circle_radius: f32,
    vertices: &[Vec2; 4],
) -> Option<CollisionDetails> {
    let mut normal = Vec2::ZERO;
    let mut depth = f32::MAX;
    for i in 0..vertices.len() {
        let va = vertices[i];
        let vb = vertices[(i + 1) % vertices.len()];

        let edge = vb - va;
        let axis = Vec2::new(-edge.x, edge.y).normalize();
        let (min_a, max_a) = project_vertices(vertices, &axis);
        let (min_b, max_b) = project_circle(circle_center, circle_radius, &axis);

        if min_a >= max_b || min_b >= max_a {
            return None;
        }

        let axis_depth = (max_b - min_a).min(max_a - min_b);
        if axis_depth < depth {
            depth = axis_depth;
            normal = axis;
        }
    }

    let cp_index = match find_closes_point_on_polygon(circle_center, vertices) {
        Some(value) => value,
        None => return None,
    };
    let cp = vertices[cp_index];
    let axis = (cp - circle_center).normalize();

    let (min_a, max_a) = project_vertices(vertices, &axis);
    let (min_b, max_b) = project_circle(circle_center, circle_radius, &axis);

    if min_a >= max_b || min_b >= max_a {
        return None;
    }

    let axis_depth = (max_b - min_a).min(max_a - min_b);
    if axis_depth < depth {
        depth = axis_depth;
        normal = axis;
    }

    // depth /= normal.length();
    // normal = normal.normalize();

    let polygon_center = find_vertices_center(vertices);

    let direction = polygon_center - circle_center;

    if direction.dot(normal) < 0. {
        normal = -normal;
    }

    return Some(CollisionDetails {
        penetration_depth: depth,
        collision_normal: normal,
    });
}

pub fn intersect_circle_circle(
    pos_a: Vec2,
    radius_a: f32,
    pos_b: Vec2,
    radius_b: f32,
) -> Option<CollisionDetails> {
    let distance = pos_a.distance(pos_b);
    let radius_sum = radius_a + radius_b;

    if distance >= radius_sum {
        return None;
    }

    let normal = (pos_b - pos_a).normalize();

    let depth = radius_sum - distance;

    Some(CollisionDetails {
        penetration_depth: depth,
        collision_normal: normal,
    })
}

pub fn intersects_polygons(
    vertices_a: &[Vec2; 4],
    vertices_b: &[Vec2; 4],
) -> Option<CollisionDetails> {
    let mut normal = Vec2::ZERO;
    let mut depth = f32::MAX;
    for i in 0..vertices_a.len() {
        let va = vertices_a[i];
        let vb = vertices_a[(i + 1) % vertices_a.len()];

        let edge = vb - va;
        let axis = Vec2::new(-edge.x, edge.y);
        let (min_a, max_a) = project_vertices(vertices_a, &axis);
        let (min_b, max_b) = project_vertices(vertices_b, &axis);

        if min_a >= max_b || min_b >= max_a {
            return None;
        }

        let axis_depth = (max_b - min_a).min(max_a - min_b);
        if axis_depth < depth {
            depth = axis_depth;
            normal = axis;
        }
    }

    for i in 0..vertices_b.len() {
        let va = vertices_b[i];
        let vb = vertices_b[(i + 1) % vertices_b.len()];

        let edge = vb - va;
        let axis = Vec2::new(-edge.x, edge.y);
        let (min_a, max_a) = project_vertices(vertices_a, &axis);
        let (min_b, max_b) = project_vertices(vertices_b, &axis);

        if min_a >= max_b || min_b >= max_a {
            return None;
        }

        let axis_depth = (max_b - min_a).min(max_a - min_b);
        if axis_depth < depth {
            depth = axis_depth;
            normal = axis;
        }
    }

    depth /= normal.length();
    normal = normal.normalize();

    let center_a = find_vertices_center(vertices_a);
    let center_b = find_vertices_center(vertices_b);

    let direction = center_b - center_a;

    if direction.dot(normal) < 0. {
        normal = -normal;
    }

    return Some(CollisionDetails {
        penetration_depth: depth,
        collision_normal: normal,
    });
}

fn find_vertices_center(vertices: &[Vec2; 4]) -> Vec2 {
    let mut sum_x = 0.;
    let mut sum_y = 0.;

    for vertex in vertices {
        sum_x += vertex.x;
        sum_y += vertex.y;
    }
    return Vec2::new(
        sum_x / vertices.len().val_num_f32(),
        sum_y / vertices.len().val_num_f32(),
    );
}

fn project_vertices(vertices: &[Vec2; 4], axis: &Vec2) -> (f32, f32) {
    let mut max = f32::MIN;
    let mut min = f32::MAX;

    for vertex in vertices {
        let projektion = vertex.dot(*axis);
        if projektion < min {
            min = projektion
        }
        if projektion > max {
            max = projektion
        }
    }
    return (min, max);
}
