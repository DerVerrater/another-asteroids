use crate::{
    GameAssets, Rotation,
    physics::{Position, Velocity, Wrapping},
};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Ship;

pub fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
            Collider::ball(0.7),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::STATIC_STATIC,
            Ship,
            Wrapping,
            Position(Vec2::default()),
            Velocity(Vec2::ZERO),
            Rotation(0.0),
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
