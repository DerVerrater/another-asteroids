use bevy::prelude::*;

use asteroids::AsteroidPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AsteroidPlugin)
        .run();
}
