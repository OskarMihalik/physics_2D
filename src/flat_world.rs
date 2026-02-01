use crate::{
    collisions::{
        Collider, CollisionDetails, intersect_circle_circle, intersect_circle_polygon,
        intersects_polygons,
    },
    flat_body::FlatBody,
    helpers::{get_global_vertices, to_vec2},
};
use bevy::mesh::Mesh2d;
use bevy::prelude::*;

#[derive(Resource)]
pub struct FlatWorld {
    pub gravity: Vec2,
    pub iterations: u32,
}

pub fn resolve_collision(
    body_a: &FlatBody,
    body_b: &FlatBody,
    normal: &Vec2,
    depth: f32,
) -> Option<(Vec2, Vec2)> {
    let relative_velocity = body_b.linear_velocity - body_a.linear_velocity;

    if relative_velocity.dot(*normal) > 0. {
        return None;
    }

    let e = body_a.restitution.min(body_b.restitution);

    let mut j = -(1. + e) * relative_velocity.dot(*normal);

    j /= body_a.inv_mass() + body_b.inv_mass();

    let impulse = j * normal;
    let impulse_a = -impulse * body_a.inv_mass();
    let impulse_b = impulse * body_b.inv_mass();

    return Some((impulse_a, impulse_b));
}

pub fn collide(
    entity_a: (&Transform, &Collider),
    entity_b: (&Transform, &Collider),
) -> Option<CollisionDetails> {
    let (pos_a, collider_a) = entity_a;
    let (pos_b, collider_b) = entity_b;

    if let Collider::Box(box_params_a) = collider_a {
        if let Collider::Box(box_params_b) = collider_b {
            let vertices_a = get_global_vertices(&pos_a, &box_params_a.verticies);
            let vertices_b = get_global_vertices(&pos_b, &box_params_b.verticies);
            return intersects_polygons(
                &vertices_a,
                &to_vec2(&pos_a.translation),
                &vertices_b,
                &to_vec2(&pos_b.translation),
            );
        } else if let Collider::Circle(circle_params_b) = collider_b {
            let vertices_a = get_global_vertices(&pos_a, &box_params_a.verticies);

            let mut collision = intersect_circle_polygon(
                &to_vec2(&pos_b.translation),
                circle_params_b.radius,
                &vertices_a,
                &to_vec2(&pos_a.translation),
            );

            if let Some(coll) = &mut collision {
                coll.collision_normal *= -1.;
            }
            return collision;
        }
    } else if let Collider::Circle(circle_params_a) = collider_a {
        if let Collider::Box(box_params_b) = collider_b {
            let vertices_a = get_global_vertices(&pos_b, &box_params_b.verticies);

            return intersect_circle_polygon(
                &to_vec2(&pos_a.translation),
                circle_params_a.radius,
                &vertices_a,
                &to_vec2(&pos_b.translation),
            );
        } else if let Collider::Circle(circle_params_b) = collider_b {
            return intersect_circle_circle(
                to_vec2(&pos_a.translation),
                circle_params_a.radius,
                to_vec2(&pos_b.translation),
                circle_params_b.radius,
            );
        }
    }

    return None;
}
