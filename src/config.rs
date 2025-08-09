//! Global constants used all over the program. Rather than leaving them scattered
//! where ever they happen to be needed, I'm concentrating them here.

use bevy::color::Color;

pub const WINDOW_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(800.0, 600.0);

pub(crate) const BACKGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
pub(crate) const PLAYER_SHIP_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
pub(crate) const SHIP_THRUSTER_COLOR_ACTIVE: Color = Color::srgb(1.0, 0.2, 0.2);
pub(crate) const SHIP_THRUSTER_COLOR_INACTIVE: Color = Color::srgb(0.5, 0.5, 0.5);
pub(crate) const ASTEROID_SMALL_COLOR: Color = Color::srgb(1.0, 0., 0.);
// TODO: asteroid medium & large

pub(crate) const SHIP_THRUST: f32 = 1.0;
pub(crate) const SHIP_ROTATION: f32 = 4.0; // +/- rotation speed in... radians per frame

pub const RNG_SEED: [u8; 32] = *b"12345678909876543210123456789098";
