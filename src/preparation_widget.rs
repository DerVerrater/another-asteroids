use bevy::prelude::*;

use crate::GameState;

pub fn preparation_widget_plugin(app: &mut App) {
	app.add_systems(OnEnter(GameState::GetReady), spawn_get_ready)
		.add_systems(OnExit(GameState::GetReady), despawn_get_ready)
		.add_systems(Update, (animate_get_ready_widget).run_if(in_state(GameState::GetReady)));
}

/// Marker component for things on the get-ready indicator
#[derive(Component)]
struct OnReadySetGo;

/// Newtype wrapper for `Timer`. Used to count down during the "get ready" phase.
#[derive(Component)]
struct ReadySetGoTimer(Timer);

/// Marker for the counter text segment
#[derive(Component)]
struct CountdownText;

fn spawn_get_ready(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        OnReadySetGo, // marker, so this can be de-spawned properly
        Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            width: Val::Percent(30.),
            height: Val::Percent(30.),
            ..default()
        },
        BackgroundColor(LIGHT_BLUE.into()),
        children![
            (Text::new("Get Ready!"), TextColor(BLACK.into())),
            (
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(10.),
                    ..default()
                },
                BackgroundColor(GREEN.into()),
            ),
            (
                CountdownText,
                Text::new("<uninit timer>"),
                TextColor(RED.into()),
            )
        ],
    ));
}

fn despawn_get_ready(
    mut commands: Commands,
){
    todo!();
}

fn animate_get_ready_widget(){
    todo!();
}
