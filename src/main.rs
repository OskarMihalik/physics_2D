use std::ops::Deref;

use bevy::{math::VectorSpace, mesh::VertexAttributeValues, prelude::*};
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod helpers;

use crate::{
    collisions::{Collider, intersect_circle_circle, intersects_polygons},
    flat_body::{
        BoxParams, CircleParams, FlatBodyParameters, MoveFlatBody, on_move_flat_body,
        on_rotate_flat_body,
    },
    helpers::{get_global_vertices, to_vec2},
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
        .add_observer(on_rotate_flat_body)
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
        // square_sprite.clone(),
        Mesh2d(meshes.add(Rectangle::new(1000.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(0.0, 50.0 * -11.0, 0.0),
        FlatBody::Static(FlatBodyParameters::new(0.)),
        Collider::Box(BoxParams::new(1000., 30.)),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
        Transform::from_xyz(0., 0., 0.0),
        FlatBody::Dynamic(FlatBodyParameters::new(1.)),
        Collider::Circle(CircleParams::new(50.)),
        UserMovable {},
    ));

    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
    //     MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
    //     Transform::from_xyz(0., 0., 0.0),
    //     FlatBody::Dynamic(FlatBodyParameters::new(1.)),
    //     Collider::Box(BoxParams::new(100., 100.)),
    //     UserMovable {},
    // ));
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
            FlatBody::Dynamic(FlatBodyParameters::new(1.)),
            Collider::Circle(CircleParams::new(50.)),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        // square
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
            MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::Dynamic(FlatBodyParameters::new(1.)),
            Collider::Box(BoxParams::new(100., 100.)),
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
                transform.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    0.0,
                    0.0,
                    current_rotation + 0.5_f32.to_radians(),
                );
            }
            FlatBody::Static(_) => {
                // Static bodies do not move
            }
        }
    }
}

fn collision_system(
    mut query: Query<(Entity, &Transform, &FlatBody, &Mesh2d, &Collider)>,
    mut commands: Commands,
    mut meshes: Res<Assets<Mesh>>,
) {
    let entities: Vec<(Entity, &Transform, &FlatBody, &Mesh2d, &Collider)> = query.iter().collect();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (entity_a, pos_a, flat_body_a, mesh2d_a, collider_a) = entities[i];
            let (entity_b, pos_b, flat_body_b, mesh2d_b, collider_b) = entities[j];

            // match flat_body_a {
            //     FlatBody::Static(flat_body_parameters) => {
            //         info!("static asss");
            //         if let Shape::Box(shape) = &flat_body_parameters.shape {
            //             info!("{:#?}", positions);
            //         }
            //     }
            //     _ => (),
            // }

            match (flat_body_a, flat_body_b) {
                (
                    FlatBody::Dynamic(flat_body_dynamic_a),
                    FlatBody::Dynamic(flat_body_dynamic_b),
                ) => match (&collider_a, &collider_b) {
                    (Collider::Circle(circle_a), Collider::Circle(circle_b)) => {
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
                    (Collider::Box(box_params_a), Collider::Box(box_params_b)) => {
                        let vertices_a = get_global_vertices(mesh2d_a, &meshes).unwrap();
                        let vertices_b = get_global_vertices(mesh2d_b, &meshes).unwrap();
                        info!("{:?} {:?}", vertices_a, vertices_b);
                        if intersects_polygons(&vertices_a, &vertices_b) {
                            info!("box collision");
                        }
                    }
                    _ => {}
                },
                (
                    FlatBody::Static(flat_body_parameters_a),
                    FlatBody::Static(flat_body_parameters_b),
                ) => {
                    // info!("collision static static")
                }
                (
                    FlatBody::Static(flat_body_parameters_a),
                    FlatBody::Dynamic(flat_body_parameters_b),
                ) => {
                    // info!("collision static dynamic")
                }
                (
                    FlatBody::Dynamic(flat_body_parameters_a),
                    FlatBody::Static(flat_body_parameters_b),
                ) => {
                    // info!("collision dynamic static")
                }
            };
        }
    }
}
