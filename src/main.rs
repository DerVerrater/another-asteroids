use bevy::{prelude::*, window::WindowResolution};

use asteroids::{AsteroidPlugin, config::WINDOW_SIZE};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#game-canvas".to_owned()),
                resolution: WindowResolution::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(AsteroidPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}
