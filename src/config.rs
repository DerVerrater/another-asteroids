//! Global constants used all over the program. Rather than leaving them scattered
//! where ever they happen to be needed, I'm concentrating them here.

use bevy::color::Color;

pub const WINDOW_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(800.0, 600.0);

pub const UI_BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15); // Button color when it's just hanging out
pub const UI_BUTTON_HOVERED: Color = Color::srgb(0.25, 0.25, 0.25); // ... when it's hovered
pub const UI_BUTTON_PRESSED: Color = Color::srgb(0.55, 0.55, 0.55); // ... when it's pressed

pub(crate) const BACKGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
pub(crate) const PLAYER_SHIP_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
pub(crate) const SHIP_THRUSTER_COLOR_ACTIVE: Color = Color::srgb(1.0, 0.2, 0.2);
pub(crate) const SHIP_THRUSTER_COLOR_INACTIVE: Color = Color::srgb(0.5, 0.5, 0.5);
pub(crate) const SHIP_FIRE_RATE: f32 = 3.0; // in bullets-per-second
pub(crate) const ASTEROID_SMALL_COLOR: Color = Color::srgb(1.0, 0., 0.);
pub(crate) const BULLET_COLOR: Color = Color::srgb(0.0, 0.1, 0.9);
// TODO: asteroid medium & large

pub(crate) const SHIP_THRUST: f32 = 1.0;
pub(crate) const SHIP_ROTATION: f32 = 4.0; // +/- rotation speed in... radians per frame

pub(crate) const BULLET_SPEED: f32 = 150.0;
pub(crate) const BULLET_LIFETIME: f32 = 2.0;

pub(crate) const ASTEROID_LIFETIME: f32 = 40.0;

pub const RNG_SEED: [u8; 32] = *b"12345678909876543210123456789098";
