//! This module contains all the "things" in the game.
//! 
//! Asteroids, the player's ship, and such.

use bevy::{ecs::component::Component, prelude::{Deref, DerefMut}};

#[derive(Component, Deref, DerefMut)]
pub struct Asteroid(pub AsteroidSize);

#[derive(Clone, Copy, Debug)]
pub enum AsteroidSize {
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    pub fn next(&self) -> Option<Self> {
        match self {
            AsteroidSize::Small => None,
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Medium),
        }
    }
}

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Bullet;
