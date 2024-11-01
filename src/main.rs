use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{plugin::EntropyPlugin, prelude::GlobalEntropy};
use rand_core::RngCore;

const WINDOW_WIDTH: f32 = 1024.;
const WINDOW_HEIGHT: f32 = 768.;
const MIN_X_POSITION: f32 = 0. - (WINDOW_WIDTH / 2.);
const MAX_X_POSITION: f32 = WINDOW_WIDTH / 2.;
const MIN_Y_POSITION: f32 = 0. - (WINDOW_HEIGHT / 2.);
const MAX_Y_POSITION: f32 = WINDOW_HEIGHT / 2.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stroids".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_event::<ProjectileFiredEvent>()
        .add_systems(Startup, (setup, setup_asteroids).chain())
        .add_systems(Update, (player_controls, projectile_spawner).chain())
        .add_systems(
            FixedUpdate,
            (apply_movement, speed_limit_system, out_of_bounds_system).chain(),
        )
        .add_systems(FixedUpdate, apply_rotational_velocity)
        .run();
}

const BACKGROUND_COLOR: Color = Color::BLACK;
const SHIP_COLOR: Color = Color::srgb(0.9, 0.0, 0.0);
const ASTEROID_COLOR: Color = Color::WHITE;
const PROJECTILE_COLOR: Color = Color::WHITE;

const SCREEN_CROSS_TIME: f32 = 1.5; // time in seconds to cross screen
const ACCELERATION_TIME: f32 = 1.0; // time to reach max acceleration
const ROTATION_TIME: f32 = 0.75; // full rotation time

const SHIP_SIZE: Vec2 = Vec2::new(40., 20.);
const MAX_SPEED: f32 = WINDOW_WIDTH / SCREEN_CROSS_TIME;
const MAX_SPEED_SQUARED: f32 = MAX_SPEED * MAX_SPEED;

const THRUST_POWER: f32 = MAX_SPEED / ACCELERATION_TIME;
const ROTATION_SPEED: f32 = TAU / ROTATION_TIME;

#[derive(Component)]
struct Player;

#[derive(Component, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component)]
struct RotationalVelocity(f32);

#[derive(Component, Clone, Copy)]
struct Heading(f32);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    velocity: Velocity,
    heading: Heading,
    sprite_bundle: SpriteBundle,
}

impl PlayerBundle {
    fn new() -> PlayerBundle {
        PlayerBundle {
            player: Player,
            velocity: Velocity(Vec2::new(0., 0.)),
            heading: Heading(0.25 * TAU),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: SHIP_COLOR,
                    ..default()
                },
                transform: Transform {
                    scale: SHIP_SIZE.extend(1.0),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Component)]
struct Projectile;

#[derive(Bundle)]
struct ProjectileBundle {
    projectile: Projectile,
    heading: Heading,
    start_velocity: Velocity,
    sprite_bundle: SpriteBundle,
}

#[derive(Event)]
struct ProjectileFiredEvent(Heading, Velocity, Vec2);

const PROJECTILE_BASE_VELOCITY: f32 = MAX_SPEED;
impl ProjectileBundle {
    fn new(heading: f32, start_velocity: Vec2, start: Vec2) -> ProjectileBundle {
        let heading_vec = Vec2::new(heading.cos(), heading.sin());
        let velo = start_velocity + heading_vec * PROJECTILE_BASE_VELOCITY;
        ProjectileBundle {
            projectile: Projectile,
            heading: Heading(heading),
            // ultimately heading doesn't really matter here, what we need is a velocity vector in
            // the right direction based on the ship's heading
            start_velocity: Velocity(velo),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: PROJECTILE_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: start.extend(1.0),
                    scale: Vec3::new(4., 4., 1.),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Component, Clone, Copy)]
enum AsteroidSize {
    Small,
    Medium,
    Large,
}

#[derive(Component)]
struct Asteroid;

#[derive(Bundle)]
struct AsteroidBundle {
    asteroid: Asteroid,
    size: AsteroidSize,
    velocity: Velocity,
    rotational_velocity: RotationalVelocity,
    heading: Heading,
    sprite_bundle: SpriteBundle,
}

impl AsteroidBundle {
    fn new(
        size: AsteroidSize,
        position: Vec2,
        velocity: Vec2,
        rotational_velocity: f32,
        heading: f32,
    ) -> AsteroidBundle {
        AsteroidBundle {
            asteroid: Asteroid,
            size,
            velocity: Velocity(velocity),
            rotational_velocity: RotationalVelocity(rotational_velocity),
            heading: Heading(heading),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: ASTEROID_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: position.extend(1.0),
                    rotation: Quat::from_rotation_z(heading),
                    scale: match size {
                        AsteroidSize::Small => Vec3::new(20., 20., 1.),
                        AsteroidSize::Medium => Vec3::new(60., 60., 1.),
                        AsteroidSize::Large => Vec3::new(100., 100., 1.),
                    },
                    ..default()
                },
                ..default()
            },
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: BACKGROUND_COLOR,
            ..default()
        },
        transform: Transform {
            scale: Vec3::new(WINDOW_WIDTH, WINDOW_HEIGHT, 0.0),
            ..default()
        },
        ..default()
    });
    commands.spawn(PlayerBundle::new());
}

const NUM_ASTEROIDS: i32 = 8;
const MAX_ASTEROID_SPEED: f32 = MAX_SPEED / 8.;
const MAX_ASTEROID_ROTATION_SPEED: f32 = TAU * 0.5;
const SAFE_RADIUS: f32 = 200.;

