mod asteroids;
pub mod config;
mod event;
mod physics;
mod preparation_widget;
mod ship;
mod title_screen;

use crate::asteroids::{Asteroid, AsteroidSpawner};
use crate::config::{
    ASTEROID_SMALL_COLOR, BACKGROUND_COLOR, BULLET_COLOR, BULLET_LIFETIME, BULLET_SPEED,
    PLAYER_SHIP_COLOR, SHIP_ROTATION, SHIP_THRUST, SHIP_THRUSTER_COLOR_ACTIVE,
    SHIP_THRUSTER_COLOR_INACTIVE, WINDOW_SIZE,
};
use crate::physics::AngularVelocity;
use crate::ship::{Bullet, Ship};

use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::*,
    render::RapierDebugRenderPlugin,
};

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            title_screen::GameMenuPlugin,
            preparation_widget::preparation_widget_plugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(WorldSize {
            width: WINDOW_SIZE.x,
            height: WINDOW_SIZE.y,
        })
        .insert_resource(Lives(3))
        .register_type::<Lives>()
        .insert_resource(Score(0))
        .insert_resource(AsteroidSpawner::new())
        .init_resource::<GameAssets>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Playing), (ship::spawn_player, spawn_ui))
        .add_systems(
            FixedUpdate,
            (
                input_ship_thruster,
                input_ship_rotation,
                input_ship_shoot,
                physics::wrap_entities,
                asteroids::tick_asteroid_manager,
                asteroids::spawn_asteroid.after(asteroids::tick_asteroid_manager),
                asteroids::split_asteroids,
                ship::bullet_impact_listener,
                collision_listener,
                // TODO: Remove debug printing
                debug_collision_event_printer,
                tick_lifetimes,
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
        .add_event::<asteroids::SpawnAsteroid>()
        .add_event::<event::AsteroidDestroy>()
        .add_event::<event::ShipDestroy>()
        .add_event::<event::BulletDestroy>();
        app.insert_state(GameState::Playing);
    }
}

fn debug_collision_event_printer(mut collision_events: EventReader<CollisionEvent>) {
    for event in collision_events.read() {
        dbg!(event);
    }
}

/// The collision event routing system.
///
/// When a `CollisionEvent` occurrs, this system checks which things collided
/// and emits secondary events accordignly.
///
/// | Objects | Response |
/// |-|-|
/// | Ship & Asteroid | emits event [`ShipDestroy`](`crate::event::ShipDestroy`) |
/// | Asteroid & Bullet | emits event [`AsteroidDestroy`](`crate::event::AsteroidDestroy`) |
/// | Asteroid & Asteroid | Nothing. Asteroids won't collide with each other |
/// | Bullet & Bullet | Nothing. Bullets won't collide with each other (and probably can't under normal gameplay conditions) |
/// | Bullet & Ship | Nothing. The player shouldn't be able to shoot themselves (and the Flying Saucer hasn't been impl.'d, so it's bullets don't count) |
fn collision_listener(
    mut collisions: EventReader<CollisionEvent>,
    mut ship_writer: EventWriter<event::ShipDestroy>,
    mut asteroid_writer: EventWriter<event::AsteroidDestroy>,
    mut bullet_writer: EventWriter<event::BulletDestroy>,
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
                    ship_writer.write(event::ShipDestroy);
                } // else, we don't care
            } else if *two == *player {
                if rocks.contains(*one) {
                    dbg!("Writing ShipDestroy event");
                    ship_writer.write(event::ShipDestroy);
                }
            }

            // Option 2: Bullet & Asteroid
            if bullets.contains(*one) {
                if rocks.contains(*two) {
                    dbg!("Writing AsteroidDestroy & BulletDestroy events");
                    asteroid_writer.write(event::AsteroidDestroy(*two));
                    bullet_writer.write(event::BulletDestroy(*one));
                }
            } else if rocks.contains(*one) {
                if bullets.contains(*two) {
                    dbg!("Writing AsteroidDestroy & BulletDestroy events");
                    asteroid_writer.write(event::AsteroidDestroy(*one));
                    bullet_writer.write(event::BulletDestroy(*two));
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States)]
pub enum GameState {
    TitleScreen, // Program is started. Present title screen and await user start
    GetReady,    // Short timer to let the player get ready after pressing start
    Playing,     // Player has started the game. Run the main loop
    GameOver,    // Game has ended. Present game over dialogue and await user restart
}

