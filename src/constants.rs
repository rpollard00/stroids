use bevy::prelude::*;
pub const WINDOW_WIDTH: f32 = 1024.;
pub const WINDOW_HEIGHT: f32 = 768.;
pub const MIN_X_POSITION: f32 = 0. - (WINDOW_WIDTH / 2.);
pub const MAX_X_POSITION: f32 = WINDOW_WIDTH / 2.;
pub const MIN_Y_POSITION: f32 = 0. - (WINDOW_HEIGHT / 2.);
pub const MAX_Y_POSITION: f32 = WINDOW_HEIGHT / 2.;

pub const MAX_SPEED: f32 = WINDOW_WIDTH / SCREEN_CROSS_TIME;
pub const MAX_SPEED_SQUARED: f32 = MAX_SPEED * MAX_SPEED;
pub const BACKGROUND_COLOR: Color = Color::BLACK;
pub const SCREEN_CROSS_TIME: f32 = 1.5; // time in seconds to cross screen
