use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .add_systems(Update, (move_physics_objects))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let square_sprite = Sprite {
        color: Color::srgb(0.7, 0.7, 0.8),
        custom_size: Some(Vec2::splat(50.0)),
        ..default()
    };

    // Ceiling
    commands.spawn((
        square_sprite.clone(),
        Transform::from_xyz(0.0, 50.0 * 6.0, 0.0),
        // RigidBody::Static,
        // Collider::rectangle(50.0, 50.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.7, 0.7, 0.8))),
        Transform::from_xyz(0.0, 50.0 * 4.0, 0.0),
    ));
}

fn move_physics_objects(mut commands: Commands) {
    // Logic to move physics objects in 2D space
    // let square_sprite = Sprite {
    //     color: Color::srgb(0.7, 0.7, 0.8),
    //     custom_size: Some(Vec2::splat(50.0)),
    //     ..default()
    // };

    // // Ceiling
    // commands.spawn((
    //     square_sprite.clone(),
    //     Transform::from_xyz(0.0, 50.0 * 6.0, 0.0).with_scale(Vec3::new(20.0, 1.0, 1.0)),
    //     // RigidBody::Static,
    //     // Collider::rectangle(50.0, 50.0),
    // ));
}
