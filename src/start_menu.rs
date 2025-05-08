use bevy::prelude::*;

use crate::game_state::GameState;

const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.15, 0.);
const HOVERED_BUTTON: Color = Color::srgba(0.25, 0.25, 0.25, 0.);
const PRESSED_BUTTON: Color = Color::srgba(0.35, 0.75, 0.35, 0.);
pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::StartMenu), spawn_menu)
            .add_systems(
                Update,
                (input_start, button_system).distributive_run_if(in_state(GameState::StartMenu)),
            )
            .add_systems(OnExit(GameState::StartMenu), despawn_menu);
    }
}

#[derive(Component)]
pub struct MenuMarker;

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            MenuMarker,
            Node {
                height: Val::Percent(100.0),
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    MenuMarker,
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        MenuMarker,
                        Text("Click or Press Space to Start".into()),
                        TextFont {
                            font: asset_server.load("Rubik-Light.ttf"),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });

    commands.spawn((
        MenuMarker,
        Sprite::from_image(asset_server.load("start-screen.png")),
        Transform::from_xyz(360., 360., 1.0),
    ));
}

fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuMarker>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                state.set(GameState::LoadLevel);
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn input_start(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    gamepads: Query<&Gamepad>,
) {
    if keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::Enter) {
        state.set(GameState::LoadLevel);
    }

    for gamepad in gamepads.iter() {
        if gamepad.pressed(GamepadButton::Start) || gamepad.pressed(GamepadButton::South) {
            state.set(GameState::LoadLevel);
        }
    }
}
