use bevy::{
    math::{
        VectorSpace,
        ops::{cos, sin},
    },
    mesh::VertexAttributeValues,
    prelude::*,
};

pub fn to_vec2(vec3: &Vec3) -> Vec2 {
    Vec2::new(vec3.x, vec3.y)
}

pub fn to_vec3(vec2: &Vec2) -> Vec3 {
    Vec3::new(vec2.x, vec2.y, 0.)
}

pub fn get_global_vertices(transform: &Transform, verticies: &[Vec2; 4]) -> [Vec2; 4] {
    let mut new_verticies: [Vec2; 4] = [Vec2::ZERO; 4];

    for (i, vertex) in verticies.iter().enumerate() {
        // Transform the vertex by applying rotation and translation
        let rotated = transform
            .rotation
            .mul_vec3(Vec3::new(vertex.x, vertex.y, 0.));
        let transformed_point = transform.translation + rotated;
        new_verticies[i] = to_vec2(&transformed_point);
    }

    return new_verticies;
}
