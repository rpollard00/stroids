use bevy::prelude::*;

use crate::{AsteroidCount, GameState};

#[derive(Component, Clone, Debug)]
pub struct Level(pub u32);

#[derive(Event)]
pub struct LevelUpEvent(pub Level);

pub fn setup_level(mut commands: Commands) {
    commands.spawn(Level(1));
}

pub fn level_completion_watcher(
    asteroid_query: Query<&AsteroidCount, Changed<AsteroidCount>>,
    mut level_query: Query<&mut Level>,
    mut next_state: ResMut<NextState<GameState>>,
    mut event_writer: EventWriter<LevelUpEvent>,
) {
    for asteroid_count in asteroid_query.iter() {
        if asteroid_count.0 <= 0 {
            let mut level = level_query.single_mut();
            event_writer.send(LevelUpEvent(Level(level.0)));
            level.0 += 1;
            next_state.set(GameState::LevelComplete)
        }
        return;
    }
}
