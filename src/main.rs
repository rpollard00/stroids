use bevy::{prelude::*, utils::warn};
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;

mod constants;
use constants::*;

mod physics;
use physics::*;

mod player;
use player::*;

mod asteroid;
use asteroid::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Loading,
    Ready,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stroids".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_event::<ProjectileFiredEvent>()
        .add_event::<AsteroidDestroyedEvent>()
        // the player part of the setup, and setup_asteroids should be run when we enter the InGame
        // state
        .add_systems(Startup, (setup, load_assets).chain())
        .add_systems(Update, move_to_ingame.run_if(in_state(GameState::Loading)))
        .add_systems(
            OnEnter(GameState::InGame),
            (setup_player, setup_asteroids).chain(),
        ) //game/ asteroids
        // player controls, projectile spawner should run in the InGame state
        .add_systems(
            Update,
            (player_controls, projectile_spawner)
                .chain()
                .run_if(in_state(GameState::InGame)),
        ) // player
        // all these systems should be in the InGame state, only
        .add_systems(
            FixedUpdate,
            (
                apply_movement,              // physics
                apply_rotational_velocity,   // physics
                distance_tracker,            // asteroid
                speed_limit_system,          // player
                out_of_bounds_system,        // physics
                collision_system,            // physics
                asteroid_destroyed_listener, // asteroids
                despawner,                   // game
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .run();
}

#[derive(Component)]
struct Despawning;

#[derive(Resource, Default, Clone)]
pub struct GameAssets {
    pub ship: Handle<Image>,
    pub asteroid_lg: Handle<Image>,
    pub asteroid_m: Handle<Image>,
    pub asteroid_sm: Handle<Image>,
}

fn draw_line_gizmo(mut gizmos: Gizmos, query: Query<&Transform, With<Player>>) {
    if let Ok(query) = query.get_single() {
        gizmos.line_2d(
            Vec2::ZERO,
            // Vec2::new(MAX_X_POSITION, MAX_Y_POSITION),
            query.translation.truncate(),
            Color::srgb(0., 1., 1.),
        );
    }
}

fn load_assets(mut commands: Commands, server: Res<AssetServer>) {
    let game_assets = GameAssets {
        ship: server.load("ship.png"),
        asteroid_lg: server.load("asteroid-lg.png"),
        asteroid_m: server.load("asteroid-m.png"),
        asteroid_sm: server.load("asteroid-sm.png"),
    };
    commands.insert_resource(game_assets);
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
}

fn move_to_ingame(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyP) {
        println!("Pressed P");
        next_state.set(GameState::InGame);
    }
}

fn screen_edge_distance(direction_norm: &Vec2) -> f32 {
    assert!(direction_norm.is_normalized());
    let abs_dir = direction_norm.abs();

    let x_edge_dist = MAX_X_POSITION / abs_dir.x;
    let y_edge_dist = MAX_Y_POSITION / abs_dir.y;

    x_edge_dist.min(y_edge_dist)
}

fn despawner(mut commands: Commands, query: Query<Entity, With<Despawning>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
