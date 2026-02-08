use bevy::prelude::*;

use crate::{collisions::Collider, helpers::to_vec3};

#[derive(Default, Debug)]
pub enum FlatBodyType {
    Static,
    #[default]
    Dynamic,
}

#[derive(Component, Default, Debug)]
#[require(Collider)]
pub struct FlatBody {
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    pub force: Vec2,
    pub restitution: f32,
    mass: f32,
    inv_mass: f32,
    inertia: f32,
    inv_inertia: f32,
    pub body_type: FlatBodyType,
}

impl FlatBody {
    pub fn new(mass: f32, body_type: FlatBodyType, restitution: f32) -> Self {
        let mut params = FlatBody {
            mass,
            body_type,
            restitution,
            ..Default::default()
        };
        // Setup inverse mass and restitution depending on body type.
        if let FlatBodyType::Static = params.body_type {
            params.inv_mass = 0.;
        } else {
            params.inv_mass = 1. / params.mass;
        }
        // params.mass = params.density * params.area;
        params
    }
    pub fn mass(&self) -> &f32 {
        &self.mass
    }

    pub fn inv_mass(&self) -> &f32 {
        &self.inv_mass
    }

    fn update_inertia(&mut self, collider: &Collider) {
        let inertia = calculate_rotational_inertia(collider, &self);
        if let FlatBodyType::Static = self.body_type {
            self.inertia = 0.;
            self.inv_inertia = 0.;
        } else {
            self.inertia = inertia;
            self.inv_inertia = 1. / inertia;
        }
    }

    pub fn inertia(&self) -> f32 {
        self.inertia
    }

    pub fn inf_inertia(&self) -> f32 {
        self.inv_inertia
    }
}

pub fn on_flat_body_added(
    event: On<Add, (FlatBody, Collider)>,
    mut query: Query<(&mut FlatBody, &Collider)>,
) {
    let (mut flat_body, collider) = match query.get_mut(event.entity) {
        Ok(ok) => ok,
        Err(_) => return,
    };
    flat_body.update_inertia(collider);
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

#[derive(Default)]
pub struct BoxParams {
    pub width: f32,
    pub height: f32,
    pub area: f32,
    pub verticies: [Vec2; 4],
}

impl BoxParams {
    pub fn new(width: f32, height: f32) -> Self {
        let left = -width / 2.;
        let right = left + width;
        let bottom = -height / 2.;
        let top = bottom + height;

        let verticies = [
            Vec2::new(left, top),
            Vec2::new(right, top),
            Vec2::new(right, bottom),
            Vec2::new(left, bottom),
        ];

        let area = width * height;

        BoxParams {
            width,
            height,
            verticies,
            area,
        }
    }
}

pub fn calculate_rotational_inertia(collider: &Collider, flat_body: &FlatBody) -> f32 {
    match collider {
        Collider::Box(box_params) => {
            return (1. / 12.)
                * flat_body.mass
                * (box_params.width.sqrt() + box_params.height.sqrt());
        }
        Collider::Circle(circle_params) => {
            return (1. / 2.) * flat_body.mass * circle_params.radius.sqrt();
        }
    }
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

pub fn handle_physics_step(
    transform: &mut Transform,
    flat_body: &mut FlatBody,
    gravity: &Vec2,
    delta_time: f32,
) {
    flat_body.linear_velocity += gravity * delta_time;
    transform.translation.x += flat_body.linear_velocity.x * delta_time;
    transform.translation.y += flat_body.linear_velocity.y * delta_time;

    let rotation_radians = flat_body.angular_velocity.to_radians();
    let current_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
    transform.rotation = Quat::from_euler(
        EulerRot::XYZ,
        0.0,
        0.0,
        current_rotation + rotation_radians * delta_time,
    );

    // Clear applied forces for next step
    flat_body.force = Vec2::ZERO;
}
