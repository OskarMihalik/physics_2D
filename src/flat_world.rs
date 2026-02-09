use crate::{
    collisions::{
        Collider, CollisionDetails, ContactPoints, find_contanct_points, intersect_circle_circle,
        intersect_circle_polygon, intersects_polygons, separate_bodies,
    },
    flat_body::{FlatBody, FlatBodyType},
    helpers::{get_global_vertices, to_vec2},
};
use bevy::{ecs::query::QueryCombinationIter, prelude::*};

#[derive(Resource, Default)]
pub struct FlatWorld {
    pub gravity: Vec2,
    pub iterations: u32,
    pub body_count: usize,
    pub world_step_time_s: u128,
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

pub fn broad_phase(
    query: &mut Query<'_, '_, (Entity, &mut Transform, &mut FlatBody, &Collider)>,
    collision_entitties: &mut Vec<(Entity, Entity, CollisionDetails, ContactPoints)>,
    iteration: u32,
    iterations: u32,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, a2]) = combinations.fetch_next() {
        let (entity_a, mut transform_a, flat_body_a, collider_a) = a1;
        let (entity_b, mut transform_b, flat_body_b, collider_b) = a2;

        let collision = collide((&transform_a, collider_a), (&transform_b, collider_b));

        if let FlatBodyType::Static = flat_body_a.body_type
            && let FlatBodyType::Static = flat_body_b.body_type
        {
            continue;
        }

        if let Some(collision_info) = collision {
            separate_bodies(
                &mut transform_a,
                &mut transform_b,
                &flat_body_a,
                &flat_body_b,
                &collision_info,
            );
            if iteration == iterations - 1 {
                let contact_points =
                    find_contanct_points(&transform_a, collider_a, &transform_b, collider_b);
                collision_entitties.push((entity_a, entity_b, collision_info, contact_points));
            }
        }
    }
}

pub fn narrow_phase(
    query: &mut Query<'_, '_, (Entity, &mut Transform, &mut FlatBody, &Collider)>,
    collision_entitties: &Vec<(Entity, Entity, CollisionDetails, ContactPoints)>,
) {
    for (entity_a, entity_b, collision_details, contact_points) in collision_entitties.iter() {
        let [
            (entity_a, mut transform_a, mut flat_body_a, collider_a),
            (entity_b, mut transform_b, mut flat_body_b, collider_b),
        ] = match query.get_many_mut([*entity_a, *entity_b]) {
            Ok(val) => val,
            Err(_) => continue,
        };

        let (impulse_a, impulse_b) = match resolve_collision(
            &flat_body_a,
            &flat_body_b,
            &collision_details.collision_normal,
            collision_details.penetration_depth,
        ) {
            Some((impulse_a, impulse_b)) => (impulse_a, impulse_b),
            None => continue,
        };

        flat_body_a.linear_velocity += impulse_a;
        flat_body_b.linear_velocity += impulse_b;
    }
}
