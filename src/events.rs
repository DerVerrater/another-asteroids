use bevy::prelude::*;

use crate::objects::AsteroidSize;

/// Signals that the player's ship has been destroyed.
///
/// Produced by the [`fn collision_listener(...)`](`crate::physics::collision_listener`)
/// system when the player collides with an asteroid. They are consumed by [`fn ship_impact_listener(...)`](`crate::objects::ship_impact_listener`).
#[derive(Event)]
pub(crate) struct ShipDestroy;

/// Signals that a particular asteroid has been destroyed.
///
/// Produced by the [`fn collision_listener(...)`](`crate::physics::collision_listener`)
/// system when bullets contact asteroids. It is consumed by [`fn split_asteroid(...)`](`crate::objects::spawn_asteroid`)
/// system, which re-emits [spawn events](`SpawnAsteroid`).
#[derive(Event)]
pub(crate) struct AsteroidDestroy(pub Entity);

/// Signals that an asteroid needs to be spawned and provides the parameters
/// for that action.
///
/// Produced by the [`tick_asteroid_manager(...)`](`crate::machinery::tick_asteroid_manager`)
/// system and consumed by the [`spawn_asteroid(...)`](`crate::objects::spawn_asteroid`) system.
#[derive(Event)]
pub struct SpawnAsteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: AsteroidSize,
}

/// Signals that a particular bullet has been destroyed (after it strikes an Asteroid).
///
/// TODO: Maybe use it for lifetime expiration (which is also a TODO item).
#[derive(Event)]
pub(crate) struct BulletDestroy(pub Entity);
