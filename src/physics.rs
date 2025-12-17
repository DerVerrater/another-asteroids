//! Custom physics items
//!
//! TODO: Refactor in terms of Rapier2D, *or* implement colliders and remove it.
//!
//! Position, both translation and rotation, are tracked by the built-in
//! Bevy [`Transform`] component. This module adds a (Linear) [`Velocity`] and
//! [`AngularVelocity`] component to the mix.
//! These two components are operated by simple integrator systems to
//! accumulate translation and rotation from the velocities.
//!
//! The [`Wrapping`] component marks things that should wrap around the world
//! boundaries. It's a kind of physical property of this world, so I'm putting
//! it here.
//!
//! Collisions are also considered physics. After all, I got *a physics engine*
//! to detect them for me (so that I don't have to write clipping code).

use crate::{
    WorldSize, messages,
    objects::{Asteroid, Bullet, Ship},
};

use bevy::prelude::*;
use bevy_rapier2d::pipeline::CollisionEvent;

#[derive(Clone, Component)]
pub(crate) struct Velocity(pub(crate) bevy::math::Vec2);

#[derive(Component)]
pub(crate) struct AngularVelocity(pub(crate) f32);

/// Marker for any entity that should wrap on screen edges
#[derive(Component)]
pub(crate) struct Wrapping;

/// Integrate linear velocity and update the entity's transform.
pub(crate) fn integrate_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        let delta = velocity.0 * time.delta_secs();
        transform.translation += delta.extend(0.0);
    }
}

/// Integrate angular velocity and update the entity's transform.
pub(crate) fn integrate_angular_velocity(
    mut objects: Query<(&mut Transform, &AngularVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, ang_vel) in &mut objects {
        let delta = ang_vel.0 * time.delta_secs();
        transform.rotate_z(delta);
    }
}

pub(crate) fn wrap_entities(
    mut query: Query<&mut Transform, With<Wrapping>>,
    world_size: Res<WorldSize>,
) {
    let right = world_size.x / 2.0;
    let left = -right;
    let top = world_size.y / 2.0;
    let bottom = -top;

    for mut pos in query.iter_mut() {
        if pos.translation.x > right {
            pos.translation.x = left;
        } else if pos.translation.x < left {
            pos.translation.x = right;
        }

        if pos.translation.y > top {
            pos.translation.y = bottom;
        } else if pos.translation.y < bottom {
            pos.translation.y = top;
        }
    }
}

/// The collision event routing system.
///
/// When a `CollisionEvent` occurrs, this system checks which things collided
/// and emits secondary events accordignly.
///
/// | Objects | Response |
/// |-|-|
/// | Ship & Asteroid | emits event [`ShipDestroy`](`crate::events::ShipDestroy`) |
/// | Asteroid & Bullet | emits event [`AsteroidDestroy`](`crate::events::AsteroidDestroy`) |
/// | Asteroid & Asteroid | Nothing. Asteroids won't collide with each other |
/// | Bullet & Bullet | Nothing. Bullets won't collide with each other (and probably can't under normal gameplay conditions) |
/// | Bullet & Ship | Nothing. The player shouldn't be able to shoot themselves (and the Flying Saucer hasn't been impl.'d, so it's bullets don't count) |
pub fn collision_listener(
    mut collisions: MessageReader<CollisionEvent>,
    mut ship_writer: MessageWriter<messages::ShipDestroy>,
    mut asteroid_writer: MessageWriter<messages::AsteroidDestroy>,
    mut bullet_writer: MessageWriter<messages::BulletDestroy>,
    player: Single<Entity, With<Ship>>,
    bullets: Query<&Bullet>,
    rocks: Query<&Asteroid>,
) {
    for event in collisions.read() {
        if let CollisionEvent::Started(one, two, _flags) = event {
            // Valid collisions are:
            //
            // - Ship & Asteroid
            // - Bullet & Asteroid
            //
            // Asteroids don't collide with each other, bullets don't collide
            // with each other, and bullets don't collide with the player ship.

            // Option 1: Ship & Asteroid
            if *one == *player {
                if rocks.contains(*two) {
                    // player-asteroid collision
                    dbg!("Writing ShipDestroy event");
                    ship_writer.write(messages::ShipDestroy);
                } // else, we don't care
            } else if *two == *player {
                if rocks.contains(*one) {
                    dbg!("Writing ShipDestroy event");
                    ship_writer.write(messages::ShipDestroy);
                }
            }

            // Option 2: Bullet & Asteroid
            if bullets.contains(*one) {
                if rocks.contains(*two) {
                    dbg!("Writing AsteroidDestroy & BulletDestroy events");
                    asteroid_writer.write(messages::AsteroidDestroy(*two));
                    bullet_writer.write(messages::BulletDestroy(*one));
                }
            } else if rocks.contains(*one) {
                if bullets.contains(*two) {
                    dbg!("Writing AsteroidDestroy & BulletDestroy events");
                    asteroid_writer.write(messages::AsteroidDestroy(*one));
                    bullet_writer.write(messages::BulletDestroy(*two));
                }
            }
        }
    }
}
