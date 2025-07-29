use std::time::Duration;

/// This is the module containing all the rock-related things
/// not... not the whole game.
use bevy::prelude::*;

use crate::{GameAssets, Position, Rotation, Velocity};

#[derive(Component, Deref, DerefMut)]
pub struct Asteroid(AsteroidSize);

pub enum AsteroidSize {
    SMALL,
    MEDIUM,
    LARGE,
}

#[derive(Resource)]
pub struct AsteroidSpawner {
    timer: Timer,
    // TODO: Configurables?
    // - interval
    // - density
    // - size distribution
}

impl AsteroidSpawner {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
        }
    }
}

/// Update the asteroid spawn timer and spawn any asteroids
/// that are due this frame.
pub fn tick_asteroid_manager(
    mut commands: Commands,
    mut spawner: ResMut<AsteroidSpawner>,
    game_assets: Res<GameAssets>,
    time: Res<Time>,
) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        commands.spawn((
            Asteroid(AsteroidSize::SMALL),
            Position(Vec2::new(40.0, 40.0)),
            Velocity(Vec2::new(10.0, 0.0)),
            Rotation(0.0),
            Mesh2d(game_assets.asteroid_small().0),
            MeshMaterial2d(game_assets.asteroid_small().1),
        ));
    }
}

/// Utility function to spawn a single asteroid of a given type
/// TODO: convert to an event listener monitoring for "spawn asteroid" events
/// from the `fn tick_asteroid_manager(...)` system.
fn spawn_asteroid(mut commands: Commands) {
    todo!();
}
