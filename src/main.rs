use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, player_movement)
        .run();
}

const SHIP_COLOR: Color = Color::srgb(0.9, 0.0, 0.0);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

const SHIP_SIZE: Vec2 = Vec2::new(20., 40.);
const PLAYER_SPEED: f32 = 300.0;
const PLAYER_ROTATION_SPEED: f32 = 360.0;

impl PlayerBundle {
    fn new() -> PlayerBundle {
        PlayerBundle {
            player: Player,
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

    commands.spawn(PlayerBundle::new());
}

fn player_movement(
    mut query: Query<&mut Transform, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();
    let mut movement_factor = 0.0;
    let mut rotation_factor = 0.0;

    if keys.pressed(KeyCode::KeyW) {
        movement_factor += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement_factor -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        rotation_factor -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        rotation_factor += 1.0;
    }

    transform
        .rotate_z(rotation_factor * f32::to_radians(PLAYER_ROTATION_SPEED) * time.delta_seconds());

    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = movement_factor * PLAYER_SPEED * time.delta_seconds();
    let translation_delta = movement_direction * movement_distance;

    transform.translation += translation_delta;
}
