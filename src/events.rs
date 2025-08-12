use bevy::prelude::*;

/// Signals that the player's ship has been destroyed.
/// Used when the player collides with an asteroid.
#[derive(Event)]
pub(crate) struct ShipDestroy;

/// Signals that a particular asteroid has been destroyed.
/// Used to split (or vanish) an asteroid when a bullet strikes it.
#[derive(Event)]
pub(crate) struct AsteroidDestroy(pub Entity);

// TODO: BulletDestroy
// Which depends on the still-pending Bullet component creation.

/// Signals that a particular bullet has been destroyed.
/// Used to despawn the bullet after it strikes an Asteroid.
///
/// TODO: Maybe use it for lifetime expiration (which is also a TODO item).
#[derive(Event)]
pub(crate) struct BulletDestroy(pub Entity);
