//! Custom physics items
//! TODO: Refactor in terms of Rapier2D, *or* implement colliders and remove it.

use crate::WorldSize;

use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Velocity(pub(crate) bevy::math::Vec2);

#[derive(Component)]
pub(crate) struct AngularVelocity(pub(crate) f32);

#[derive(Component)]
pub(crate) struct Rotation(pub(crate) f32);

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

/*
 Assigns the rotation to the transform by copying it from the Rotation component.
*/
pub(crate) fn apply_rotation_to_mesh(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in &mut query {
        transform.rotation = Quat::from_rotation_z(rotation.0);
    }
}

pub(crate) fn wrap_entities(
    mut query: Query<&mut Transform, With<Wrapping>>,
    world_size: Res<WorldSize>,
) {
    let right = world_size.width / 2.0;
    let left = -right;
    let top = world_size.height / 2.0;
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
