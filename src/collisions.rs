use core::f32;

use crate::helpers::{get_global_vertices, nearly_equal, nearly_equal_vec};
use crate::{
    flat_body::{
        BoxParams, CircleParams, FlatBody, FlatBodyType, on_move_flat_body, on_rotate_flat_body,
    },
    flat_manifold::FlatManifold,
    flat_world::{FlatWorld, collide, resolve_collision},
    helpers::{to_vec2, to_vec3},
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};
use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec3},
    ui::ValNum,
};
use bevy::{math::VectorSpace, prelude::*};

#[derive(Component)]
pub enum Collider {
    Box(BoxParams),
    Circle(CircleParams),
}

impl Default for Collider {
    fn default() -> Self {
        Collider::Box(BoxParams {
            width: 0.,
            height: 0.,
            area: 0.,
            verticies: [Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO],
        })
    }
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
    polygon_center: &Vec2,
) -> Option<CollisionDetails> {
    let mut normal = Vec2::ZERO;
    let mut depth = f32::MAX;
    for i in 0..vertices.len() {
        let va = vertices[i];
        let vb = vertices[(i + 1) % vertices.len()];

        let edge = vb - va;
        let axis = Vec2::new(-edge.y, edge.x).normalize();
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
    center_a: &Vec2,
    vertices_b: &[Vec2; 4],
    center_b: &Vec2,
) -> Option<CollisionDetails> {
    let mut normal = Vec2::ZERO;
    let mut depth = f32::MAX;
    for i in 0..vertices_a.len() {
        let va = vertices_a[i];
        let vb = vertices_a[(i + 1) % vertices_a.len()];

        let edge = vb - va;
        let axis = Vec2::new(-edge.y, edge.x).normalize();
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
        let axis = Vec2::new(-edge.x, edge.y).normalize();
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

pub fn handle_collision_step(
    transform_a: &mut Transform,
    transform_b: &mut Transform,
    flat_body_a: &mut FlatBody,
    flat_body_b: &mut FlatBody,
    collision_info: &crate::collisions::CollisionDetails,
) {
    if let FlatBodyType::Static = flat_body_a.body_type {
        transform_b.translation +=
            to_vec3(&(collision_info.collision_normal * collision_info.penetration_depth));
    } else if let FlatBodyType::Static = flat_body_b.body_type {
        transform_a.translation +=
            to_vec3(&(-collision_info.collision_normal * collision_info.penetration_depth));
    } else {
        transform_a.translation +=
            to_vec3(&(-collision_info.collision_normal * collision_info.penetration_depth / 2.));

        transform_b.translation +=
            to_vec3(&(collision_info.collision_normal * collision_info.penetration_depth / 2.));
    }
}

/// returns contact point
pub fn find_contanct_point(center_a: &Vec2, radius_a: f32, center_b: &Vec2) -> Vec2 {
    let ab = center_b - center_a;
    let direction = ab.normalize();
    let center_point = center_a + direction * radius_a;
    return center_point;
}

/// returns distance squared and contact point
fn point_segment_distance(p: &Vec2, a: &Vec2, b: &Vec2) -> (f32, Vec2) {
    let ab = b - a;
    let ap = p - a;

    let proj = ap.dot(ab);
    let ab_len_sq = ab.length_squared();
    let d = proj / ab_len_sq;

    let mut contact_point = Vec2::ZERO;
    if d <= 0. {
        contact_point = a.clone();
    } else if d >= 1. {
        contact_point = b.clone();
    } else {
        contact_point = a + ab * d;
    }

    let distance_squared = p.distance_squared(contact_point);

    return (distance_squared, contact_point);
}

fn find_contact_point_polygon_circle(
    circle_center: &Vec2,
    circle_radius: f32,
    polygon_center: &Vec2,
    polygon_vertices: &[Vec2; 4],
) -> ContactPoints {
    let mut cp = Vec2::ZERO;
    let mut min_distance_squared = f32::MAX;

    for i in 0..polygon_vertices.len() {
        let va = polygon_vertices[i];
        let vb = polygon_vertices[(i + 1) % polygon_vertices.len()];

        let (distance_squared, contact) = point_segment_distance(circle_center, &va, &vb);

        if distance_squared < min_distance_squared {
            min_distance_squared = distance_squared;
            cp = contact.clone();
        }
    }
    return ContactPoints::One(cp);
}

fn find_contact_points_polygon_polygon(
    vertices_a: &[Vec2; 4],
    vertices_b: &[Vec2; 4],
) -> ContactPoints {
    let mut contact1 = Vec2::ZERO;
    let mut contact2 = Vec2::ZERO;
    let mut min_dist_sq = f32::MAX;
    let mut contact_count = 0;

    for i in 0..vertices_a.len() {
        let p = vertices_a[i];
        for j in 0..vertices_b.len() {
            let va = vertices_b[j];
            let vb = vertices_b[(j + 1) % vertices_b.len()];

            let (distance_squared, contact_point) = point_segment_distance(&p, &va, &vb);

            if nearly_equal(distance_squared, min_dist_sq) {
                if !nearly_equal_vec(&contact_point, &contact1) {
                    contact2 = contact_point.clone();
                    contact_count = 2;
                }
            } else if distance_squared < min_dist_sq {
                min_dist_sq = distance_squared;
                contact1 = contact_point;
                contact_count = 1;
            }
        }
    }

    for i in 0..vertices_b.len() {
        let p = vertices_b[i];
        for j in 0..vertices_a.len() {
            let va = vertices_a[j];
            let vb = vertices_a[(j + 1) % vertices_b.len()];

            let (distance_squared, contact_point) = point_segment_distance(&p, &va, &vb);

            if nearly_equal(distance_squared, min_dist_sq) {
                if !nearly_equal_vec(&contact_point, &contact1) {
                    contact2 = contact_point.clone();
                    contact_count = 2;
                }
            } else if distance_squared < min_dist_sq {
                min_dist_sq = distance_squared;
                contact1 = contact_point;
                contact_count = 1;
            }
        }
    }

    if contact_count == 2 {
        return ContactPoints::Two((contact1, contact2));
    };

    return ContactPoints::One(contact1);
}

pub enum ContactPoints {
    One(Vec2),
    Two((Vec2, Vec2)),
}

pub fn find_contanct_points(
    trans_a: &Transform,
    collider_a: &Collider,
    trans_b: &Transform,
    collider_b: &Collider,
) -> ContactPoints {
    match (collider_a, collider_b) {
        (Collider::Box(box_params_a), Collider::Box(box_params_b)) => {
            let vertices_a = get_global_vertices(&trans_a, &box_params_a.verticies);
            let vertices_b = get_global_vertices(&trans_b, &box_params_b.verticies);
            return find_contact_points_polygon_polygon(&vertices_a, &vertices_b);
        }
        (Collider::Box(box_params_a), Collider::Circle(circle_params_b)) => {
            let pos_a = to_vec2(&trans_a.translation);
            let vertices_a = get_global_vertices(&trans_a, &box_params_a.verticies);
            return find_contact_point_polygon_circle(
                &to_vec2(&trans_b.translation),
                circle_params_b.radius,
                &pos_a,
                &vertices_a,
            );
        }
        (Collider::Circle(circle_params_a), Collider::Box(box_params_b)) => {
            let pos_b = to_vec2(&trans_b.translation);
            let vertices_b = get_global_vertices(&trans_b, &box_params_b.verticies);
            return find_contact_point_polygon_circle(
                &to_vec2(&trans_a.translation),
                circle_params_a.radius,
                &pos_b,
                &vertices_b,
            );
        }
        (Collider::Circle(circle_params_a), Collider::Circle(_circle_params_b)) => {
            let contact_point = find_contanct_point(
                &to_vec2(&trans_a.translation),
                circle_params_a.radius,
                &to_vec2(&trans_b.translation),
            );
            ContactPoints::One(contact_point)
        }
    }
}
