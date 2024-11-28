use bevy::{prelude::*, window::WindowResolution};

use asteroids::{config::WINDOW_SIZE, AsteroidPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AsteroidPlugin)
        .run();
}
