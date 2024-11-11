use bevy::prelude::*;

use crate::{GameAssets, Lives};

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
                h2_style(&font),
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

pub fn setup_gameover_screen(
    mut commands: Commands,
    lives_query: Query<&Lives>,
    assets: Res<GameAssets>,
) {
    let font = &assets.font;
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
            parent.spawn(TextBundle::from_section("Press [Enter]", h3_style(&font)));
        });
}

pub fn despawn_gameover_screen(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
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
