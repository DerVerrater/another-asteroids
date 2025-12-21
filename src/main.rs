use bevy::{prelude::*, window::WindowResolution};

use asteroids::{AsteroidPlugin, config::WINDOW_SIZE};

#[cfg(feature = "debug_ui")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
#[cfg(feature = "debug_ui")]
use bevy_rapier2d::render::RapierDebugRenderPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#game-canvas".to_owned()),
            resolution: WindowResolution::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(AsteroidPlugin);

    #[cfg(feature = "debug_ui")]
    app.add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierDebugRenderPlugin::default());

    app.run();
}
