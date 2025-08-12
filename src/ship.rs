use crate::{
    AngularVelocity, GameAssets, GameState, Lives,
    objects::Asteroid,
    events::{BulletDestroy, ShipDestroy},
    physics::{Velocity, Wrapping},
};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Bullet;

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
