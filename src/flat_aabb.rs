use bevy::math::Vec2;

pub struct FlatAABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl FlatAABB {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        FlatAABB {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }
}

impl Default for FlatAABB {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}
