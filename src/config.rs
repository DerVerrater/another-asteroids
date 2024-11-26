/*
 Global constants used all over the program. Rather than leaving them scattered
where ever they happen to be needed, I'm concentrating them here.
*/

use bevy::color::Color;

pub(crate) const BACKGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
pub(crate) const PLAYER_SHIP_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

pub(crate) const SHIP_THRUST_LIMIT: f32 = 10.0;
pub(crate) const SHIP_ROTATION_LIMIT: f32 = 5.0; // +/- rotation speed in... uunniittss
