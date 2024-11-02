use crate::constants::*;
use crate::physics::Heading;
use crate::physics::Velocity;
use crate::Despawning;
use bevy::prelude::*;
use std::f32::consts::TAU;

const ACCELERATION_TIME: f32 = 1.0; // time to reach max acceleration
const ROTATION_TIME: f32 = 0.75; // full rotation time
const THRUST_POWER: f32 = MAX_SPEED / ACCELERATION_TIME;
const ROTATION_SPEED: f32 = TAU / ROTATION_TIME;
const SHIP_SIZE: Vec2 = Vec2::new(40., 20.);
const PROJECTILE_BASE_VELOCITY: f32 = MAX_SPEED;
const SHIP_COLOR: Color = Color::srgb(0.9, 0.0, 0.0);
const PROJECTILE_COLOR: Color = Color::WHITE;

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub velocity: Velocity,
    pub heading: Heading,
    pub sprite_bundle: SpriteBundle,
}

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct TravelDistance {
    current: f32,
    max: f32,
}

#[derive(Bundle)]
struct ProjectileBundle {
    projectile: Projectile,
    heading: Heading,
    start_velocity: Velocity,
    travel_distance: TravelDistance,
    sprite_bundle: SpriteBundle,
}

#[derive(Event)]
pub struct ProjectileFiredEvent(Heading, Velocity, Vec2);

impl PlayerBundle {
    pub fn new() -> PlayerBundle {
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

impl ProjectileBundle {
    pub fn new(heading: f32, start_velocity: Vec2, start: Vec2) -> ProjectileBundle {
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
            travel_distance: TravelDistance {
                current: 0.,
                max: WINDOW_WIDTH,
            },
        }
    }
}

pub fn player_controls(
    mut query: Query<(&mut Velocity, &mut Heading, &Transform), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ev_fire: EventWriter<ProjectileFiredEvent>,
) {
    if let Ok((mut velocity, mut heading, transform)) = query.get_single_mut() {
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
}

pub fn speed_limit_system(mut query: Query<&mut Velocity, With<Player>>) {
    for mut velo in query.iter_mut() {
        let speed_squared = velo.0.length_squared();
        if speed_squared > MAX_SPEED_SQUARED {
            velo.0 = velo.0.normalize() * MAX_SPEED;
        }
    }
}

pub fn projectile_spawner(mut commands: Commands, mut ev_fire: EventReader<ProjectileFiredEvent>) {
    for ev in ev_fire.read() {
        let (heading, velocity, location) = (ev.0, ev.1, ev.2);
        commands.spawn(ProjectileBundle::new(heading.0, velocity.0, location));
    }
}

pub fn distance_tracker(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TravelDistance, &Velocity)>,
    time: Res<Time>,
) {
    for (entity, mut travel_distance, velocity) in query.iter_mut() {
        travel_distance.current += velocity.0.length() * time.delta_seconds();

        if travel_distance.current > travel_distance.max {
            commands.entity(entity).insert(Despawning);
        }
    }
}
