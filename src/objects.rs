//! This module contains all the "things" in the game.
//!
//! Asteroids, the player's ship, and such.

use bevy::{
    ecs::{
        component::Component,
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res},
    },
    math::{Vec2, Vec3Swizzles},
    prelude::{Deref, DerefMut},
    render::mesh::Mesh2d,
    sprite::MeshMaterial2d,
    time::{Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{Collider, Sensor};

use crate::{
    GameAssets, Lifetime,
    config::ASTEROID_LIFETIME,
    events::{AsteroidDestroy, SpawnAsteroid},
    physics::Velocity,
};

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

/// Responds to [`SpawnAsteroid`] events, spawning as specified
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
