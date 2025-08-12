//! This is the module containing all the rock-related things.
//! Not... not the whole game.

use bevy_rapier2d::prelude::*;
use rand::{Rng, SeedableRng};
use std::time::Duration;

use bevy::prelude::*;

use crate::{
    config::ASTEROID_LIFETIME, events::{AsteroidDestroy, SpawnAsteroid}, objects::{Asteroid, AsteroidSize}, physics::Velocity, GameAssets, Lifetime, WorldSize
};

#[derive(Resource)]
pub struct AsteroidSpawner {
    rng: std::sync::Mutex<rand::rngs::StdRng>,
    timer: Timer,
    // TODO: Configurables?
    // - interval
    // - density
    // - size distribution
}

impl AsteroidSpawner {
    pub fn new() -> Self {
        Self {
            rng: std::sync::Mutex::new(rand::rngs::StdRng::from_seed(crate::config::RNG_SEED)),
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
        }
    }
}

/// Update the asteroid spawn timer and spawn any asteroids
/// that are due this frame.
pub fn tick_asteroid_manager(
    mut events: EventWriter<SpawnAsteroid>,
    mut spawner: ResMut<AsteroidSpawner>,
    time: Res<Time>,
    play_area: Res<WorldSize>,
) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        let mut rng = spawner
            .rng
            .lock()
            .expect("Expected to acquire lock on the AsteroidSpawner's RNG field.");

        // Use polar coordinate to decide where the asteroid will spawn
        // Theta will be random between 0 to 2pi
        let spawn_angle = rng.random_range(0.0..(std::f32::consts::PI * 2.0));
        // Rho will be the radius of a circle bordering the viewport, multiplied by 1.2
        // TODO: Use view diagonal to get a minimally sized circle around the play area
        let spawn_distance = play_area.width.max(play_area.height) / 2.0;

        // Convert polar to Cartesian, use as position
        let pos = Vec2::new(
            spawn_distance * spawn_angle.cos(),
            spawn_distance * spawn_angle.sin(),
        );

        // Right now, I'm thinking I can use the opposite signs attached to the position Vec components.
        // pos.x == -100, then vel.x = + <random>
        // pos.x == 100, then vel.x = - <random>
        // etc,
        let mut vel = Vec2::new(rng.random_range(0.0..100.0), rng.random_range(0.0..100.0));
        if pos.x > 0.0 {
            vel.x *= -1.0;
        }
        if pos.y > 0.0 {
            vel.y *= -1.0;
        }

        let size = match rng.random_range(0..=2) {
            0 => AsteroidSize::Small,
            1 => AsteroidSize::Medium,
            2 => AsteroidSize::Large,
            _ => unreachable!(),
        };

        events.write(SpawnAsteroid { pos, vel, size });
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
        let (mesh, material) = match spawn.size {
            AsteroidSize::Small => game_assets.asteroid_small(),
            AsteroidSize::Medium => game_assets.asteroid_medium(),
            AsteroidSize::Large => game_assets.asteroid_large(),
        };

        let collider_radius = match spawn.size {
            AsteroidSize::Small => 10.0,
            AsteroidSize::Medium => 20.0,
            AsteroidSize::Large => 40.0,
        };

        commands.spawn((
            Asteroid(spawn.size),
            Collider::ball(collider_radius),
            Sensor,
            Transform::from_translation(spawn.pos.extend(0.0)),
            Velocity(spawn.vel),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Lifetime(Timer::from_seconds(ASTEROID_LIFETIME, TimerMode::Once)),
        ));
    }
}

/// Event listener for asteroid destruction events. Shrinks and multiplies
/// asteroids until they vanish.
///
/// - Large -> 2x Medium
/// - Medium -> 2x Small
/// - Small -> (despawned)
///
/// The velocity of the child asteroids is scattered somewhat, as if they were
/// explosively pushed apart.
pub fn split_asteroids(
    mut destroy_events: EventReader<AsteroidDestroy>,
    mut respawn_events: EventWriter<SpawnAsteroid>,
    mut commands: Commands,
    query: Query<(&Transform, &Asteroid, &Velocity)>,
) {
    for event in destroy_events.read() {
        if let Ok((transform, rock, velocity)) = query.get(event.0) {
            let next_size = rock.0.next();
            if let Some(size) = next_size {
                let pos = transform.translation.xy();
                let left_offset = Vec2::from_angle(0.4);
                let right_offset = Vec2::from_angle(-0.4);
                respawn_events.write(SpawnAsteroid {
                    pos,
                    vel: left_offset.rotate(velocity.0),
                    size,
                });
                respawn_events.write(SpawnAsteroid {
                    pos,
                    vel: right_offset.rotate(velocity.0),
                    size,
                });
            }
            // Always despawn the asteroid. New ones (may) be spawned in it's
            // place, but this one is gone.
            commands.entity(event.0).despawn();
        }
    }
}
