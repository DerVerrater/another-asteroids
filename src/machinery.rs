//! These are Systems that power the main game mechanics (and some misc items to support them)

use rand::{Rng, SeedableRng};
use std::time::Duration;

use bevy::prelude::*;

use crate::{WorldSize, events::SpawnAsteroid, objects::AsteroidSize};

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
