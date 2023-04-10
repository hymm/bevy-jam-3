use bevy::prelude::*;

use crate::game_state::GameState;

const NORMAL_BUTTON: Color = Color::rgba(0.15, 0.15, 0.15, 0.);
const HOVERED_BUTTON: Color = Color::rgba(0.25, 0.25, 0.25, 0.);
const PRESSED_BUTTON: Color = Color::rgba(0.35, 0.75, 0.35, 0.);
pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_menu.in_schedule(OnEnter(GameState::StartMenu)))
            .add_systems(
                (input_start, button_system).distributive_run_if(in_state(GameState::StartMenu)),
            )
            .add_system(despawn_menu.in_schedule(OnExit(GameState::StartMenu)));
    }
}

#[derive(Component)]
pub struct MenuMarker;

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            MenuMarker,
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    MenuMarker,
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        MenuMarker,
                        TextBundle::from_section(
                            "Click or Press Space to Start",
                            TextStyle {
                                font: asset_server.load("Rubik-Light.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                    ));
                });
        });

    commands.spawn((
        MenuMarker,
        SpriteBundle {
            texture: asset_server.load("start-screen.png"),
            transform: Transform::from_xyz(0., 0., 1.0),
            ..default()
        },
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
            Interaction::Clicked => {
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
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<NextState<GameState>>,
    button_inputs: Res<Input<GamepadButton>>,
    gamepads: Res<Gamepads>,
) {
    if keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::Return) {
        state.set(GameState::LoadLevel);
    }

    for gamepad in gamepads.iter() {
        if button_inputs.pressed(GamepadButton::new(gamepad, GamepadButtonType::Start))
            || button_inputs.pressed(GamepadButton::new(gamepad, GamepadButtonType::South))
        {
            state.set(GameState::LoadLevel);
        }
    }
}
