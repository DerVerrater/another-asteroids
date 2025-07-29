pub mod config;
mod preparation_widget;
mod title_screen;

use std::time::Duration;

use crate::config::{BACKGROUND_COLOR, PLAYER_SHIP_COLOR, SHIP_ROTATION, SHIP_THRUST, WINDOW_SIZE};

use bevy::prelude::*;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;

use config::{ASTEROID_SMALL_COLOR, SHIP_THRUSTER_COLOR_ACTIVE, SHIP_THRUSTER_COLOR_INACTIVE};

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            title_screen::GameMenuPlugin,
            preparation_widget::preparation_widget_plugin,
        ))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(WorldSize {
            width: WINDOW_SIZE.x,
            height: WINDOW_SIZE.y,
        })
        .insert_resource(Lives(3))
        .register_type::<Lives>()
        .insert_resource(Score(0))
        .insert_resource(AsteroidSpawner {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
        })
        .init_resource::<GameAssets>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Playing), (spawn_player, spawn_ui))
        .add_systems(
            FixedUpdate,
            (
                input_ship_thruster,
                input_ship_rotation,
                wrap_entities,
                tick_asteroid_manager,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedPostUpdate,
            (integrate_velocity, update_positions, apply_rotation_to_mesh)
                .run_if(in_state(GameState::Playing)),
        );
        app.insert_state(GameState::TitleScreen);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, States)]
pub enum GameState {
    TitleScreen, // Program is started. Present title screen and await user start
    GetReady,    // Short timer to let the player get ready after pressing start
    Playing,     // Player has started the game. Run the main loop
    GameOver,    // Game has ended. Present game over dialogue and await user restart
}

#[derive(Component)]
struct Position(bevy::math::Vec2);

#[derive(Component)]
struct Velocity(bevy::math::Vec2);

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Ship;

#[derive(Component, Deref, DerefMut)]
struct Asteroid(AsteroidSize);

enum AsteroidSize {
    SMALL,
    MEDIUM,
    LARGE,
}

#[derive(Resource)]
struct AsteroidSpawner {
    timer: Timer,
    // TODO: Configurables?
    // - interval
    // - density
    // - size distribution
}

/// Marker for any entity that should wrap on screen edges
#[derive(Component)]
struct Wrapping;

#[derive(Resource, Debug, Deref, Clone, Copy)]
struct Score(i32);

impl From<Score> for String {
    fn from(value: Score) -> Self {
        value.to_string()
    }
}

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
    meshes: [Handle<Mesh>; 4],
    materials: [Handle<ColorMaterial>; 6],
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
        ];
        GameAssets { meshes, materials }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
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

/// Update the asteroid spawn timer and spawn any asteroids
/// that are due this frame.
fn tick_asteroid_manager(
    mut commands: Commands,
    mut spawner: ResMut<AsteroidSpawner>,
    game_assets: Res<GameAssets>,
    time: Res<Time>,
) {
    spawner.timer.tick(time.delta());
    if spawner.timer.just_finished() {
        commands.spawn((
            Asteroid(AsteroidSize::SMALL),
            Position(Vec2::new(40.0, 40.0)),
            Velocity(Vec2::new(10.0, 0.0)),
            Rotation(0.0),
            Mesh2d(game_assets.asteroid_small().0),
            MeshMaterial2d(game_assets.asteroid_small().1),
        ));
    }
}

/// Utility function to spawn a single asteroid of a given type
/// TODO: convert to an event listener monitoring for "spawn asteroid" events
/// from the `fn tick_asteroid_manager(...)` system.
fn spawn_asteroid(mut commands: Commands) {
    todo!();
}

/*
 Checks if "W" is pressed and increases velocity accordingly.
*/
fn input_ship_thruster(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Rotation, &mut Children), With<Ship>>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    // TODO: Maybe change for a Single<Ship>> so this only runs for the one ship
    // buuut... that would silently do nothing if there are 0 or >1 ships, and
    // I might want to crash on purpose in that case.
    let Ok((mut velocity, rotation, children)) = query.single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    let thrusters = children
        .first()
        .expect("Couldn't find first child, which should be the thruster");

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0 += Vec2::from_angle(rotation.0) * SHIP_THRUST;
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
    mut query: Query<&mut Rotation, With<Ship>>,
) {
    let Ok(mut rotation) = query.single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    if keyboard_input.pressed(KeyCode::KeyA) {
        rotation.0 += SHIP_ROTATION;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        rotation.0 -= SHIP_ROTATION;
    }
}

// TODO: Combine movement integration steps into one function
// They need to be ordered so the physics is deterministic. Bevy can enforce
// order, but it makes more sense to cut out the extra machinery and have one
// single function. Probably better for cache locality or whatever, too.
/*
 Add velocity to position
*/
fn integrate_velocity(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in &mut query {
        position.0 += velocity.0 * time.delta_secs();
    }
}

fn update_positions(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut query {
        transform.translation.x = position.0.x;
        transform.translation.y = position.0.y;
    }
}

/*
 Assigns the rotation to the transform by copying it from the Rotation component.
*/
fn apply_rotation_to_mesh(mut query: Query<(&mut Transform, &Rotation)>) {
    for (mut transform, rotation) in &mut query {
        transform.rotation = Quat::from_rotation_z(rotation.0);
    }
}

fn wrap_entities(mut query: Query<&mut Position, With<Wrapping>>, world_size: Res<WorldSize>) {
    let right = world_size.width / 2.0;
    let left = -right;
    let top = world_size.height / 2.0;
    let bottom = -top;

    for mut pos in query.iter_mut() {
        if pos.0.x > right {
            pos.0.x = left;
        } else if pos.0.x < left {
            pos.0.x = right;
        }

        if pos.0.y > top {
            pos.0.y = bottom;
        } else if pos.0.y < bottom {
            pos.0.y = top;
        }
    }
}

fn spawn_ui(mut commands: Commands, score: Res<Score>, lives: Res<Lives>) {
    commands.spawn((
        Text::new(format!("Score: {score:?} | Lives: {lives:?}")),
        TextFont::from_font_size(25.0),
    ));
}
