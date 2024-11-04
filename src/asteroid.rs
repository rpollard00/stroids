use crate::constants::*;
use crate::physics::Heading;
use crate::physics::RotationalVelocity;
use crate::physics::Velocity;
use crate::screen_edge_distance;
use crate::Despawning;
use crate::GameAssets;
use bevy::prelude::*;
use std::f32::consts::TAU;

use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::RngCore;

const ASTEROID_COLOR: Color = Color::WHITE;
const NUM_ASTEROIDS: i32 = 8;
const MAX_ASTEROID_SPEED: f32 = MAX_SPEED / 8.;
const MAX_ASTEROID_ROTATION_SPEED: f32 = TAU * 0.5;
const SAFE_RADIUS: f32 = 200.;
const LARGE_ASTEROID_SIZE: Vec2 = Vec2::new(128., 128.);
const MEDIUM_ASTEROID_SIZE: Vec2 = Vec2::new(64., 64.);
const SMALL_ASTEROID_SIZE: Vec2 = Vec2::new(32., 32.);

#[derive(Event)]
pub struct AsteroidDestroyedEvent(pub Entity, pub Transform, pub Velocity, pub AsteroidSize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum AsteroidSize {
    Small,
    Medium,
    Large,
}

#[derive(Component)]
pub struct Asteroid;

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
    pub fn new(
        size: AsteroidSize,
        position: Vec2,
        velocity: Vec2,
        rotational_velocity: f32,
        heading: f32,
        game_assets: &GameAssets,
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
                    custom_size: match size {
                        AsteroidSize::Small => Some(SMALL_ASTEROID_SIZE),
                        AsteroidSize::Medium => Some(MEDIUM_ASTEROID_SIZE),
                        AsteroidSize::Large => Some(LARGE_ASTEROID_SIZE),
                    },
                    ..default()
                },
                texture: match size {
                    AsteroidSize::Small => game_assets.asteroid_sm.clone(),
                    AsteroidSize::Medium => game_assets.asteroid_m.clone(),
                    AsteroidSize::Large => game_assets.asteroid_lg.clone(),
                },
                transform: Transform {
                    translation: position.extend(1.0),
                    rotation: Quat::from_rotation_z(heading),
                    ..default()
                },
                ..default()
            },
        }
    }
}

pub fn setup_asteroids(
    mut commands: Commands,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    game_assets: Res<GameAssets>,
) {
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
            &game_assets,
        ));
    }
}

pub fn asteroid_destroyed_listener(
    mut commands: Commands,
    mut asteroid_ev: EventReader<AsteroidDestroyedEvent>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    game_assets: Res<GameAssets>,
) {
    for ev in asteroid_ev.read() {
        let (entity, transform, velocity, size) = (ev.0, ev.1, ev.2, ev.3);

        _ = match size {
            AsteroidSize::Small => {
                commands.entity(entity).insert(Despawning);
            }
            AsteroidSize::Medium => {
                commands.entity(entity).insert(Despawning);
                commands.spawn(AsteroidBundle::new(
                    AsteroidSize::Small,
                    transform.translation.truncate(),
                    Vec2::new(velocity.0.y, velocity.0.x),
                    (rng.next_u32() as f32) % TAU,
                    0.0,
                    &game_assets,
                ));
                commands.spawn(AsteroidBundle::new(
                    AsteroidSize::Small,
                    transform.translation.truncate(),
                    Vec2::new(-velocity.0.y, -velocity.0.x),
                    (rng.next_u32() as f32) % TAU,
                    0.0,
                    &game_assets,
                ));
            }
            AsteroidSize::Large => {
                commands.entity(entity).insert(Despawning);
                commands.spawn(AsteroidBundle::new(
                    AsteroidSize::Medium,
                    transform.translation.truncate(),
                    Vec2::new(velocity.0.y, velocity.0.x),
                    (rng.next_u32() as f32) % TAU,
                    0.0,
                    &game_assets,
                ));
                commands.spawn(AsteroidBundle::new(
                    AsteroidSize::Medium,
                    transform.translation.truncate(),
                    Vec2::new(-velocity.0.y, -velocity.0.x),
                    (rng.next_u32() as f32) % TAU,
                    0.0,
                    &game_assets,
                ));
            }
        }
    }
}
