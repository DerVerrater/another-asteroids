use crate::GameState;

use bevy::prelude::*;

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
	fn build(&self, app: &mut App){
		app
			.add_systems(OnEnter(GameState::TitleScreen), spawn_menu)
			.add_systems(OnExit(GameState::TitleScreen), despawn_menu);
	}
}

// Marker component for the title screen UI entity.
// This way, a query for the TitleUI can be used to despawn the title screen
#[derive(Component)]
struct TitleUI;

fn spawn_menu(mut commands: Commands) {
	commands.spawn(Camera2d);
    commands
        .spawn((
            TitleUI,
            Node {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                Text::new("Robert's Bad Asteroids Game"),
                TextFont::from_font_size(50.0),
            ));
            cmds.spawn((
                Text::new("Press space to begin"),
                TextFont::from_font_size(40.0),
            ));
        });
}

fn despawn_menu() {
	todo!();
}