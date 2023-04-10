use bevy::prelude::*;

use crate::game_state::GameState;

pub struct WinScreenPlugin;
impl Plugin for WinScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_win_screen.in_schedule(OnEnter(GameState::WinScreen)))
            .add_system(despawn_win_screen.in_schedule(OnExit(GameState::WinScreen)));
    }
}

#[derive(Component)]
struct WinMarker;

fn spawn_win_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        WinMarker,
        SpriteBundle {
            texture: asset_server.load("end-screen.png"),
            transform: Transform::from_xyz(0., 0., 1.0),
            ..default()
        },
    ));
}

fn despawn_win_screen(mut commands: Commands, q: Query<Entity, With<WinMarker>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
