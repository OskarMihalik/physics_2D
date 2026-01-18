use bevy::{mesh::VertexAttributeValues, prelude::*};

pub fn to_vec2(vec3: &Vec3) -> Vec2 {
    Vec2::new(vec3.x, vec3.y)
}

pub fn to_vec3(vec2: &Vec2) -> Vec3 {
    Vec3::new(vec2.x, vec2.y, 0.)
}

pub fn get_global_vertices(
    mesh2d: &Mesh2d,
    meshes_res: &Res<'_, Assets<Mesh>>,
) -> Result<Vec<Vec3>, String> {
    let mesh = meshes_res
        .get(&mesh2d.0)
        .expect("mesh2d unavailable in meshes database");

    let attribute = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(attr) => attr,
        None => {
            info!("Mesh2d mesh asset doesn't have positions. WEIRD");
            return Err("Mesh2d mesh asset doesn't have positions. WEIRD".to_string());
        }
    };

    let positions = match attribute {
        &VertexAttributeValues::Float32x3(ref positions) => positions,
        _ => {
            info!("Mesh2d mesh asset has unexpected vertex attribute format");
            return Err("Mesh2d mesh asset has unexpected vertex attribute format".to_string());
        }
    };
    let vertices: Vec<Vec3> = positions
        .iter()
        .map(|pos| return Vec3::new(pos[0], pos[1], pos[2]))
        .collect();

    return Ok(vertices);
}