fn setup_asteroids(mut commands: Commands, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    for _ in 0..NUM_ASTEROIDS {
        // Random direction in radians
        let asteroid_direction = (rng.next_u32() as f32) % TAU;
        // Random direction as a unit vector
        let asteroid_direction_vec = Vec2::new(asteroid_direction.cos(), asteroid_direction.sin());

        // distance along ray to screen edge
        let screen_edge_distance = screen_edge_distance(&asteroid_direction_vec);

        // now I have a distance that represents the max length value
        let random_length =
            SAFE_RADIUS + ((rng.next_u32() as f32) % (screen_edge_distance - SAFE_RADIUS));
        let pos = asteroid_direction_vec * random_length;

        let x_velo: f32 =
            ((rng.next_u32() as f32) % (MAX_ASTEROID_SPEED * 2.)) - MAX_ASTEROID_SPEED;
        let y_velo: f32 =
            ((rng.next_u32() as f32) % (MAX_ASTEROID_SPEED * 2.)) - MAX_ASTEROID_SPEED;

        let heading: f32 = (rng.next_u32() as f32) % TAU;
        let random_rotational_velo: f32 = ((rng.next_u32() as f32) % MAX_ASTEROID_ROTATION_SPEED)
            - (0.5 * MAX_ASTEROID_ROTATION_SPEED);

        commands.spawn(AsteroidBundle::new(
            AsteroidSize::Large,
            pos,
            Vec2::new(x_velo, y_velo),
            random_rotational_velo,
            heading,
        ));
    }
}

fn screen_edge_distance(direction_norm: &Vec2) -> f32 {
    assert!(direction_norm.is_normalized());
    let abs_dir = direction_norm.abs();

    let x_edge_dist = MAX_X_POSITION / abs_dir.x;
    let y_edge_dist = MAX_Y_POSITION / abs_dir.y;

    x_edge_dist.min(y_edge_dist)
}

fn player_controls(
    mut query: Query<(&mut Velocity, &mut Heading, &Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ev_fire: EventWriter<ProjectileFiredEvent>,
) {
    let (mut velocity, mut heading, transform) = query.single_mut();

    if keys.pressed(KeyCode::KeyD) {
        heading.0 -= ROTATION_SPEED * time.delta_seconds();
    }
    if keys.pressed(KeyCode::KeyA) {
        heading.0 += ROTATION_SPEED * time.delta_seconds();
    }

    heading.0 = heading.0 % TAU;

    let heading_vec = Vec2::new(heading.0.cos(), heading.0.sin());

    if keys.just_pressed(KeyCode::Space) {
        let firing_start_pt = transform.scale.x * 0.5 + 5.;
        let ship_front = Vec2::new(
            heading_vec.x * firing_start_pt,
            heading_vec.y * firing_start_pt,
        );
        let projectile_location = ship_front + transform.translation.xy();
        ev_fire.send(ProjectileFiredEvent(
            Heading(heading.0),
            Velocity(velocity.0),
            projectile_location,
        ));
    }

    if keys.pressed(KeyCode::KeyW) {
        let thrust = heading_vec * THRUST_POWER * time.delta_seconds();
        velocity.0 += thrust;
    }
}

fn apply_movement(mut query: Query<(&mut Transform, &Heading, &Velocity)>, time: Res<Time>) {
    for (mut transform, heading, velocity) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(heading.0);
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
    }
}

fn apply_rotational_velocity(
    mut query: Query<(&mut Heading, &RotationalVelocity)>,
    time: Res<Time>,
) {
    for (mut heading, rotational_velocity) in query.iter_mut() {
        heading.0 += rotational_velocity.0 * time.delta_seconds();
        heading.0 = heading.0 % TAU;
    }
}

fn out_of_bounds_system(mut query: Query<&mut Transform>) {
    // we want to let things go out of bounds before moving them to prevent popping off the screen
    for mut transform in query.iter_mut() {
        let out_of_bound_offset = transform.scale.x.max(transform.scale.y);

        let min_x_position = MIN_X_POSITION - out_of_bound_offset;
        let max_x_position = MAX_X_POSITION + out_of_bound_offset;
        let min_y_position = MIN_Y_POSITION - out_of_bound_offset;
        let max_y_position = MAX_Y_POSITION + out_of_bound_offset;

        if transform.translation.x < min_x_position {
            transform.translation.x = max_x_position;
        }

        if transform.translation.x > max_x_position {
            transform.translation.x = min_x_position;
        }

        if transform.translation.y < min_y_position {
            transform.translation.y = max_y_position;
        }
        if transform.translation.y > max_y_position {
            transform.translation.y = min_y_position;
        }
    }
}

fn speed_limit_system(mut query: Query<&mut Velocity, With<Player>>) {
    for mut velo in query.iter_mut() {
        let speed_squared = velo.0.length_squared();
        if speed_squared > MAX_SPEED_SQUARED {
            velo.0 = velo.0.normalize() * MAX_SPEED;
        }
    }
}

fn projectile_spawner(mut command: Commands, mut ev_fire: EventReader<ProjectileFiredEvent>) {
    for ev in ev_fire.read() {
        let (heading, velocity, location) = (ev.0, ev.1, ev.2);
        command.spawn(ProjectileBundle::new(heading.0, velocity.0, location));
    }
}
