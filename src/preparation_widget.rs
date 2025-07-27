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

fn spawn_get_ready(
    mut commands: Commands,
){
    todo!();
}

fn despawn_get_ready(
    mut commands: Commands,
){
    todo!();
}

fn animate_get_ready_widget(){
    todo!();
}
