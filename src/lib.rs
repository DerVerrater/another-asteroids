//! Asteroids, implemented as a Bevy plugin.
//!
//! Compile-time configurables can be found in the [`config`] module.

pub mod config;
mod events;
mod machinery;
mod objects;
mod physics;
mod resources;
mod widgets;

use crate::config::{
    ASTEROID_SMALL_COLOR, BACKGROUND_COLOR, BULLET_COLOR, BULLET_LIFETIME, BULLET_SPEED,
    PLAYER_SHIP_COLOR, SHIP_ROTATION, SHIP_THRUST, SHIP_THRUSTER_COLOR_ACTIVE,
    SHIP_THRUSTER_COLOR_INACTIVE,
};
use crate::machinery::AsteroidSpawner;
use crate::objects::{Bullet, Ship, Weapon};
use crate::physics::AngularVelocity;

use bevy::prelude::*;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::*,
    render::RapierDebugRenderPlugin,
};
use machinery::Lifetime;
use resources::{GameAssets, Lives, Score, WorldSize};

/// The main game plugin.
pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            widgets::PluginGameMenu,
            widgets::PluginGameOver,
            widgets::PluginGetReady,
            widgets::PluginGameHud,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(WorldSize::default())
        .insert_resource(Lives(3))
        .register_type::<Lives>()
        .insert_resource(Score(0))
        .register_type::<Score>()
        .insert_resource(AsteroidSpawner::new())
        .init_resource::<GameAssets>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Playing), objects::spawn_player)
        .add_systems(
            FixedUpdate,
            (
                input_ship_thruster,
                input_ship_rotation,
                input_ship_shoot,
                physics::wrap_entities,
                machinery::tick_asteroid_manager,
                objects::spawn_asteroid.after(machinery::tick_asteroid_manager),
                objects::split_asteroids,
                objects::bullet_impact_listener,
                objects::ship_impact_listener,
                physics::collision_listener,
                machinery::tick_lifetimes,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedPostUpdate,
            (
                physics::integrate_velocity,
                physics::integrate_angular_velocity,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_event::<events::SpawnAsteroid>()
        .add_event::<events::AsteroidDestroy>()
        .add_event::<events::ShipDestroy>()
        .add_event::<events::BulletDestroy>();
        app.insert_state(GameState::TitleScreen);
    }
}

/// The game's main state tracking mechanism, registered with Bevy as a [`State`](`bevy::prelude::State`).
#[derive(Clone, Debug, Eq, Hash, PartialEq, States)]
pub enum GameState {
    TitleScreen, // Program is started. Present title screen and await user start
    GetReady,    // Short timer to let the player get ready after pressing start
    Playing,     // Player has started the game. Run the main loop
    GameOver,    // Game has ended. Present game over dialogue and await user restart
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Player's thruster control system.
///
/// Checks if "W" is pressed and increases velocity accordingly.
fn input_ship_thruster(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut physics::Velocity, &Transform, &mut Children), With<Ship>>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    // TODO: Maybe change for a Single<Ship>> so this only runs for the one ship
    // buuut... that would silently do nothing if there are 0 or >1 ships, and
    // I might want to crash on purpose in that case.
    let Ok((mut velocity, transform, children)) = query.single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    let thrusters = children
        .first()
        .expect("Couldn't find first child, which should be the thruster");

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0 += (transform.rotation * Vec3::X).xy() * SHIP_THRUST;
        commands
            .entity(*thrusters)
            .insert(MeshMaterial2d(game_assets.thruster_mat_active()));
    } else {
        commands
            .entity(*thrusters)
            .insert(MeshMaterial2d(game_assets.thruster_mat_inactive()));
    }
}

/// Player's rotation control system.
///
/// Checks if "A" or "D" is pressed and updates the player's [`AngularVelocity`]
/// component accordingly.
fn input_ship_rotation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut AngularVelocity, With<Ship>>,
) {
    let Ok(mut angular_vel) = query.single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    if keyboard_input.pressed(KeyCode::KeyA) {
        angular_vel.0 = SHIP_ROTATION;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        angular_vel.0 = -SHIP_ROTATION;
    } else {
        angular_vel.0 = 0.0;
    }
}

/// Player's gun trigger.
///
/// Checks if the spacebar has just been pressed, spawning a bullet if so.
///
/// TODO: Hook up a timer to control weapon fire-rate. Something will have to
/// tick those timers. Maybe this system?
fn input_ship_shoot(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ship: Single<(&Transform, &physics::Velocity, &mut Weapon), With<Ship>>,
    game_assets: Res<GameAssets>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let (ship_pos, ship_vel, mut weapon) = ship.into_inner();

    // Tick the timer so the cooldown eventually finishes. Once it does, the
    // value will clamp at 0. The weapon is now ready.
    weapon.tick(time.delta());

    // If the weapon is ready and the player presses the trigger,
    // spawn a bullet & reset the timer.
    if weapon.finished() && keyboard_input.pressed(KeyCode::Space) {
        weapon.reset();
        // Derive bullet velocity, add to the ship's velocity
        let bullet_vel = (ship_pos.rotation * Vec3::X).xy() * BULLET_SPEED;
        let bullet_vel = bullet_vel + ship_vel.0;

        commands.spawn((
            Bullet,
            Collider::ball(0.2),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::STATIC_STATIC,
            physics::Velocity(bullet_vel),
            Mesh2d(game_assets.bullet().0),
            MeshMaterial2d(game_assets.bullet().1),
            ship_pos.clone(), // clone ship transform
            Lifetime(Timer::from_seconds(BULLET_LIFETIME, TimerMode::Once)),
        ));
    }
}
