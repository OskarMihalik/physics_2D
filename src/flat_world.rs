use crate::{
    collisions::{
        Collider, CollisionDetails, ContactPoints, Shape, find_contanct_points, intersect_aabbs,
        intersect_circle_circle, intersect_circle_polygon, intersects_polygons, separate_bodies,
    },
    flat_body::{FlatBody, FlatBodyType},
    helpers::{get_global_vertices, nearly_equal_vec, to_vec2},
};
use bevy::{
    color::palettes::css::LIME,
    ecs::query::QueryCombinationIter,
    math::{FloatPow, VectorSpace},
    prelude::*,
};

#[derive(Resource, Default)]
pub struct FlatWorld {
    pub gravity: Vec2,
    pub iterations: u32,
    pub body_count: usize,
    pub world_step_time_s: u128,
}

pub fn resolve_collision_basic(
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

pub fn resolve_collision_with_rotation(
    body_a: &mut FlatBody,
    transform_a: &Transform,
    body_b: &mut FlatBody,
    transform_b: &Transform,
    contact_points: &ContactPoints,
    normal: &Vec2,
    depth: f32,
) {
    let e = body_a.restitution.min(body_b.restitution);
    let mut impulses: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];
    let mut ra_list: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];
    let mut rb_list: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];

    for (i, contact_point) in contact_points.iter().enumerate() {
        let ra = contact_point - to_vec2(&transform_a.translation);
        let rb = contact_point - to_vec2(&transform_b.translation);

        let ra_perp = Vec2::new(-ra.y, ra.x);
        let rb_perp = Vec2::new(-rb.y, rb.x);

        let angular_linear_velocity_a = ra_perp * body_a.angular_velocity;
        let angular_linear_velocity_b = rb_perp * body_b.angular_velocity;

        let relative_velocity = (body_b.linear_velocity + angular_linear_velocity_b)
            - (body_a.linear_velocity + angular_linear_velocity_a);

        let contact_velocity_magnitude = relative_velocity.dot(*normal);

        if contact_velocity_magnitude > 0. {
            // bodies are already separating from each other
            continue;
        }

        let ra_perp_dot_n = ra_perp.dot(*normal);
        let rb_perp_dot_n = rb_perp.dot(*normal);

        let denom = body_a.inv_mass()
            + body_b.inv_mass()
            + (ra_perp_dot_n.squared() * body_a.inv_inertia())
            + (rb_perp_dot_n.squared() * body_b.inv_inertia());

        let mut j = -(1. + e) * contact_velocity_magnitude;
        j /= denom;
        j /= contact_points.len() as f32;

        let impulse = j * normal;
        impulses[i] = impulse;
        ra_list[i] = ra;
        rb_list[i] = rb;
    }

    for (i, impulse) in impulses.iter().enumerate() {
        body_a.linear_velocity += -impulse * body_a.inv_mass();
        body_a.angular_velocity += -ra_list[i].perp_dot(*impulse) * body_a.inv_inertia();
        body_b.linear_velocity += impulse * body_b.inv_mass();
        body_b.angular_velocity += rb_list[i].perp_dot(*impulse) * body_b.inv_inertia();
    }
}

pub fn resolve_collision_with_rotation_and_friction(
    body_a: &mut FlatBody,
    transform_a: &Transform,
    body_b: &mut FlatBody,
    transform_b: &Transform,
    contact_points: &ContactPoints,
    normal: &Vec2,
    depth: f32,
) {
    let e = body_a.restitution.min(body_b.restitution);
    let sf = (body_a.static_friction() + body_b.static_friction()) * 0.5;
    let df = (body_a.dynamic_friction() + body_b.dynamic_friction()) * 0.5;

    let mut friction_impulses: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];
    let mut impulses: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];
    let mut j_list: Vec<f32> = vec![0.0; contact_points.len()];
    let mut ra_list: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];
    let mut rb_list: Vec<Vec2> = vec![Vec2::ZERO; contact_points.len()];

    // bounce and rotation
    for (i, contact_point) in contact_points.iter().enumerate() {
        let ra = contact_point - to_vec2(&transform_a.translation);
        let rb = contact_point - to_vec2(&transform_b.translation);

        let ra_perp = Vec2::new(-ra.y, ra.x);
        let rb_perp = Vec2::new(-rb.y, rb.x);

        let angular_linear_velocity_a = ra_perp * body_a.angular_velocity;
        let angular_linear_velocity_b = rb_perp * body_b.angular_velocity;

        let relative_velocity = (body_b.linear_velocity + angular_linear_velocity_b)
            - (body_a.linear_velocity + angular_linear_velocity_a);

        let contact_velocity_magnitude = relative_velocity.dot(*normal);

        if contact_velocity_magnitude > 0. {
            // bodies are already separating from each other
            continue;
        }

        let ra_perp_dot_n = ra_perp.dot(*normal);
        let rb_perp_dot_n = rb_perp.dot(*normal);

        let denom = body_a.inv_mass()
            + body_b.inv_mass()
            + (ra_perp_dot_n.squared() * body_a.inv_inertia())
            + (rb_perp_dot_n.squared() * body_b.inv_inertia());

        let mut j = -(1. + e) * contact_velocity_magnitude;
        j /= denom;
        j /= contact_points.len() as f32;

        let impulse = j * normal;
        j_list[i] = j;
        impulses[i] = impulse;
        ra_list[i] = ra;
        rb_list[i] = rb;
    }

    for (i, impulse) in impulses.iter().enumerate() {
        body_a.linear_velocity += -impulse * body_a.inv_mass();
        body_a.angular_velocity += -ra_list[i].perp_dot(*impulse) * body_a.inv_inertia();
        body_b.linear_velocity += impulse * body_b.inv_mass();
        body_b.angular_velocity += rb_list[i].perp_dot(*impulse) * body_b.inv_inertia();
    }

    // friction
    for (i, contact_point) in contact_points.iter().enumerate() {
        let ra = contact_point - to_vec2(&transform_a.translation);
        let rb = contact_point - to_vec2(&transform_b.translation);

        let ra_perp = Vec2::new(-ra.y, ra.x);
        let rb_perp = Vec2::new(-rb.y, rb.x);

        let angular_linear_velocity_a = ra_perp * body_a.angular_velocity;
        let angular_linear_velocity_b = rb_perp * body_b.angular_velocity;

        let relative_velocity = (body_b.linear_velocity + angular_linear_velocity_b)
            - (body_a.linear_velocity + angular_linear_velocity_a);

        let tangent = relative_velocity - relative_velocity.dot(*normal) * *normal;

        if nearly_equal_vec(&tangent, &Vec2::ZERO) {
            continue;
        }

        let tangent = tangent.normalize();

        let ra_perp_dot_t = ra_perp.dot(tangent);
        let rb_perp_dot_t = rb_perp.dot(tangent);

        let denom = body_a.inv_mass()
            + body_b.inv_mass()
            + (ra_perp_dot_t.squared() * body_a.inv_inertia())
            + (rb_perp_dot_t.squared() * body_b.inv_inertia());

        let mut jt = -relative_velocity.dot(tangent);
        jt /= denom;
        jt /= contact_points.len() as f32;
        let j = j_list[i];

        if jt.abs() <= j * sf {
            let impulse_friction = jt * tangent;
            friction_impulses[i] = impulse_friction;
        } else {
            let impulse_friction = -j * tangent * df;
            friction_impulses[i] = impulse_friction;
        }
    }

    for (i, impulse) in friction_impulses.iter().enumerate() {
        body_a.linear_velocity += -impulse * body_a.inv_mass();
        body_a.angular_velocity += -ra_list[i].perp_dot(*impulse) * body_a.inv_inertia();
        body_b.linear_velocity += impulse * body_b.inv_mass();
        body_b.angular_velocity += rb_list[i].perp_dot(*impulse) * body_b.inv_inertia();
    }
}

