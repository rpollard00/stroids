use bevy::prelude::*;

use crate::{GameAssets, Lives};

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

#[derive(Component)]
pub struct TitleScreen;

pub fn setup_title_screen(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(TitleScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Stroids",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 42.,
                    ..default()
                },
            ));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Press P",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 30.,
                    ..default()
                },
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
    let lives_remaining = format!("{} lives remaining", lives);
    commands
        .spawn(NodeBundle {
            style: ui_screen_style(),
            ..default()
        })
        .insert(DiedScreen)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "You Died",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 42.,
                    ..default()
                },
            ));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                lives_remaining,
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 30.,
                    ..default()
                },
            ));
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Press P",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 30.,
                    ..default()
                },
            ));
        });
}

pub fn despawn_died_screen(mut commands: Commands, query: Query<Entity, With<DiedScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
