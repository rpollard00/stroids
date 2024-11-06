use crate::constants::*;
use crate::AsteroidDestroyedEvent;
use crate::CollisionHulls;
use crate::Despawning;
use crate::GameAssets;
use crate::GameState;
use crate::Hull;
use crate::Player;
use crate::PlayerKilledEvent;
use crate::Projectile;

use crate::asteroid::*;

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
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

pub fn screen_edge_distance(direction_norm: &Vec2) -> f32 {
    assert!(direction_norm.is_normalized());
    let abs_dir = direction_norm.abs();

    let x_edge_dist = MAX_X_POSITION / abs_dir.x;
    let y_edge_dist = MAX_Y_POSITION / abs_dir.y;

    x_edge_dist.min(y_edge_dist)
}

pub fn collision_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    asteroid_query: Query<(Entity, &Transform, &Velocity, &AsteroidSize, &Sprite), With<Asteroid>>,
    mut asteroid_event: EventWriter<AsteroidDestroyedEvent>,
    mut player_killed_event: EventWriter<PlayerKilledEvent>,
) {
    if let Ok((player_transform, player_sprite)) = player_query.get_single() {
        for (asteroid, asteroid_transform, asteroid_velocity, asteroid_size, asteroid_sprite) in
            asteroid_query.iter()
        {
            // check for player asteroid collisions
            let player_collision = check_bb_collision(
                Aabb2d::new(
                    player_transform.translation.truncate(),
                    player_sprite.custom_size.unwrap() / 2.,
                ),
                Aabb2d::new(
                    asteroid_transform.translation.truncate(),
                    asteroid_sprite.custom_size.unwrap() / 2.,
                ),
            );

            if let Some(_) = player_collision {
                player_killed_event.send(PlayerKilledEvent);
            }

            for (projectile, projectile_transform) in projectile_query.iter() {
                let projectile_collision = check_bb_collision(
                    Aabb2d::new(
                        projectile_transform.translation.truncate(),
                        projectile_transform.scale.truncate() / 2.,
                    ),
                    Aabb2d::new(
                        asteroid_transform.translation.truncate(),
                        asteroid_sprite.custom_size.unwrap() / 2.,
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

// #[derive(Resource, Default, Clone)]
// pub struct CollisionHulls {
//     pub ship: Hull,
//     pub asteroid_lg: Hull,
//     pub asteroid_m: Hull,
//     pub asteroid_sm: Hull,
// }
//
pub fn collision_hull_builder(
    mut commands: Commands,
    assets: Res<Assets<Image>>,
    handles: Res<GameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let ship = assets.get(&handles.ship).unwrap();
    let asteroid_sm = assets.get(&handles.asteroid_sm).unwrap();
    let asteroid_m = assets.get(&handles.asteroid_m).unwrap();
    let asteroid_lg = assets.get(&handles.asteroid_lg).unwrap();

    let ship_visible_pixels = extract_visible_pixels(ship);
    let asteroid_sm_pixels = extract_visible_pixels(asteroid_sm);
    let asteroid_m_pixels = extract_visible_pixels(asteroid_m);
    let asteroid_lg_pixels = extract_visible_pixels(asteroid_lg);

    let hulls = CollisionHulls {
        ship: convex_hull(&ship_visible_pixels),
        asteroid_sm: convex_hull(&asteroid_sm_pixels),
        asteroid_m: convex_hull(&asteroid_m_pixels),
        asteroid_lg: convex_hull(&asteroid_lg_pixels),
    };

    commands.insert_resource(hulls);
    next_state.set(GameState::Ready);
}

pub fn extract_visible_pixels(image: &Image) -> Vec<Vec2> {
    let mut visible_points = Vec::new();

    let pixel_data = &image.data;
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;

    if image.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb {
        // vec of u8, each pixel is made up of [RGBA], each value is a u8
        // ex width is 4
        //    height is 4
        //  first pixel would be  (y(0) * image_width(4) + x(0) * 4(size of pixel data)) = 0
        //  5th pixel would be y = 1 * 4 = 4 + 0 = 4 * 4 = 16
        //  alpha of that is the last component from the calculated position so its index + 3
        //
        //          0    1    2    3 --> x
        // so its 0 RGBA,RGBA,RGBA,RGBA
        // so its 1 RGBA,RGBA,RGBA,RGBA
        // so its 2 RGBA,RGBA,RGBA,RGBA
        // so its 3 RGBA,RGBA,RGBA,RGBA
        //        y
        // row 1(second row) pixel 15(0 index)
        // y = 1
        // x = 15
        for y in 0..height {
            for x in 0..width {
                let pixel_index = (y * width + x) * 4;
                let pixel_alpha = pixel_data[pixel_index + 3];

                // realign the pixels around the center of the image
                let x_f = (x as f32) - (width as f32) / 2.;
                let y_f = (y as f32) - (height as f32) / 2.;

                if pixel_alpha > 0 {
                    visible_points.push(Vec2::new(x_f as f32, y_f as f32))
                }
            }
        }
    }

    visible_points
}

#[derive(Eq, PartialEq)]
enum Orientation {
    Collinear,
    Clockwise,
    CounterClockwise,
}

pub fn orientation(a: Vec2, b: Vec2, c: Vec2) -> Orientation {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);

    if cross == 0. {
        Orientation::Collinear
    } else if cross > 0. {
        Orientation::CounterClockwise
    } else {
        Orientation::Clockwise
    }
}

pub fn convex_hull(pixel_data: &Vec<Vec2>) -> Hull {
    // scan for the minimum, leftmost coord, i don't know if this is the best but for now we will
    // consider 0,0 to be the smallest possible coord
    //
    let mut origin: Vec2 = Vec2::MAX;

    // let origin_index: usize = 0;

    // scan and find the minimum origin
    for pt in pixel_data {
        if pt.y < origin.y || (pt.y == origin.y && pt.x < origin.x) {
            origin = *pt;
        }
    }

    let mut sorted_pixel_data: Vec<Vec2> = pixel_data
        .iter()
        .filter(|pt| pt.x != origin.x && pt.y != origin.y)
        .cloned()
        .collect();

    sorted_pixel_data.sort_by(|a, b| {
        let angle_a = angle_from_vec(a.x - origin.x, a.y - origin.y);
        let angle_b = angle_from_vec(b.x - origin.x, b.y - origin.y);

        angle_a.partial_cmp(&angle_b).unwrap()
    });

    let mut hull: Vec<Vec2> = vec![origin];

    for pt in sorted_pixel_data {
        while hull.len() > 1
            && orientation(hull[hull.len() - 2], hull[hull.len() - 1], pt) == Orientation::Clockwise
        {
            hull.pop();
        }
        hull.push(pt);
    }

    hull.push(*hull.first().clone().unwrap());

    hull
}

fn angle_from_vec(x: f32, y: f32) -> f32 {
    y.atan2(x)
}
