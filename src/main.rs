use bevy::{asset::LoadState, prelude::*};
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
    Processing,
    Ready,
    InGame,
    GameOver,
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
        .add_event::<PlayerKilledEvent>()
        .add_systems(Startup, (setup, load_assets).chain())
        // Always run the despawner
        .add_systems(Update, despawner)
        //
        // Loading State
        //
        .add_systems(
            Update,
            assets_loaded_listener.run_if(in_state(GameState::Loading)),
        )
        //
        // Processing State
        //
        .add_systems(OnEnter(GameState::Processing), collision_hull_builder)
        //
        // Ready State
        //
        .add_systems(Update, move_to_ingame.run_if(in_state(GameState::Ready)))
        //
        // InGame
        //
        // Spawn in game entities before entering the InGame state
        .add_systems(
            OnEnter(GameState::InGame),
            (setup_player, setup_asteroids).chain(),
        )
        .add_systems(
            Update,
            (player_controls, projectile_spawner, draw_line_gizmo)
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
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
                player_killed_listener,      // player
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_ingame)
        // GameOver State
        .add_systems(OnExit(GameState::GameOver), cleanup_gameover)
        .run();
}

#[derive(Component)]
struct Despawning;

type Hull = Vec<Vec2>;

#[derive(Resource, Default, Clone)]
pub struct CollisionHulls {
    pub ship: Hull,
    pub asteroid_lg: Hull,
    pub asteroid_m: Hull,
    pub asteroid_sm: Hull,
}

#[derive(Resource, Default, Clone)]
pub struct GameAssets {
    pub ship: Handle<Image>,
    pub asteroid_lg: Handle<Image>,
    pub asteroid_m: Handle<Image>,
    pub asteroid_sm: Handle<Image>,
}

#[derive(Resource)]
struct AssetsLoading(Vec<UntypedHandle>);

fn draw_line_gizmo(
    mut gizmos: Gizmos,
    query: Query<&Transform, With<Player>>,
    hulls: Res<CollisionHulls>,
) {
    if let Ok(query) = query.get_single() {
        gizmos.line_2d(
            Vec2::ZERO,
            // Vec2::new(MAX_X_POSITION, MAX_Y_POSITION),
            query.translation.truncate(),
            Color::srgb(0., 1., 1.),
        );
    }

    // draw_lines_from_pts(&mut gizmos, &hulls.ship);
    draw_lines_from_pts(&mut gizmos, &hulls.asteroid_lg, Color::srgb(1.0, 0.0, 0.0));
    draw_lines_from_pts(&mut gizmos, &hulls.asteroid_m, Color::srgb(0.0, 1.0, 0.0));
    draw_lines_from_pts(&mut gizmos, &hulls.asteroid_sm, Color::srgb(0.0, 0.0, 1.0));
    draw_lines_from_pts(&mut gizmos, &hulls.ship, Color::srgb(1.0, 0.0, 1.0));
}

fn draw_lines_from_pts(gizmos: &mut Gizmos, pts: &Vec<Vec2>, color: Color) {
    for line in pts.windows(2) {
        let start = line[0];
        let end = line[1];
        gizmos.line_2d(start, end, color);
    }
}

fn load_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let game_assets = GameAssets {
        ship: server.load("ship.png"),
        asteroid_lg: server.load("asteroid-lg.png"),
        asteroid_m: server.load("asteroid-m.png"),
        asteroid_sm: server.load("asteroid-sm.png"),
    };

    loading.0.push(game_assets.ship.clone().untyped());
    loading.0.push(game_assets.asteroid_lg.clone().untyped());
    loading.0.push(game_assets.asteroid_m.clone().untyped());
    loading.0.push(game_assets.asteroid_sm.clone().untyped());

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

    commands.spawn(Lives(5));
    commands.insert_resource(AssetsLoading(Vec::new()));
}

fn move_to_ingame(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyP) {
        println!("Pressed P");
        next_state.set(GameState::InGame);
    }
}

fn cleanup_ingame(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Player>, With<Projectile>, With<Asteroid>)>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for entity in query.iter() {
        commands.entity(entity).try_insert(Despawning);
    }

    next_state.set(GameState::Ready);
}

fn cleanup_gameover(mut commands: Commands, mut query: Query<&mut Lives>) {
    let maybe_lives = query.get_single_mut();

    if let Ok(mut lives) = maybe_lives {
        lives.0 = 5;
    } else {
        commands.spawn(Lives(5));
    }
}

fn despawner(mut commands: Commands, query: Query<Entity, With<Despawning>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn assets_loaded_listener(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    server: Res<AssetServer>,
    handles: Res<AssetsLoading>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ev in ev_asset.read() {
        let AssetEvent::LoadedWithDependencies { id: _ } = ev else {
            continue;
        };

        let all_loaded = handles
            .0
            .iter()
            .all(|h| server.load_state(h) == LoadState::Loaded);

        if all_loaded {
            next_state.set(GameState::Processing);
        }
    }
}
