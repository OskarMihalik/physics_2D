use bevy::{prelude::*, reflect::Tuple};

use crate::{
    collisions::{Collider, compute_area},
    helpers::to_vec3,
};

#[derive(Component)]
pub enum FlatBody {
    Static(FlatBodyParameters),
    Dynamic(FlatBodyParameters),
}

#[derive(Default)]
pub struct FlatBodyParameters {
    pub linear_velocity: Vec2,
    pub rotation: f32,
    pub rotational_velocity: f32,
    // pub density: f32,
    // pub mass: f32,
    pub restitution: f32,
}

impl FlatBodyParameters {
    pub fn new(density: f32) -> Self {
        let mut params = FlatBodyParameters {
            // density,
            ..Default::default()
        };
        // params.mass = params.density * params.area;
        params
    }
}

impl Default for FlatBody {
    fn default() -> Self {
        FlatBody::Dynamic(FlatBodyParameters::default())
    }
}

pub struct CircleParams {
    pub radius: f32,
    pub area: f32,
}

impl CircleParams {
    pub fn new(radius: f32) -> Self {
        let area = std::f32::consts::PI * radius * radius;
        CircleParams { radius, area }
    }
}

pub struct BoxParams {
    pub width: f32,
    pub height: f32,
    pub area: f32,
    verticies: [Vec2; 4],
}

impl BoxParams {
    pub fn new(width: f32, height: f32) -> Self {
        let left = width / 2.;
        let right = left + width;
        let bottom = -height / 2.;
        let top = bottom + height;

        let verticies = [
            Vec2::new(left, top),
            Vec2::new(right, top),
            Vec2::new(left, bottom),
            Vec2::new(right, bottom),
        ];

        let area = width * height;

        BoxParams {
            width,
            height,
            verticies,
            area,
        }
    }

    // pub fn get_transformed_vertices(self, rotation)-> (Vec2, Vec2, Vec2, Vec2) {
    //     let mut transformed_verticies = self.verticies.clone();
    //     for vertex in self.verticies.iter().enumerate() {
    //         transformed_verticies[vertex.0].
    //     }
    //     return vertices
    // }
}

#[derive(Event)]
pub struct MoveFlatBody {
    pub entity: Entity,
    pub amount: Vec2,
}

pub fn on_move_flat_body(
    trigger: On<MoveFlatBody>,
    mut query: Query<(&mut Transform, &mut FlatBody)>,
) {
    let body = query.get_mut(trigger.entity);

    match body {
        Ok(mut body) => {
            body.0.translation += to_vec3(&trigger.amount);
        }
        _ => {}
    }
}

#[derive(Event)]
pub struct RotateFlatBody {
    pub entity: Entity,
    pub amount: f32,
}

pub fn on_rotate_flat_body(
    trigger: On<RotateFlatBody>,
    mut query: Query<(&mut Transform, &mut FlatBody)>,
) {
    let body = query.get_mut(trigger.entity);

    match body {
        Ok(mut body) => {
            body.0.rotate_z(trigger.amount);
            // if let FlatBody::Dynamic() =  {

            // }
        }
        _ => {}
    }
}
