use crate::{
    GameState,
    config::{UI_BUTTON_HOVERED, UI_BUTTON_NORMAL, UI_BUTTON_PRESSED},
    resources::{Lives, Score},
};

use bevy::{
    color::palettes::css::{BLACK, GREEN, LIGHT_BLUE, RED, WHITE},
    prelude::*,
};

/// Plugin for the main menu
pub struct PluginGameMenu;

impl Plugin for PluginGameMenu {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TitleScreen), spawn_menu)
            .add_systems(OnExit(GameState::TitleScreen), despawn::<MarkerMainMenu>)
            .add_systems(
                Update,
                handle_spacebar.run_if(in_state(GameState::TitleScreen)),
            );
    }
}

/// Plugin for the "get ready" period as gameplay starts.
pub struct PluginGetReady;

impl Plugin for PluginGetReady {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GetReady), spawn_get_ready)
            .add_systems(OnExit(GameState::GetReady), despawn::<OnReadySetGo>)
            .add_systems(
                Update,
                (animate_get_ready_widget).run_if(in_state(GameState::GetReady)),
            )
            .insert_resource(ReadySetGoTimer(Timer::from_seconds(3.0, TimerMode::Once)));
    }
}

/// Plugin for the game-over screen (TODO)
pub struct PluginGameOver;

impl Plugin for PluginGameOver {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), spawn_gameover_ui)
            .add_systems(OnExit(GameState::GameOver), despawn::<MarkerGameOver>)
            .add_systems(
                Update,
                operate_gameover_ui.run_if(in_state(GameState::GameOver)),
            );
    }
}

// Marker component for the title screen UI entity.
// This way, a query for the TitleUI can be used to despawn the title screen
#[derive(Component)]
struct MarkerMainMenu;

/// Marker component for things on the get-ready indicator
#[derive(Component)]
struct OnReadySetGo;

/// Marker for things on the game-over screen
#[derive(Component)]
struct MarkerGameOver;

/// Action specifier for the game-over menu's buttons.
///
/// Attach this component to a button and [`PluginGameOver`] will use it to
/// decide what to do when that button is pressed.
#[derive(Component)]
enum GameOverMenuAction {
    ToMainMenu,
    Quit,
}

/// Newtype wrapper for `Timer`. Used to count down during the "get ready" phase.
#[derive(Deref, DerefMut, Resource)]
struct ReadySetGoTimer(Timer);

/// Marker for the counter text segment
#[derive(Component)]
struct CountdownText;

/// Marker for the counter bar segment
#[derive(Component)]
struct CountdownBar;

/// Despawns entities matching the generic argument. Intended to remove UI
/// elements.
fn despawn<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            MarkerMainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                Text::new("Robert's Bad Asteroids Game"),
                TextFont::from_font_size(50.0),
                TextLayout::new_with_justify(JustifyText::Center),
                TextShadow::default(),
            ));
            cmds.spawn((
                Text::new("Press space to begin"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextShadow::default(),
            ));
        });
}

fn spawn_get_ready(mut commands: Commands) {
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
                CountdownBar,
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

/// Spawns the game over screen.
///
/// Used by [`PluginGameOver`] when entering the [`GameState::GameOver`] state.
fn spawn_gameover_ui(mut commands: Commands) {
    commands.spawn((
        MarkerGameOver, // Marker, so `despawn<T>` can remove this on state exit.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Button,
                GameOverMenuAction::ToMainMenu,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(UI_BUTTON_NORMAL),
                children![(
                    Text::new("Main Menu"),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )]
            ),
            (
                Button,
                GameOverMenuAction::Quit,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(UI_BUTTON_NORMAL),
                children![(
                    Text::new("Quit"),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )]
            )
        ],
    ));
}

fn animate_get_ready_widget(
    mut text_segment: Single<&mut Text, With<CountdownText>>,
    mut bar_segment: Single<&mut Node, With<CountdownBar>>,
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

    // Shrink the progress bar Node
    bar_segment.width = Val::Percent(100.0 * (1.0 - timer.fraction()));

    // If the timer has expired, change state to playing.
    if timer.finished() {
        game_state.set(GameState::Playing);
    }
}

/// Handles interaction and performs updates to the game-over UI.
///
/// Used by [`PluginGameOver`] while in the [`GameState::GameOver`] state.
///
/// Mostly a button input handler, but it also makes for a convenient single
/// place to keep all system logic for this plugin.
fn operate_gameover_ui(
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &GameOverMenuAction,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    // TODO: Better colors. These are taken from the example and they're ugly.
    for (interaction, mut color, mut border_color, menu_action) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *color = UI_BUTTON_PRESSED.into();
                border_color.0 = RED.into();
                match menu_action {
                    GameOverMenuAction::ToMainMenu => {
                        game_state.set(GameState::TitleScreen);
                    }
                    GameOverMenuAction::Quit => {
                        app_exit_events.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *color = UI_BUTTON_HOVERED.into();
                border_color.0 = WHITE.into();
            }
            Interaction::None => {
                *color = UI_BUTTON_NORMAL.into();
                border_color.0 = BLACK.into();
            }
        }
    }
}

/// Main menu input listener. Starts game when the spacebar is pressed.
fn handle_spacebar(input: Res<ButtonInput<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if input.just_pressed(KeyCode::Space) {
        game_state.set(GameState::GetReady);
    }
}

pub fn spawn_ui(mut commands: Commands, score: Res<Score>, lives: Res<Lives>) {
    commands.spawn((
        Text::new(format!("Score: {score:?} | Lives: {lives:?}")),
        TextFont::from_font_size(25.0),
    ));
}
