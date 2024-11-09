use crate::check_for_collision;
use crate::constants::*;
use crate::AsteroidDestroyedEvent;
use crate::Despawning;
use crate::Hull;
use crate::Player;
use crate::PlayerKilledEvent;
use crate::Projectile;

use crate::asteroid::*;

use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Component, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct RotationalVelocity(pub f32);

#[derive(Component, Clone, Copy)]
pub struct Heading(pub f32);

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

pub fn screen_edge_distance(direction_norm: &Vec2) -> f32 {
    assert!(direction_norm.is_normalized());
    let abs_dir = direction_norm.abs();

    let x_edge_dist = MAX_X_POSITION / abs_dir.x;
    let y_edge_dist = MAX_Y_POSITION / abs_dir.y;

    x_edge_dist.min(y_edge_dist)
}

pub fn collision_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Hull), With<Player>>,
    projectile_query: Query<(Entity, &Transform, &Hull), With<Projectile>>,
    asteroid_query: Query<(Entity, &Transform, &Velocity, &AsteroidSize, &Hull), With<Asteroid>>,
    mut asteroid_event: EventWriter<AsteroidDestroyedEvent>,
    mut player_killed_event: EventWriter<PlayerKilledEvent>,
) {
    if let Ok((player_transform, player_hull)) = player_query.get_single() {
        for (asteroid, asteroid_transform, asteroid_velocity, asteroid_size, asteroid_hull) in
            asteroid_query.iter()
        {
            // check for player asteroid collisions
            let player_collision = check_for_collision(
                &player_hull,
                &player_transform,
                &asteroid_hull,
                &asteroid_transform,
            );

            if let Some(_) = player_collision {
                player_killed_event.send(PlayerKilledEvent);
            }

            for (projectile, projectile_transform, projectile_hull) in projectile_query.iter() {
                let projectile_collision = check_for_collision(
                    &projectile_hull,
                    &projectile_transform,
                    &asteroid_hull,
                    &asteroid_transform,
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
