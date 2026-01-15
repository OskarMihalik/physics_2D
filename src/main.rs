use bevy::prelude::*;
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;

use crate::{
    flat_body::{FlatBodyParameters, Shape},
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .add_systems(Update, (move_physics_objects, spawn_physics_object))
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
        FlatBody::Dynamic(FlatBodyParameters::new(Shape::Circle { radius: 50.0 }, 1.)),
    ));
}

fn spawn_physics_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_position: Res<MyWorldCoords>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Spawn a new physics object at the cursor position
        // This is a placeholder for actual spawning logic
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.7, 0.7, 0.8))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::Dynamic(FlatBodyParameters::new(Shape::Circle { radius: 50.0 }, 1.)),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        let square_sprite = Sprite {
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::splat(100.0)),
            ..default()
        };

        // Ceiling
        commands.spawn((
            square_sprite.clone(),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::Dynamic(FlatBodyParameters::new(
                Shape::Box {
                    width: 100.0,
                    height: 100.0,
                },
                1.,
            )),
            // RigidBody::Static,
            // Collider::rectangle(50.0, 50.0),
        ));
    }
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