#[derive(Resource, Debug, Deref, Clone, Copy)]
struct Score(i32);

impl From<Score> for String {
    fn from(value: Score) -> Self {
        value.to_string()
    }
}

#[derive(Component)]
struct Lifetime(Timer);

#[derive(InspectorOptions, Reflect, Resource, Debug, Deref, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
struct Lives(i32);

impl From<Lives> for String {
    fn from(value: Lives) -> Self {
        value.to_string()
    }
}

#[derive(Resource)]
struct WorldSize {
    width: f32,
    height: f32,
}

#[derive(Resource)]
struct GameAssets {
    meshes: [Handle<Mesh>; 5],
    materials: [Handle<ColorMaterial>; 7],
}

impl GameAssets {
    fn ship(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[0].clone(), self.materials[0].clone())
    }

    // The thruster mesh is actually just the ship mesh
    fn thruster_mesh(&self) -> Handle<Mesh> {
        self.meshes[0].clone()
    }

    // TODO: Look into parameterizing the material
    // A shader uniform should be able to do this, but I don't know how to
    // load those in Bevy.
    fn thruster_mat_inactive(&self) -> Handle<ColorMaterial> {
        self.materials[1].clone()
    }

    fn thruster_mat_active(&self) -> Handle<ColorMaterial> {
        self.materials[2].clone()
    }

    fn asteroid_small(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[1].clone(), self.materials[1].clone())
    }

    fn asteroid_medium(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[2].clone(), self.materials[2].clone())
    }

    fn asteroid_large(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[3].clone(), self.materials[3].clone())
    }

    fn bullet(&self) -> (Handle<Mesh>, Handle<ColorMaterial>) {
        (self.meshes[4].clone(), self.materials[6].clone())
    }
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let mut world_meshes = world.resource_mut::<Assets<Mesh>>();
        let meshes = [
            world_meshes.add(Triangle2d::new(
                Vec2::new(0.5, 0.0),
                Vec2::new(-0.5, 0.45),
                Vec2::new(-0.5, -0.45),
            )),
            world_meshes.add(Circle::new(10.0)),
            world_meshes.add(Circle::new(20.0)),
            world_meshes.add(Circle::new(40.0)),
            world_meshes.add(Circle::new(0.2)),
        ];
        let mut world_materials = world.resource_mut::<Assets<ColorMaterial>>();
        let materials = [
            world_materials.add(PLAYER_SHIP_COLOR),
            world_materials.add(SHIP_THRUSTER_COLOR_INACTIVE),
            world_materials.add(SHIP_THRUSTER_COLOR_ACTIVE),
            world_materials.add(ASTEROID_SMALL_COLOR),
            // TODO: asteroid medium and large colors
            world_materials.add(ASTEROID_SMALL_COLOR),
            world_materials.add(ASTEROID_SMALL_COLOR),
            world_materials.add(BULLET_COLOR),
        ];
        GameAssets { meshes, materials }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/*
 Checks if "W" is pressed and increases velocity accordingly.
*/
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

/*
 Checks if "A" or "D" is pressed and updates the player's Rotation component accordingly
 Does *not* rotate the graphical widget! (that's done by the `apply_rotation_to_mesh` system)
*/
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

fn input_ship_shoot(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ship: Single<(&Transform, &physics::Velocity), With<Ship>>,
    game_assets: Res<GameAssets>,
    mut commands: Commands,
) {
    let (ship_pos, ship_vel) = *ship;

    // Derive bullet velocity, add to the ship's velocity
    let bullet_vel = (ship_pos.rotation * Vec3::X).xy() * BULLET_SPEED;
    let bullet_vel = bullet_vel + ship_vel.0;

    // TODO: create a timer for the gun fire rate.
    // For now, spawn one for each press of the spacebar.
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.spawn((
            ship::Bullet,
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

fn spawn_ui(mut commands: Commands, score: Res<Score>, lives: Res<Lives>) {
    commands.spawn((
        Text::new(format!("Score: {score:?} | Lives: {lives:?}")),
        TextFont::from_font_size(25.0),
    ));
}

fn tick_lifetimes(mut commands: Commands, time: Res<Time>, query: Query<(Entity, &mut Lifetime)>) {
    for (e, mut life) in query {
        life.0.tick(time.delta());
        if life.0.just_finished() {
            commands.entity(e).despawn();
        }
    }
}
