mod config;

use crate::config::{BACKGROUND_COLOR, PLAYER_SHIP_COLOR, SHIP_THRUST, SHIP_ROTATION};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, spawn_player))
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(FixedUpdate, (input_ship_thruster, input_ship_rotation))
            // .add_systems(FixedUpdate, input_ship_rotation);
            .add_systems(FixedPostUpdate, (integrate_velocity, update_positions));
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Ship,
        Position(Vec2::default()),
        Velocity(Vec2::ZERO),
        Rotation(0.0),
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(10.0)).into(),
            material: materials.add(PLAYER_SHIP_COLOR),
            transform: Transform::default(),
            ..default()
        },
    ));
}

/*
 Checks if "W" is pressed and increases velocity accordingly.
*/
fn input_ship_thruster(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &Rotation), With<Ship>>,
) {
    let Ok((mut velocity, rotation)) = query.get_single_mut() else {
        let count = query.iter().count();
        panic!("There should be exactly one player ship! Instead, there seems to be {count}.");
    };

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity.0 += Vec2::from_angle(rotation.0) * SHIP_THRUST;
    }
}

/*
 Checks if "A" or "D" is pressed and rotates the ship accordingly
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