pub fn collide(
    entity_a: (&Transform, &Collider),
    entity_b: (&Transform, &Collider),
) -> Option<CollisionDetails> {
    let (pos_a, collider_a) = entity_a;
    let (pos_b, collider_b) = entity_b;

    if let Shape::Box(box_params_a) = &collider_a.shape {
        if let Shape::Box(box_params_b) = &collider_b.shape {
            let vertices_a = get_global_vertices(&pos_a, &box_params_a.verticies);
            let vertices_b = get_global_vertices(&pos_b, &box_params_b.verticies);

            return intersects_polygons(
                &vertices_a,
                &to_vec2(&pos_a.translation),
                &vertices_b,
                &to_vec2(&pos_b.translation),
            );
        } else if let Shape::Circle(circle_params_b) = &collider_b.shape {
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
    } else if let Shape::Circle(circle_params_a) = &collider_a.shape {
        if let Shape::Box(box_params_b) = &collider_b.shape {
            let vertices_a = get_global_vertices(&pos_b, &box_params_b.verticies);

            return intersect_circle_polygon(
                &to_vec2(&pos_a.translation),
                circle_params_a.radius,
                &vertices_a,
                &to_vec2(&pos_b.translation),
            );
        } else if let Shape::Circle(circle_params_b) = &collider_b.shape {
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
    query: &mut Query<'_, '_, (Entity, &mut Transform, &mut FlatBody, &mut Collider)>,
    collision_entitties: &mut Vec<(Entity, Entity)>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, a2]) = combinations.fetch_next() {
        let (entity_a, mut transform_a, flat_body_a, mut collider_a) = a1;
        let (entity_b, mut transform_b, flat_body_b, mut collider_b) = a2;

        if let FlatBodyType::Static = flat_body_a.body_type
            && let FlatBodyType::Static = flat_body_b.body_type
        {
            continue;
        }

        if !intersect_aabbs(
            &collider_a.get_aabb(&transform_a),
            &collider_b.get_aabb(&transform_b),
        ) {
            continue;
        }

        collision_entitties.push((entity_a, entity_b));
    }
}

pub fn narrow_phase(
    query: &mut Query<'_, '_, (Entity, &mut Transform, &mut FlatBody, &mut Collider)>,
    collision_entitties: &Vec<(Entity, Entity)>,
) {
    for (entity_a, entity_b) in collision_entitties.iter() {
        let [
            (_entity_a, mut transform_a, mut flat_body_a, collider_a),
            (_entity_b, mut transform_b, mut flat_body_b, collider_b),
        ] = match query.get_many_mut([*entity_a, *entity_b]) {
            Ok(val) => val,
            Err(_) => continue,
        };

        let collision = collide((&transform_a, &collider_a), (&transform_b, &collider_b));
        if let Some(collision_info) = collision {
            separate_bodies(
                &mut transform_a,
                &mut transform_b,
                &flat_body_a,
                &flat_body_b,
                &collision_info,
            );
            let contact_points =
                find_contanct_points(&transform_a, &collider_a, &transform_b, &collider_b);

            resolve_collision_with_rotation_and_friction(
                &mut flat_body_a,
                &transform_a,
                &mut flat_body_b,
                &transform_b,
                &contact_points,
                &collision_info.collision_normal,
                collision_info.penetration_depth,
            )
        }
    }
}
