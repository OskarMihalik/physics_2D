use bevy::prelude::*;

use crate::helpers::to_vec3;

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
    pub density: f32,
    pub mass: f32,
    pub restitution: f32,
    pub area: f32,
    pub shape: Shape,
}

impl FlatBodyParameters {
    pub fn compute_area(&mut self) {
        self.area = match &self.shape {
            Shape::Circle(radius) => std::f32::consts::PI * radius.radius * radius.radius,
            Shape::Box(BoxParams { width, height }) => width * height,
        };
    }
    pub fn new(shape: Shape, density: f32) -> Self {
        let mut params = FlatBodyParameters {
            shape,
            density,
            ..Default::default()
        };
        params.compute_area();
        params.mass = params.density * params.area;
        params
    }
}

impl Default for FlatBody {
    fn default() -> Self {
        FlatBody::Dynamic(FlatBodyParameters::default())
    }
}

pub enum Shape {
    Circle(CircleParams),
    Box(BoxParams),
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Circle(CircleParams { radius: 25.0 })
    }
}

pub struct CircleParams {
    pub radius: f32,
}

pub struct BoxParams {
    pub width: f32,
    pub height: f32,
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
            info!("moving")
        }
        _ => {}
    }
}
