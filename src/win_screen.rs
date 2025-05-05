use bevy::prelude::*;

use crate::game_state::GameState;

pub struct WinScreenPlugin;
impl Plugin for WinScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::WinScreen), spawn_win_screen)
            .add_systems(OnExit(GameState::WinScreen), despawn_win_screen);
    }
}

#[derive(Component)]
struct WinMarker;

fn spawn_win_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        WinMarker,
        Sprite::from_image(asset_server.load("end-screen.png")),
        Transform::from_xyz(360., 360., 1.0),
    ));
}

fn despawn_win_screen(mut commands: Commands, q: Query<Entity, With<WinMarker>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
