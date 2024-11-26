/*
 Global constants used all over the program. Rather than leaving them scattered
where ever they happen to be needed, I'm concentrating them here.
*/

use bevy::color::Color;

pub(crate) const BACKGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
pub(crate) const PLAYER_SHIP_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

pub(crate) const SHIP_THRUST: f32 = 1.0;
pub(crate) const SHIP_ROTATION: f32 = 0.1; // +/- rotation speed in... radians per frame
