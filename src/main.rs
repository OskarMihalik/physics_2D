use std::time::{Duration, SystemTime};

use bevy::{color::palettes::css::WHITE, prelude::*};
mod flat_body;
mod mouse_position;
use flat_body::FlatBody;
mod collisions;
mod flat_aabb;
mod flat_world;
mod helpers;

use crate::{
    collisions::{Collider, Shape},
    flat_body::{BoxParams, CircleParams, FlatBodyType, handle_physics_step, on_flat_body_added},
    flat_world::{FlatWorld, broad_phase, narrow_phase},
    mouse_position::{MousePositionPlugin, MyWorldCoords},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MousePositionPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(FlatWorld {
            gravity: Vec2::new(0., -300.),
            iterations: 6,
            ..Default::default()
        })
        .insert_resource(DiagnosisConfig {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        })
        .add_systems(Startup, (setup, spawn_text_in_ui).chain())
        .add_systems(
            Update,
            (spawn_physics_object, diagnosis_ui, draw_line_for_circle),
        )
        .add_systems(FixedUpdate, (world_step).chain())
        .add_observer(on_flat_body_added)
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Ceiling
    commands.spawn((
        // square_sprite.clone(),
        Mesh2d(meshes.add(Rectangle::new(400.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(50.0 * -7., 50.0 * -3., 0.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.,
            0.,
            -0.3,
        )),
        FlatBody::new(1., FlatBodyType::Static, 0.5),
        Collider::new(Shape::Box(BoxParams::new(400., 30.))),
    ));

    commands.spawn((
        // square_sprite.clone(),
        Mesh2d(meshes.add(Rectangle::new(400.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(50.0 * 7., 50.0 * 2.0, 0.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.,
            0.,
            0.3,
        )),
        FlatBody::new(1., FlatBodyType::Static, 0.5),
        Collider::new(Shape::Box(BoxParams::new(400., 30.))),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(700.0, 30.))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_xyz(0.0, 50.0 * -9.0, 0.0),
        FlatBody::new(1., FlatBodyType::Static, 0.5),
        Collider::new(Shape::Box(BoxParams::new(700., 30.))),
    ));

    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
    //     MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
    //     Transform::from_xyz(0., 0., 0.0),
    //     FlatBody::new(1., FlatBodyType::Static, 2.),
    //     Collider::Box(BoxParams::new(100., 100.)),
    // ));
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
            // Collider::Circle(CircleParams::new(50.)),
            Collider::new(Shape::Circle(CircleParams::new(50.))),
        ));
    } else if buttons.just_pressed(MouseButton::Right) {
        // square
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.))),
            MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
            Transform::from_xyz(cursor_position.0.x, cursor_position.0.y, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 0.)),
            FlatBody::new(1., FlatBodyType::Dynamic, 0.5),
            Collider::new(Shape::Box(BoxParams::new(100., 100.))),
        ));
    }
}

fn world_step(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &mut Transform, &mut FlatBody, &mut Collider)>,
    mut flat_world: ResMut<FlatWorld>,
    mut collision_entitties: Local<Vec<(Entity, Entity)>>,
) {
    let world_step_start = SystemTime::now();
    flat_world.body_count = query.count();
    let delta_time_origin = fixed_time.delta_secs();

    for _iteration in 0..flat_world.iterations {
        let delta_time = delta_time_origin / (flat_world.iterations as f32);
        collision_entitties.clear();

        // physics step
        for (_entity, mut transform, mut flat_body, mut _collider) in query.iter_mut() {
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
        broad_phase(&mut query, &mut collision_entitties);
        // collision resolve
        narrow_phase(&mut query, &collision_entitties);
    }

    flat_world.world_step_time_s = world_step_start.elapsed().unwrap().as_micros();
}

fn draw_line_for_circle(
    query: Query<(Entity, &Transform, &FlatBody, &Collider)>,
    mut gizmos: Gizmos,
) {
    for (_entity, transform, _flat_body, collider) in query.iter() {
        if let Shape::Circle(circle_params) = &collider.shape {
            let rotated = transform.rotation * Vec3::X * circle_params.radius;
            gizmos.ray(transform.translation, rotated, WHITE);
        }
    }
}
