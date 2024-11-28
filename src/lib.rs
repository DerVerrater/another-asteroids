pub mod config;

use crate::config::{BACKGROUND_COLOR, PLAYER_SHIP_COLOR, SHIP_ROTATION, SHIP_THRUST, WINDOW_SIZE};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use config::{SHIP_THRUSTER_COLOR_ACTIVE, SHIP_THRUSTER_COLOR_INACTIVE};

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_camera, spawn_player, spawn_ui),
        )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(WorldSize {
            width: WINDOW_SIZE.x,
            height: WINDOW_SIZE.y,
        })
        .insert_resource(Lives(3))
        .insert_resource(Score(0))
        .add_systems(
            FixedUpdate,
            (input_ship_thruster, input_ship_rotation, wrap_entities),
        )
        .add_systems(
            FixedPostUpdate,
            (integrate_velocity, update_positions, apply_rotation_to_mesh),
        );
    }
}

#[derive(Component)]
struct Position(bevy::math::Vec2);

#[derive(Component)]
struct Velocity(bevy::math::Vec2);

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Ship;

// Data component to store color properties attached to an entity
// This was easier (and imo better) than holding global consts with
// UUID assets.
#[derive(Component)]
struct ThrusterColors(Handle<ColorMaterial>, Handle<ColorMaterial>);

#[derive(Resource, Debug, Deref, Clone, Copy)]
struct Score(i32);

impl From<Score> for String {
    fn from(value: Score) -> Self {
        value.to_string()
    }
}

#[derive(Resource, Debug, Deref, Clone, Copy)]
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let triangle = Triangle2d::new(
        Vec2::new(0.5, 0.0),
        Vec2::new(-0.5, 0.45),
        Vec2::new(-0.5, -0.45),
    );
    let thruster_firing_id = materials.add(SHIP_THRUSTER_COLOR_ACTIVE);
    let thruster_stopped_id = materials.add(SHIP_THRUSTER_COLOR_INACTIVE);

    let ship_mesh = MaterialMesh2dBundle {
        mesh: meshes.add(triangle).into(),
        material: materials.add(PLAYER_SHIP_COLOR),
        transform: Transform::default().with_scale(Vec3::new(20.0, 20.0, 20.0)),
        ..default()
    };

    let thruster_mesh = MaterialMesh2dBundle {
        mesh: meshes.add(triangle).into(),
        material: materials.add(PLAYER_SHIP_COLOR),
        transform: Transform::default()
            .with_scale(Vec3::splat(0.5))
            .with_translation(Vec3::new(-0.5, 0.0, -0.1)),
        ..default()
    };

    let thruster = commands.spawn(thruster_mesh).id();

    let mut ship_id = commands.spawn((
        Ship,
        Position(Vec2::default()),
        Velocity(Vec2::ZERO),
        Rotation(0.0),
        ship_mesh,
        ThrusterColors(thruster_firing_id, thruster_stopped_id),
    ));

    ship_id.add_child(thruster);
}

/*
 Checks if "W" is pressed and increases velocity accordingly.
*/
fn input_ship_thruster(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Rotation, &mut Children, &ThrusterColors), With<Ship>>,
    mut commands: Commands,
) {
    let Ok((mut velocity, rotation, children, colors)) = query.get_single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    let thrusters = children
        .first()
        .expect("Couldn't find first child, which should be the thruster");

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0 += Vec2::from_angle(rotation.0) * SHIP_THRUST;
        commands.entity(*thrusters).insert(colors.0.clone());
    } else {
        commands.entity(*thrusters).insert(colors.1.clone());
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
    let Ok(mut rotation) = query.get_single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    if keyboard_input.pressed(KeyCode::KeyA) {
        rotation.0 += SHIP_ROTATION;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        rotation.0 -= SHIP_ROTATION;
    }
}

/*
 Add velocity to position
*/
fn integrate_velocity(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in &mut query {
        position.0 += velocity.0 * time.delta_seconds();
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

fn wrap_entities(mut query: Query<&mut Position>, world_size: Res<WorldSize>) {
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
    commands.spawn(TextBundle::from_sections([
        TextSection::new(
            "Score: ",
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        ),
        TextSection::new(
            *score,
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        ),
        TextSection::new(
            " | Lives: ",
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        ),
        TextSection::new(
            *lives,
            TextStyle {
                font_size: 25.0,
                ..default()
            },
        )
    ]));
}
