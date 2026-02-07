use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use bevy::{
    color::palettes::css::LIME,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod flat_manifold;
mod flat_world;
mod helpers;

use crate::{
    collisions::{
        Collider, CollisionDetails, ContactPoints, find_contanct_points, handle_collision_step,
    },
    flat_body::{
        BoxParams, CircleParams, FlatBodyType, handle_physics_step, on_move_flat_body,
        on_rotate_flat_body,
    },
    flat_world::{FlatWorld, collide, resolve_collision},
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(FlatWorld {
            gravity: Vec2::new(0., -309.81),
            iterations: 3,
            ..Default::default()
        })
        .insert_resource(DiagnosisConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        })
        .add_systems(Startup, (setup, spawn_text_in_ui).chain())
        .add_systems(Update, (spawn_physics_object, movement, diagnosis_ui))
        .add_systems(FixedUpdate, (world_step).chain())
        .add_observer(on_move_flat_body)
        .add_observer(on_rotate_flat_body)
        .run();
}

#[derive(Resource)]
struct DiagnosisConfig {
    /// How often to update ui?
    timer: Timer,
}

#[derive(Component)]
struct BodyCountText {}
#[derive(Component)]
struct StepTimeText {}

fn spawn_text_in_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,

                bottom: px(20.0),
                right: px(5.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Left),
        ))
        .with_children(|builder| {
            builder
                .spawn((Text::new("Body count: "),))
                .with_child((TextSpan::default(), BodyCountText {}));
            builder
                .spawn((Text::new("step time micros: "),))
                .with_child((TextSpan::default(), StepTimeText {}));
        });
}

fn diagnosis_ui(
    mut body_count_query: Query<&mut TextSpan, (With<BodyCountText>, Without<StepTimeText>)>,
    mut step_time_query: Query<&mut TextSpan, (With<StepTimeText>, Without<BodyCountText>)>,
    time: Res<Time>,
    mut diagnosis_config: ResMut<DiagnosisConfig>,
    flat_world: Res<FlatWorld>,
) {
    diagnosis_config.timer.tick(time.delta());

    if diagnosis_config.timer.just_finished() {
        for mut span in &mut body_count_query {
            // Update the value of the second section
            **span = format!("{}", flat_world.body_count);
        }
        for mut span in &mut step_time_query {
            // Update the value of the second section
            **span = format!("{}", flat_world.world_step_time_s);
        }
    }
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

fn world_step(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &mut Transform, &mut FlatBody, &Collider)>,
    mut flat_world: ResMut<FlatWorld>,
    mut collision_entitties: Local<
        HashMap<String, (Entity, Entity, CollisionDetails, ContactPoints)>,
    >,
    mut gizmos: Gizmos,
) {
    let world_step_start = SystemTime::now();
    flat_world.body_count = query.count();
    let delta_time_origin = fixed_time.delta_secs();

    for _ in 0..flat_world.iterations {
        let delta_time = delta_time_origin / (flat_world.iterations as f32);
        // physics step
        for (_entity, mut transform, mut flat_body, _) in query.iter_mut() {
            if let FlatBodyType::Static = flat_body.body_type {
                continue;
            }

            handle_physics_step(
                &mut transform,
                &mut flat_body,
                &flat_world.gravity,
                delta_time,
            );
        }

        // Collision step
        collision_entitties.clear();
        let mut combinations = query.iter_combinations_mut();
        while let Some([a1, a2]) = combinations.fetch_next() {
            let (entity_a, mut transform_a, mut flat_body_a, collider_a) = a1;
            let (entity_b, mut transform_b, mut flat_body_b, collider_b) = a2;

            let collision = collide((&transform_a, collider_a), (&transform_b, collider_b));

            if let FlatBodyType::Static = flat_body_a.body_type
                && let FlatBodyType::Static = flat_body_b.body_type
            {
                continue;
            }

            if let Some(collision_info) = collision {
                handle_collision_step(
                    &mut transform_a,
                    &mut transform_b,
                    &mut flat_body_a,
                    &mut flat_body_b,
                    &collision_info,
                );
                let contact_points =
                    find_contanct_points(&transform_a, collider_a, &transform_b, collider_b);
                collision_entitties.insert(
                    format!("{}-{}", entity_a, entity_b),
                    (entity_a, entity_b, collision_info, contact_points),
                );
            }
        }

        // collision resolve
        for (_id, (entity_a, entity_b, collision_details, contact_points)) in
            collision_entitties.iter()
        {
            let [
                (entity_a, mut transform_a, mut flat_body_a, collider_a),
                (entity_b, mut transform_b, mut flat_body_b, collider_b),
            ] = match query.get_many_mut([*entity_a, *entity_b]) {
                Ok(val) => val,
                Err(_) => continue,
            };

            let (impulse_a, impulse_b) = match resolve_collision(
                &flat_body_a,
                &flat_body_b,
                &collision_details.collision_normal,
                collision_details.penetration_depth,
            ) {
                Some((impulse_a, impulse_b)) => (impulse_a, impulse_b),
                None => continue,
            };
            if let ContactPoints::One(contact_point) = contact_points {
                gizmos.rect(
                    Isometry3d::new(
                        Vec3::new(contact_point.x, contact_point.y, 0.),
                        Quat::from_rotation_y(0.),
                    ),
                    Vec2::splat(10.),
                    LIME,
                );
            }
            if let ContactPoints::Two((contact_point1, contact_point2)) = contact_points {
                gizmos.rect(
                    Isometry3d::new(
                        Vec3::new(contact_point1.x, contact_point1.y, 0.),
                        Quat::from_rotation_y(0.),
                    ),
                    Vec2::splat(10.),
                    LIME,
                );
                gizmos.rect(
                    Isometry3d::new(
                        Vec3::new(contact_point2.x, contact_point2.y, 0.),
                        Quat::from_rotation_y(0.),
                    ),
                    Vec2::splat(10.),
                    LIME,
                );
            }

            flat_body_a.linear_velocity += impulse_a;
            flat_body_b.linear_velocity += impulse_b;
        }
    }

    flat_world.world_step_time_s = world_step_start.elapsed().unwrap().as_micros();
}
