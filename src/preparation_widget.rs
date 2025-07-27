use bevy::{
    color::palettes::css::{BLACK, GREEN, LIGHT_BLUE, RED},
    prelude::*,
};

use crate::GameState;

pub fn preparation_widget_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GetReady), spawn_get_ready)
        .add_systems(OnExit(GameState::GetReady), despawn_get_ready)
        .add_systems(
            Update,
            (animate_get_ready_widget).run_if(in_state(GameState::GetReady)),
        )
        .insert_resource(ReadySetGoTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

/// Marker component for things on the get-ready indicator
#[derive(Component)]
struct OnReadySetGo;

/// Newtype wrapper for `Timer`. Used to count down during the "get ready" phase.
#[derive(Deref, DerefMut, Resource)]
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

// TODO: Replace this with a generic somewhere else in the crate
// want: `despawn_screen::<OnReadySetGo>>()`
fn despawn_get_ready(mut commands: Commands, to_despawn: Query<Entity, With<OnReadySetGo>>) {
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

fn animate_get_ready_widget(
    mut text_segment: Single<&mut Text, With<CountdownText>>,
    time: Res<Time>,
    mut timer: ResMut<ReadySetGoTimer>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    // Advance the timer, read the remaining time and write it onto the label.
    timer.tick(time.delta());

    // Add one to the visual value so the countdown starts at 3 and stops at 1.
    // Otherwise it starts at 2 and disappears after showing 0.
    // That feels wrong even though it's functionally identical.
    let tval = timer.0.remaining().as_secs() + 1;
    **text_segment = format!("{tval}").into();
    
    // If the timer has expired, change state to playing.
    if timer.finished() {
        game_state.set(GameState::Playing);
    }
}
