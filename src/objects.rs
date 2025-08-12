//! This module contains all the "things" in the game.
//!
//! Asteroids, the player's ship, and such.

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res, ResMut, Single},
    },
    math::{Vec2, Vec3, Vec3Swizzles},
    prelude::{Deref, DerefMut},
    render::mesh::Mesh2d,
    sprite::MeshMaterial2d,
    state::state::NextState,
    time::{Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

use crate::{
    AngularVelocity, GameAssets, GameState, Lives,
    config::ASTEROID_LIFETIME,
    events::{AsteroidDestroy, BulletDestroy, ShipDestroy, SpawnAsteroid},
    machinery::Lifetime,
    physics::{Velocity, Wrapping},
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

pub fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
            Collider::ball(0.7),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::STATIC_STATIC,
            Ship,
            Wrapping,
            Velocity(Vec2::ZERO),
            AngularVelocity(0.0),
            Mesh2d(game_assets.ship().0),
            MeshMaterial2d(game_assets.ship().1),
            Transform::default().with_scale(Vec3::new(20.0, 20.0, 20.0)),
        ))
        .with_child((
            Mesh2d(game_assets.thruster_mesh()),
            MeshMaterial2d(game_assets.thruster_mat_inactive()),
            Transform::default()
                .with_scale(Vec3::splat(0.5))
                .with_translation(Vec3::new(-0.5, 0.0, -0.1)),
        ));
}

/// Watch for [`BulletDestroy`] events and despawn
/// the associated bullet.
pub fn bullet_impact_listener(mut commands: Commands, mut events: EventReader<BulletDestroy>) {
    for event in events.read() {
        commands.entity(event.0).despawn();
    }
}

/// Watch for [`ShipDestroy`] events and update game state accordingly.
///
/// - Subtract a life
/// - Check life count. If 0, go to game-over state
/// - Clear all asteroids
/// - Respawn player
pub fn ship_impact_listener(
    mut events: EventReader<ShipDestroy>,
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    rocks: Query<Entity, With<Asteroid>>,
    mut player: Single<(&mut Transform, &mut Velocity), With<Ship>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in events.read() {
        // STEP 1: Decrement lives (and maybe go to game over)
        if lives.0 == 0 {
            // If already at 0, game is over.
            next_state.set(GameState::GameOver);
        } else {
            // Decrease life count.
            lives.0 -= 1;
        }

        // STEP 2: Clear asteroids
        for rock in rocks {
            commands.entity(rock).despawn();
        }

        // STEP 3: Respawn player (teleport them to the origin)
        player.0.translation = Vec3::ZERO;
        player.1.0 = Vec2::ZERO;
    }
}
