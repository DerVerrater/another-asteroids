use std::time::Duration;

/// This is the module containing all the rock-related things
/// not... not the whole game.
use bevy::{math::VectorSpace, prelude::*};

use crate::{GameAssets, Position, Rotation, Velocity};

#[derive(Component, Deref, DerefMut)]
pub struct Asteroid(AsteroidSize);

pub enum AsteroidSize {
    Small,
    Medium,
    Large,
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

#[derive(Event)]
pub struct SpawnAsteroid {
    pos: Vec2,
    vel: Vec2,
    size: AsteroidSize,
}

/// Update the asteroid spawn timer and spawn any asteroids
/// that are due this frame.
pub fn tick_asteroid_manager(
    mut events: EventWriter<SpawnAsteroid>,
    mut spawner: ResMut<AsteroidSpawner>,
    time: Res<Time>,
) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        events.write(SpawnAsteroid {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            size: AsteroidSize::Small,
        });
    }
}

/// Utility function to spawn a single asteroid of a given type
/// TODO: convert to an event listener monitoring for "spawn asteroid" events
/// from the `fn tick_asteroid_manager(...)` system.
pub fn spawn_asteroid(
    mut events: EventReader<SpawnAsteroid>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for spawn in events.read() {
        // TODO: Use the asteroid properties defined in the event to spawn the entity
        commands.spawn((
            Asteroid(AsteroidSize::Small),
            Position(Vec2::new(40.0, 40.0)),
            Velocity(Vec2::new(10.0, 0.0)),
            Rotation(0.0),
            Mesh2d(game_assets.asteroid_small().0),
            MeshMaterial2d(game_assets.asteroid_small().1),
        ));
    }
}
