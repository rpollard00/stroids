use crate::constants::*;
use crate::AsteroidDestroyedEvent;
use crate::Despawning;
use crate::Player;
use crate::Projectile;

use crate::asteroid::*;

use bevy::prelude::*;
use std::f32::consts::TAU;

use bevy::math::bounding::{Aabb2d, IntersectsVolume};

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct RotationalVelocity(pub f32);

#[derive(Component, Clone, Copy)]
pub struct Heading(pub f32);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Collision;

pub fn apply_movement(mut query: Query<(&mut Transform, &Heading, &Velocity)>, time: Res<Time>) {
    for (mut transform, heading, velocity) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(heading.0);
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
    }
}

pub fn apply_rotational_velocity(
    mut query: Query<(&mut Heading, &RotationalVelocity)>,
    time: Res<Time>,
) {
    for (mut heading, rotational_velocity) in query.iter_mut() {
        heading.0 += rotational_velocity.0 * time.delta_seconds();
        heading.0 = heading.0 % TAU;
    }
}

pub fn out_of_bounds_system(mut query: Query<&mut Transform>) {
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

pub fn collision_system(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<Player>>,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    asteroid_query: Query<(Entity, &Transform, &Velocity, &AsteroidSize), With<Asteroid>>,
    mut asteroid_event: EventWriter<AsteroidDestroyedEvent>,
) {
    if let Ok((player, player_transform)) = player_query.get_single() {
        for (asteroid, asteroid_transform, asteroid_velocity, asteroid_size) in
            asteroid_query.iter()
        {
            // check for player asteroid collisions
            let player_collision = check_bb_collision(
                Aabb2d::new(
                    player_transform.translation.truncate(),
                    player_transform.scale.truncate() / 2.,
                ),
                Aabb2d::new(
                    asteroid_transform.translation.truncate(),
                    asteroid_transform.scale.truncate() / 2.,
                ),
            );

            if let Some(_) = player_collision {
                commands.entity(player).insert(Despawning);
            }

            for (projectile, projectile_transform) in projectile_query.iter() {
                let projectile_collision = check_bb_collision(
                    Aabb2d::new(
                        projectile_transform.translation.truncate(),
                        projectile_transform.scale.truncate() / 2.,
                    ),
                    Aabb2d::new(
                        asteroid_transform.translation.truncate(),
                        asteroid_transform.scale.truncate() / 2.,
                    ),
                );

                if let Some(_) = projectile_collision {
                    commands.entity(projectile).insert(Despawning);
                    asteroid_event.send(AsteroidDestroyedEvent(
                        asteroid,
                        *asteroid_transform,
                        *asteroid_velocity,
                        *asteroid_size,
                    ));
                }
            }
        }
    } else {
        return;
    }
}

fn check_bb_collision(collider: Aabb2d, collidee: Aabb2d) -> Option<Collision> {
    if !collider.intersects(&collidee) {
        None
    } else {
        Some(Collision)
    }
}
