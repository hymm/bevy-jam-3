use bevy::prelude::*;

use crate::game_state::GameState;

pub struct WinScreenPlugin;
impl Plugin for WinScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_win_screen.in_schedule(OnEnter(GameState::WinScreen)));
    }
}

#[derive(Component)]
struct WinMarker;

fn spawn_win_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    dbg!("spawn_win");
    commands
        .spawn((
            WinMarker,
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
            parent.spawn((
                WinMarker,
                TextBundle::from_section(
                    "You Win!",
                    TextStyle {
                        font: asset_server.load("FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
            ));
        });
}
