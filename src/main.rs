use bevy::prelude::*;
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod flat_world;
mod helpers;

use crate::{
    collisions::Collider,
    flat_body::{BoxParams, CircleParams, on_move_flat_body, on_rotate_flat_body},
    flat_world::{FlatWorld, collide, resolve_collision},
    helpers::to_vec3,
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(FlatWorld {
            force_magnitude: 8.,
        })
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

    // Ceiling
    commands.spawn((
        // square_sprite.clone(),
        Mesh2d(meshes.add(Rectangle::new(1000.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(0.0, 50.0 * -11.0, 0.0),
        FlatBody {
            restitution: 1.,
            mass: 1.,
            ..Default::default()
        },
        Collider::Box(BoxParams::new(1000., 30.)),
    ));

    // commands.spawn((
    //     Mesh2d(meshes.add(Circle::new(50.0))),
    //     MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
    //     Transform::from_xyz(0., 0., 0.0),
    //     FlatBody::Dynamic(FlatBodyParameters::new(1.)),
    //     Collider::Circle(CircleParams::new(50.)),
    //     UserMovable {},
    // ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
        MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
        Transform::from_xyz(0., 0., 0.0),
        FlatBody {
            restitution: 1.,
            mass: 1.,
            ..Default::default()
        },
        Collider::Box(BoxParams::new(100., 100.)),
        UserMovable {},
    ));
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut controllable_flat_bodies_query: Query<(Entity, &mut FlatBody), With<UserMovable>>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let speed = 3.;
    let delta_time = time.delta_secs();
    // let entity_a = marbles.single().unwrap();
    for (_entity, mut flat_body) in &mut controllable_flat_bodies_query {
        if keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            flat_body.force = Vec2::Y * speed * delta_time;
        }
        if keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
            flat_body.force = -Vec2::Y * speed * delta_time;
        }
        if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
            flat_body.force = -Vec2::X * speed * delta_time;
        }
        if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
            flat_body.force = Vec2::X * speed * delta_time;
        }
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
            FlatBody {
                restitution: 1.,
                mass: 1.,
                ..Default::default()
            },
            Collider::Circle(CircleParams::new(50.)),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        // square
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
            MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody {
                restitution: 1.,
                mass: 1.,
                ..Default::default()
            },
            Collider::Box(BoxParams::new(100., 100.)),
        ));
    }
}

fn move_physics_objects(mut query: Query<(Entity, &mut Transform, &mut FlatBody)>) {
    for (_entity, mut transform, mut flat_body) in query.iter_mut() {
        // Update position based on linear velocity
        let acceleration = flat_body.force / flat_body.mass;
        flat_body.linear_velocity += acceleration;
        transform.translation.x += flat_body.linear_velocity.x;
        transform.translation.y += flat_body.linear_velocity.y;

        // Update rotation based on rotational velocity
        let rotation_radians = flat_body.rotational_velocity.to_radians();
        let current_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
        transform.rotation =
            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, current_rotation + rotation_radians);

        flat_body.force = Vec2::ZERO;
    }
}

fn collision_system(
    mut query: Query<(Entity, &mut Transform, &mut FlatBody, &Mesh2d, &Collider)>,
    // mut commands: Commands,
    // mut meshes: Res<Assets<Mesh>>,
    // mut gizmos: Gizmos,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, a2]) = combinations.fetch_next() {
        let (entity_a, mut transform_a, mut flat_body_a, mesh2d_a, collider_a) = a1;
        let (entity_b, mut transform_b, mut flat_body_b, mesh2d_b, collider_b) = a2;

        let collision = collide((&transform_a, collider_a), (&transform_b, collider_b));

        if let Some(collision_info) = collision {
            transform_a.translation += to_vec3(
                &(-collision_info.collision_normal * collision_info.penetration_depth / 2.),
            );

            transform_b.translation +=
                to_vec3(&(collision_info.collision_normal * collision_info.penetration_depth / 2.));

            let (move_a, move_b) = resolve_collision(
                &flat_body_a,
                &flat_body_b,
                &collision_info.collision_normal,
                collision_info.penetration_depth,
            );

            flat_body_a.linear_velocity += move_a;
            flat_body_b.linear_velocity += move_b;
        }
    }
}
