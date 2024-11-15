use bevy::prelude::*;

use crate::{GameAssets, Level, Lives, Score, LIFE_BONUS};

#[derive(Component)]
pub struct TitleScreen;

pub fn setup_title_screen(mut commands: Commands, assets: Res<GameAssets>) {
    let font = &assets.font;
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(TitleScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Stroids", h1_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Press [Enter] to Play",
                h3_style(&font),
            ));
        });
}

pub fn despawn_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct DiedScreen;

pub fn setup_died_screen(
    mut commands: Commands,
    lives_query: Query<&Lives>,
    assets: Res<GameAssets>,
) {
    let Lives(lives) = lives_query.single();
    let font = &assets.font;
    let lives_remaining = format!("{} lives remaining", lives);
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(DiedScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("You Died", h1_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(lives_remaining, h2_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Press [Enter]", h3_style(&font)));
        });
}

pub fn despawn_died_screen(mut commands: Commands, query: Query<Entity, With<DiedScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct GameOverScreen;

pub fn setup_gameover_screen(mut commands: Commands, assets: Res<GameAssets>, score: Res<Score>) {
    let font = &assets.font;
    let score_text = format!("Final Score: {}", score.0);
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(GameOverScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Game Over", h1_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(score_text, h2_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Press [Enter]", h3_style(&font)));
        });
}

pub fn despawn_gameover_screen(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct LevelCompleteScreen;

pub fn setup_level_complete_screen(
    mut commands: Commands,
    assets: Res<GameAssets>,
    score: Res<Score>,
    level_query: Query<&Level>,
    lives_query: Query<&Lives>,
) {
    let next_level = level_query.single();
    let lives = lives_query.single();
    let next_level_text = format!("Next Level: {} ", next_level.0);
    let font = &assets.font;
    let score_text = format!(
        "Score: {} (Lives Bonus: {})",
        score.0,
        lives.0 as u64 * (LIFE_BONUS * (next_level.0 - 1) as u64)
    );
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(LevelCompleteScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Level Complete", h1_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(score_text, h2_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(next_level_text, h2_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Press [Enter]", h3_style(&font)));
        });
}

pub fn despawn_level_complete_screen(
    mut commands: Commands,
    query: Query<Entity, With<LevelCompleteScreen>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct InGameUi;

#[derive(Component)]
pub struct ScoreText;

pub fn setup_ingame_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
    lives_query: Query<&Lives>,
    level_query: Query<&Level>,
    score: Res<Score>,
) {
    let lives = lives_query.single();
    let level = level_query.single();
    let font = &assets.font;
    let score_text = format!("Score: {}", score.0);
    let lives_text = format!("Lives: {}", lives.0);
    let level_text = format!("Level: {}", level.0);

    commands
        .spawn(NodeBundle {
            style: ingame_ui_style(),
            ..default()
        })
        .insert(InGameUi)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(lives_text, h3_style(&font)));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("|", h3_style(&font)));
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section(score_text, h3_style(&font)))
                .insert(ScoreText);
        });

    commands
        .spawn(NodeBundle {
            style: right_align_ui_style(),
            ..default()
        })
        .insert(InGameUi)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(level_text, h3_style(&font)));
        });
}

pub fn despawn_ingame_ui(mut commands: Commands, query: Query<Entity, With<InGameUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn update_score_ui(score: Res<Score>, mut score_text_query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        if let Ok(mut text) = score_text_query.get_single_mut() {
            text.sections[0].value = format!("Score: {}", score.0);
        }
    }
}

fn ui_screen_style() -> Style {
    Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(10.),
        ..default()
    }
}

fn ingame_ui_style() -> Style {
    Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Start,
        align_items: AlignItems::Start,
        column_gap: Val::Px(20.),
        ..default()
    }
}

fn right_align_ui_style() -> Style {
    Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::End,
        align_items: AlignItems::Start,
        column_gap: Val::Px(20.),
        ..default()
    }
}

fn h1_style(font: &Handle<Font>) -> TextStyle {
    TextStyle {
        font: font.clone(),
        font_size: 42.,
        ..default()
    }
}

fn h2_style(font: &Handle<Font>) -> TextStyle {
    TextStyle {
        font: font.clone(),
        font_size: 30.,
        ..default()
    }
}

fn h3_style(font: &Handle<Font>) -> TextStyle {
    TextStyle {
        font: font.clone(),
        font_size: 18.,
        ..default()
    }
}
