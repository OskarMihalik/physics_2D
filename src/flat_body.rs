use bevy::prelude::*;

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
            Shape::Circle { radius } => std::f32::consts::PI * radius * radius,
            Shape::Box { width, height } => width * height,
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
    Circle { radius: f32 },
    Box { width: f32, height: f32 },
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Circle { radius: 25.0 }
    }
}
