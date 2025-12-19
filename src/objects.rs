//! This module contains all the "things" in the game.
//!
//! Asteroids, the player's ship, and such.

use bevy::{
    audio::{AudioPlayer, PlaybackSettings},
    camera::visibility::Visibility,
    ecs::{
        component::Component,
        entity::Entity,
        message::{MessageReader, MessageWriter},
        query::With,
        system::{Commands, Query, Res, ResMut, Single},
    },
    math::{Vec2, Vec3, Vec3Swizzles},
    mesh::Mesh2d,
    prelude::{Deref, DerefMut},
    sprite_render::MeshMaterial2d,
    state::state::NextState,
    time::{Timer, TimerMode},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};

use crate::{
    AngularVelocity, GameAssets, GameState, Lives,
    config::{ASTEROID_LIFETIME, DEBRIS_LIFETIME, SHIP_FIRE_RATE},
    machinery::{Lifetime, Sparkler},
    messages::{AsteroidDestroy, BulletDestroy, ShipDestroy, SpawnAsteroid},
    physics::{Velocity, Wrapping},
};

/// The asteroid, defined entirely by [it's size](`AsteroidSize`).
#[derive(Component, Deref, DerefMut)]
pub struct Asteroid(pub AsteroidSize);

#[derive(Clone, Copy, Debug)]
pub enum AsteroidSize {
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    /// Convenience util to get the "next smallest" size. Useful for splitting
    /// after collision.
    pub fn next(&self) -> Option<Self> {
        match self {
            AsteroidSize::Small => None,
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Medium),
        }
    }
}

/// Marker component for the player's ship.
#[derive(Component)]
pub struct Ship;

/// The ship's gun (is just a timer)
#[derive(Component, Deref, DerefMut)]
pub struct Weapon(Timer);

/// Marker component for bullets.
#[derive(Component)]
pub struct Bullet;

/// Debris left behind after the ship is destroyed.
#[derive(Component)]
pub struct Debris;

/// Responds to [`SpawnAsteroid`] events, spawning as specified
pub fn spawn_asteroid(
    mut events: MessageReader<SpawnAsteroid>,
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
            AsteroidSize::Small => 5.0,
            AsteroidSize::Medium => 10.0,
            AsteroidSize::Large => 20.0,
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
    mut destroy_events: MessageReader<AsteroidDestroy>,
    mut respawn_events: MessageWriter<SpawnAsteroid>,
    mut commands: Commands,
    query: Query<(&Transform, &Asteroid, &Velocity)>,
    game_assets: Res<GameAssets>,
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

            // Play a sound for the asteroid exploding
            commands.spawn((
                AudioPlayer::new(game_assets.asteroid_crack_sound()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

/// Spawns the player at the world origin. Used during the state change to
/// [`GameState::Playing`] to spawn the player.
///
/// This only spawns the player. For player **re**-spawn activity, see the
/// [`ship_impact_listener()`] system.
pub fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
            Collider::ball(0.7),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::STATIC_STATIC,
            Ship,
            Weapon(Timer::from_seconds(1.0 / SHIP_FIRE_RATE, TimerMode::Once)),
            Wrapping,
            Velocity(Vec2::ZERO),
            AngularVelocity(0.0),
            Mesh2d(game_assets.ship().0),
            MeshMaterial2d(game_assets.ship().1),
            Transform::default().with_scale(Vec3::new(20.0, 20.0, 20.0)),
            AudioPlayer::new(game_assets.ship_thruster_sound()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: true,
                ..Default::default()
            },
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
pub fn bullet_impact_listener(mut commands: Commands, mut events: MessageReader<BulletDestroy>) {
    for event in events.read() {
        commands.entity(event.0).despawn();
    }
}

/// Watch for [`ShipDestroy`] events and update game state accordingly.
///
/// One life is taken from the counter, asteroids are cleared, and the player
/// is placed back at the origin. If lives reach 0, this system will change
/// states to [`GameState::GameOver`].
/// - Subtract a life
/// - Check life count. If 0, go to game-over state
/// - Clear all asteroids
/// - Respawn player
pub fn ship_impact_listener(
    mut events: MessageReader<ShipDestroy>,
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    rocks: Query<Entity, With<Asteroid>>,
    mut player: Single<(&mut Transform, &mut Velocity), With<Ship>>,
    mut next_state: ResMut<NextState<GameState>>,
    game_assets: Res<GameAssets>,
) {
    for _ in events.read() {
        // STEP 1: Clear asteroids
        for rock in rocks {
            commands.entity(rock).despawn();
        }

        // STEP 2: Decrement lives
        if lives.0 == 0 {
            // If the player has run out, return early with a state change.
            next_state.set(GameState::GameOver);
            return;
        } else {
            // Decrease life count.
            lives.0 -= 1;
        }

        // STEP 3: spawn the debris field where the player used to be.
        for i in 0..10 {
            let angle_rads = (i as f32) / 10.0 * std::f32::consts::TAU;
            let dir = Vec2::from_angle(angle_rads);
            let vel = player.1.0 + dir * 100.0;
            commands.spawn((
                Debris,
                Visibility::Visible, // make sure it's "visible" not "Inherited" so the cycle works right
                Lifetime(Timer::from_seconds(DEBRIS_LIFETIME, TimerMode::Once)),
                Sparkler::at_interval(0.15),
                Mesh2d(game_assets.thruster_mesh()), // borrow the thruster mesh for now
                MeshMaterial2d(game_assets.thruster_mat_active()), // ... and the active thruster material
                player.0.clone(),                                  // clone the player transform
                Velocity(vel),
            ));
        }

        // STEP 4: Respawn player (teleport them to the origin)
        player.0.translation = Vec3::ZERO;
        player.1.0 = Vec2::ZERO;

        // STEP 5: Play crash sound
        commands.spawn((
            AudioPlayer::new(game_assets.wreck_sound()),
            PlaybackSettings::DESPAWN, // despawn this entity when playback ends.
        ));
    }
}
