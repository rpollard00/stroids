use bevy::prelude::*;

use crate::{AsteroidDestroyedEvent, LevelUpEvent, Lives};

#[derive(Resource, Clone)]
pub struct Score(pub u64);

pub fn setup_score(mut commands: Commands) {
    let score = Score(0);

    commands.insert_resource(score);
}

pub fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}

const LARGE_ASTEROID_VALUE: u64 = 100;
const MEDIUM_ASTEROID_VALUE: u64 = 50;
const SMALL_ASTEROID_VALUE: u64 = 25;
pub const LIFE_BONUS: u64 = 1000;

pub fn update_score_listener(
    mut asteroid_destroyed_event: EventReader<AsteroidDestroyedEvent>,
    mut level_up_event: EventReader<LevelUpEvent>,
    mut score: ResMut<Score>,
    lives_query: Query<&Lives>,
) {
    for ev in asteroid_destroyed_event.read() {
        let (_, _, _, size) = (ev.0, ev.1, ev.2, ev.3);

        score.0 += match size {
            crate::AsteroidSize::Small => SMALL_ASTEROID_VALUE,
            crate::AsteroidSize::Medium => MEDIUM_ASTEROID_VALUE,
            crate::AsteroidSize::Large => LARGE_ASTEROID_VALUE,
        };
    }

    for ev in level_up_event.read() {
        let level = ev.0 .0;
        let lives = lives_query.single();

        score.0 += lives.0 as u64 * (LIFE_BONUS * level as u64);
    }
}
