use std::f32::consts::TAU;

use bevy::prelude::*;

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
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                player_movement,
                apply_player_movement,
                speed_limit_system,
                out_of_bounds_system,
            )
                .chain(),
        )
        .run();
}

const BACKGROUND_COLOR: Color = Color::BLACK;
const SHIP_COLOR: Color = Color::srgb(0.9, 0.0, 0.0);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Heading(f32);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    velocity: Velocity,
    heading: Heading,
    sprite_bundle: SpriteBundle,
}

const SHIP_SIZE: Vec2 = Vec2::new(40., 20.);

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

fn player_movement(
    mut query: Query<(&mut Velocity, &mut Heading), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut velocity, mut heading) = query.single_mut();

    if keys.pressed(KeyCode::KeyD) {
        heading.0 -= TAU * time.delta_seconds();
    }
    if keys.pressed(KeyCode::KeyA) {
        heading.0 += TAU * time.delta_seconds();
    }

    heading.0 = heading.0 % TAU;

    let heading_vec = Vec2::new(heading.0.cos(), heading.0.sin());

    if keys.pressed(KeyCode::KeyW) {
        let thrust = heading_vec * 10.0;
        velocity.0 += thrust;
    }
}

fn apply_player_movement(
    mut query: Query<(&mut Transform, &Heading, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    let (mut transform, heading, velocity) = query.single_mut();

    transform.rotation = Quat::from_rotation_z(heading.0);
    transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
}

fn out_of_bounds_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        if transform.translation.x < MIN_X_POSITION {
            transform.translation.x = MAX_X_POSITION;
        }

        if transform.translation.x > MAX_X_POSITION {
            transform.translation.x = MIN_X_POSITION;
        }

        if transform.translation.y < MIN_Y_POSITION {
            transform.translation.y = MAX_Y_POSITION;
        }
        if transform.translation.y > MAX_Y_POSITION {
            transform.translation.y = MIN_Y_POSITION;
        }
    }
}

const SPEED_LIMIT: f32 = 750.;
const SPEED_LIMIT_SQUARED: f32 = SPEED_LIMIT * SPEED_LIMIT;
fn speed_limit_system(mut query: Query<&mut Velocity>) {
    for mut velo in query.iter_mut() {
        let speed_squared = velo.0.length_squared();
        if speed_squared > SPEED_LIMIT_SQUARED {
            velo.0 = velo.0.normalize() * SPEED_LIMIT;
        }
    }
}
