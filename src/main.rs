use bevy::prelude::*;
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod flat_world;
mod helpers;

use crate::{
    collisions::Collider,
    flat_body::{BoxParams, CircleParams, FlatBodyType, on_move_flat_body, on_rotate_flat_body},
    flat_world::{FlatWorld, collide, resolve_collision},
    helpers::to_vec3,
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(FlatWorld {
            gravity: Vec2::new(0., -309.81),
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
        Mesh2d(meshes.add(Rectangle::new(700.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(0.0, 50.0 * -9.0, 0.0),
        FlatBody::new(1., FlatBodyType::Static, 0.5),
        Collider::Box(BoxParams::new(700., 30.)),
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
        FlatBody::new(1., FlatBodyType::Dynamic, 0.5),
        Collider::Box(BoxParams::new(100., 100.)),
        UserMovable {},
    ));

    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
    //     MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
    //     Transform::from_xyz(0., 0., 0.0),
    //     FlatBody::new(1., FlatBodyType::Static, 2.),
    //     Collider::Box(BoxParams::new(100., 100.)),
    // ));
}

fn movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut controllable_flat_bodies_query: Query<(Entity, &mut FlatBody), With<UserMovable>>,
) {
    // Treat `speed` as a force magnitude (units: force per second). We
    // accumulate directional input into a force vector here. The actual
    // acceleration is applied in the fixed-step integrator using delta time.
    let speed = 40.;
    for (_entity, mut flat_body) in &mut controllable_flat_bodies_query {
        let mut force = Vec2::ZERO;
        if keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            force += Vec2::Y;
        }
        if keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
            force += -Vec2::Y;
        }
        if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
            force += -Vec2::X;
        }
        if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
            force += Vec2::X;
        }

        // Set force for the physics integrator (force per second).
        flat_body.force = force.normalize_or_zero() * speed;
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
            FlatBody::new(1., FlatBodyType::Dynamic, 0.5),
            Collider::Circle(CircleParams::new(50.)),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        // square
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
            MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0),
            FlatBody::new(1., FlatBodyType::Dynamic, 0.5),
            Collider::Box(BoxParams::new(100., 100.)),
        ));
    }
}

fn move_physics_objects(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &mut Transform, &mut FlatBody)>,
    flat_world: Res<FlatWorld>,
) {
    let delta_time = fixed_time.delta_secs();
    for (_entity, mut transform, mut flat_body) in query.iter_mut() {
        if let FlatBodyType::Static = flat_body.body_type {
            continue;
        }
        // Update linear velocity using F = m * a -> a = F / m, integrated over delta_time
        // let mass = *flat_body.mass();
        // let acceleration = flat_body.force / mass;
        // flat_body.linear_velocity += acceleration * delta_time;

        // Integrate position using velocity * dt
        flat_body.linear_velocity += flat_world.gravity * delta_time;
        transform.translation.x += flat_body.linear_velocity.x * delta_time;
        transform.translation.y += flat_body.linear_velocity.y * delta_time;

        // Update rotation based on rotational velocity (degrees per second -> radians per second)
        let rotation_radians = flat_body.rotational_velocity.to_radians();
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
}

fn collision_system(
    mut query: Query<(Entity, &mut Transform, &mut FlatBody, &Mesh2d, &Collider)>,
    // mut commands: Commands,
    // mut meshes: Res<Assets<Mesh>>,
    // mut gizmos: Gizmos,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([a1, a2]) = combinations.fetch_next() {
        let (_entity_a, mut transform_a, mut flat_body_a, _mesh2d_a, collider_a) = a1;
        let (_entity_b, mut transform_b, mut flat_body_b, _mesh2d_b, collider_b) = a2;

        let collision = collide((&transform_a, collider_a), (&transform_b, collider_b));

        if let FlatBodyType::Static = flat_body_a.body_type
            && let FlatBodyType::Static = flat_body_b.body_type
        {
            continue;
        }

        if let Some(collision_info) = collision {
            if let FlatBodyType::Static = flat_body_a.body_type {
                transform_b.translation +=
                    to_vec3(&(collision_info.collision_normal * collision_info.penetration_depth));
            } else if let FlatBodyType::Static = flat_body_b.body_type {
                transform_a.translation +=
                    to_vec3(&(-collision_info.collision_normal * collision_info.penetration_depth));
            } else {
                transform_a.translation += to_vec3(
                    &(-collision_info.collision_normal * collision_info.penetration_depth / 2.),
                );

                transform_b.translation += to_vec3(
                    &(collision_info.collision_normal * collision_info.penetration_depth / 2.),
                );
            }

            let (impulse_a, impulse_b) = match resolve_collision(
                &flat_body_a,
                &flat_body_b,
                &collision_info.collision_normal,
                collision_info.penetration_depth,
            ) {
                Some((impulse_a, impulse_b)) => (impulse_a, impulse_b),
                None => continue,
            };
            info!("collision impulses: {} {}", impulse_a, impulse_b);

            flat_body_a.linear_velocity += impulse_a;
            flat_body_b.linear_velocity += impulse_b;
        }
    }
}
