use std::ops::Deref;

use bevy::prelude::*;
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod helpers;

use crate::{
    collisions::intersect_circle_circle,
    flat_body::{
        BoxParams, CircleParams, FlatBodyParameters, MoveFlatBody, Shape, on_move_flat_body,
    },
    helpers::to_vec2,
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_physics_object, movement))
        .add_systems(
            FixedUpdate,
            (move_physics_objects, collision_system).chain(),
        )
        .add_observer(on_move_flat_body)
        .run();
}

#[derive(Component)]
struct UserMovable {}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let square_sprite = Sprite {
        color: Color::srgb(0.7, 0.7, 0.8),
        custom_size: Some(Vec2::new(1000.0, 30.0)),
        ..default()
    };

    // Ceiling
    commands.spawn((
        square_sprite.clone(),
        Transform::from_xyz(0.0, 50.0 * -11.0, 0.0),
        // RigidBody::Static,
        // Collider::rectangle(50.0, 50.0),
        FlatBody::Static(FlatBodyParameters::new(
            Shape::Box(BoxParams {
                width: 1000.0,
                height: 30.0,
            }),
            0.,
        )),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
        Transform::from_xyz(0., 0., 0.0),
        FlatBody::Dynamic(FlatBodyParameters::new(
            Shape::Circle(CircleParams { radius: 50.0 }),
            1.,
        )),
        UserMovable {},
    ));
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    marbles: Query<Entity, With<UserMovable>>,
    mut commands: Commands,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs();
    let entity_a = marbles.single().unwrap();
    if keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        // Use a higher acceleration for upwards movement to overcome gravity
        commands.trigger(MoveFlatBody {
            entity: entity_a,
            amount: Vec2::Y * delta_time * 100.,
        });
    }
    if keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        commands.trigger(MoveFlatBody {
            entity: entity_a,
            amount: -Vec2::Y * delta_time * 100.,
        });
    }
    if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        commands.trigger(MoveFlatBody {
            entity: entity_a,
            amount: -Vec2::X * delta_time * 100.,
        });
    }
    if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        commands.trigger(MoveFlatBody {
            entity: entity_a,
            amount: Vec2::X * delta_time * 100.,
        });
    }
}

fn spawn_physics_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_position: Res<MyWorldCoords>,
) {
    let random_red = rand::random::<f32>();
    let random_green = rand::random::<f32>();
    let random_blue = rand::random::<f32>();
    if buttons.just_pressed(MouseButton::Left) {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(Color::srgb(random_red, random_green, random_blue))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::Dynamic(FlatBodyParameters::new(
                Shape::Circle(CircleParams { radius: 50.0 }),
                1.,
            )),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        let square_sprite = Sprite {
            color: Color::srgb(random_red, random_green, random_blue),
            custom_size: Some(Vec2::splat(100.0)),
            ..default()
        };

        // Ceiling
        commands.spawn((
            square_sprite.clone(),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::Dynamic(FlatBodyParameters::new(
                Shape::Box(BoxParams {
                    width: 100.0,
                    height: 100.0,
                }),
                1.,
            )),
        ));
    }
}

fn move_physics_objects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FlatBody)>,
) {
    for (entity, mut transform, mut flat_body) in query.iter_mut() {
        match &mut *flat_body {
            FlatBody::Dynamic(params) => {
                // Update position based on linear velocity
                transform.translation.x += params.linear_velocity.x;
                transform.translation.y += params.linear_velocity.y;

                // Update rotation based on rotational velocity
                let rotation_radians = params.rotational_velocity.to_radians();
                let current_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
                transform.rotation =
                    Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, current_rotation + rotation_radians);
            }
            FlatBody::Static(_) => {
                // Static bodies do not move
            }
        }
    }
}

fn collision_system(mut query: Query<(Entity, &Transform, &FlatBody)>, mut commands: Commands) {
    let entities: Vec<(Entity, &Transform, &FlatBody)> = query.iter().collect();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (entity_a, pos_a, flat_body_a) = entities[i];
            let (entity_b, pos_b, flat_body_b) = entities[j];

            let dynamic = match (flat_body_a, flat_body_b) {
                (
                    FlatBody::Dynamic(flat_body_dynamic_a),
                    FlatBody::Dynamic(flat_body_dynamic_b),
                ) => match (&flat_body_dynamic_a.shape, &flat_body_dynamic_b.shape) {
                    (Shape::Circle(circle_a), Shape::Circle(circle_b)) => {
                        let res = intersect_circle_circle(
                            to_vec2(&pos_a.translation),
                            circle_a.radius,
                            to_vec2(&pos_b.translation),
                            circle_b.radius,
                        );
                        if let Some(res) = res {
                            commands.trigger(MoveFlatBody {
                                entity: entity_a,
                                amount: -res.collision_normal * res.penetration_depth / 2.,
                            });
                            commands.trigger(MoveFlatBody {
                                entity: entity_b,
                                amount: res.collision_normal * res.penetration_depth / 2.,
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            };
        }
    }
}
